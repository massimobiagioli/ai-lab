//! Worker scheduler loop: tick, lease, dispatch and retry.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use ailab_domain::{AgentId, CardId, RunId};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::orchestration::{LeaseError, LeaseManager};
use crate::pipeline::RunPipeline;

/// A queued run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedRun {
    /// Run.
    pub run_id: RunId,
    /// Card.
    pub card_id: CardId,
    /// Agent.
    pub agent_id: AgentId,
    /// Attempt.
    pub attempt: u8,
    /// Enqueued at.
    pub enqueued_at: DateTime<Utc>,
    /// Card key for branch/PR identity.
    pub card_key: Option<String>,
}

impl QueuedRun {
    /// New first attempt.
    #[must_use]
    pub const fn new(
        run_id: RunId,
        card_id: CardId,
        agent_id: AgentId,
        enqueued_at: DateTime<Utc>,
    ) -> Self {
        Self {
            run_id,
            card_id,
            agent_id,
            attempt: 1,
            enqueued_at,
            card_key: None,
        }
    }

    /// Attach the card key.
    #[must_use]
    pub fn with_card_key(mut self, card_key: impl Into<String>) -> Self {
        self.card_key = Some(card_key.into());
        self
    }
}

/// Executor failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ExecError {
    /// Transient; requeue with backoff.
    #[error("transient failure: {0}")]
    Transient(String),
    /// Terminal; dead-letter.
    #[error("terminal failure: {0}")]
    Terminal(String),
    /// The run was cancelled at a tool boundary.
    #[error("run cancelled")]
    Cancelled,
    /// The run budget was exhausted.
    #[error("budget exhausted")]
    Budget,
}

impl ExecError {
    /// True when the run may be retried.
    #[must_use]
    pub const fn retryable(&self) -> bool {
        matches!(self, Self::Transient(_))
    }
}

/// Executor port (the real adapter wires LLM, sandbox and SCM).
#[async_trait]
pub trait RunExecutor: Send + Sync {
    /// Execute a queued run and return its pipeline.
    async fn execute(&self, run: &QueuedRun) -> Result<RunPipeline, ExecError>;

    /// Reconcile a crashed run against external SCM state.
    async fn recover(
        &self,
        _card_id: CardId,
        _run_id: RunId,
        _card_key: &str,
    ) -> Option<(crate::pipeline::RecoveryAction, Option<String>)> {
        None
    }

    /// Take the most recent pipeline state (including failed runs).
    fn last_pipeline(&self) -> Option<RunPipeline> {
        None
    }
}

/// Probe used to cancel a running run at a tool boundary.
#[async_trait]
pub trait CancelProbe: Send + Sync {
    /// True when the run was cancelled.
    async fn is_cancelled(&self, run_id: RunId) -> bool;
}

/// Worker failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkerError {
    /// The queue lock is poisoned.
    #[error("worker queue lock poisoned")]
    QueuePoisoned,
}

/// Tick report.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TickReport {
    /// Dispatched runs.
    pub dispatched: usize,
    /// Completed runs.
    pub completed: usize,
    /// Failed runs (dead-lettered).
    pub failed: usize,
    /// Requeued runs.
    pub requeued: usize,
    /// Cancelled runs.
    pub cancelled: usize,
    /// Expired leases reaped.
    pub reaped: usize,
}

/// Worker scheduler.
pub struct Worker {
    leases: Arc<LeaseManager>,
    queue: Arc<Mutex<VecDeque<QueuedRun>>>,
    executor: Arc<dyn RunExecutor>,
    concurrency: usize,
    max_attempts: u8,
}

impl Worker {
    /// Build a worker.
    #[must_use]
    pub fn new(
        leases: Arc<LeaseManager>,
        executor: Arc<dyn RunExecutor>,
        concurrency: usize,
    ) -> Self {
        Self {
            leases,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            executor,
            concurrency: concurrency.max(1),
            max_attempts: 3,
        }
    }

    /// Enqueue a run; returns the queue depth.
    pub fn enqueue(&self, run: QueuedRun) -> Result<usize, WorkerError> {
        let mut queue = self.queue.lock().map_err(|_| WorkerError::QueuePoisoned)?;
        queue.push_back(run);
        Ok(queue.len())
    }

    /// Current queue depth.
    #[must_use]
    pub fn queue_depth(&self) -> usize {
        self.queue.lock().map_or(0, |queue| queue.len())
    }

    /// One scheduler tick: reap expired leases and dispatch up to concurrency.
    pub async fn tick(&self, now: DateTime<Utc>) -> Result<TickReport, WorkerError> {
        let reaped = self.leases.reap(now).len();
        let mut report = TickReport {
            reaped,
            ..TickReport::default()
        };
        while report.dispatched < self.concurrency {
            let next = {
                let mut queue = self.queue.lock().map_err(|_| WorkerError::QueuePoisoned)?;
                queue.pop_front()
            };
            let Some(run) = next else {
                break;
            };
            let card_id = run.card_id;
            match self
                .leases
                .acquire(run.card_id, run.run_id, run.agent_id, now)
            {
                Ok(_lease) => {
                    report.dispatched += 1;
                    match self.executor.execute(&run).await {
                        Ok(_pipeline) => report.completed += 1,
                        Err(error) if error.retryable() && run.attempt < self.max_attempts => {
                            let mut retry = run;
                            retry.attempt += 1;
                            self.enqueue(retry)?;
                            report.requeued += 1;
                        }
                        Err(ExecError::Cancelled) => report.cancelled += 1,
                        Err(_) => report.failed += 1,
                    }
                    self.leases.release(card_id);
                }
                Err(LeaseError::Conflict) => {
                    self.enqueue(run)?;
                    break;
                }
                Err(LeaseError::NotFound) => {}
            }
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::pipeline::PipelineStage;

    struct CountingExecutor {
        calls: AtomicUsize,
        fail: bool,
    }

    #[async_trait]
    impl RunExecutor for CountingExecutor {
        async fn execute(&self, _run: &QueuedRun) -> Result<RunPipeline, ExecError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                Err(ExecError::Transient("boom".to_owned()))
            } else {
                Ok(RunPipeline::new(_run.run_id, _run.card_id, Utc::now()))
            }
        }
    }

    struct PipelineExecutor;

    #[async_trait]
    impl RunExecutor for PipelineExecutor {
        async fn execute(&self, run: &QueuedRun) -> Result<RunPipeline, ExecError> {
            let now = Utc::now();
            let mut pipeline = RunPipeline::new(run.run_id, run.card_id, now);
            for _ in 0..12 {
                pipeline
                    .advance(now)
                    .map_err(|error| ExecError::Terminal(error.to_string()))?;
            }
            pipeline.record_step("tool", "hash", 5, Some(0), "ok");
            Ok(pipeline)
        }
    }

    fn queued() -> QueuedRun {
        QueuedRun::new(
            RunId::generate(),
            CardId::generate(),
            AgentId::generate(),
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn tick_respects_concurrency_and_drains_the_queue() {
        let executor = Arc::new(CountingExecutor {
            calls: AtomicUsize::new(0),
            fail: false,
        });
        let worker = Worker::new(Arc::new(LeaseManager::new(900)), executor.clone(), 2);
        for _ in 0..5 {
            worker.enqueue(queued()).expect("enqueue");
        }
        let first = worker.tick(Utc::now()).await.expect("tick");
        assert_eq!(first.dispatched, 2);
        assert_eq!(worker.queue_depth(), 3);
        let second = worker.tick(Utc::now()).await.expect("tick");
        assert_eq!(second.dispatched, 2);
        let third = worker.tick(Utc::now()).await.expect("tick");
        assert_eq!(third.dispatched, 1);
        assert_eq!(executor.calls.load(Ordering::SeqCst), 5);
        assert_eq!(worker.queue_depth(), 0);
    }

    #[tokio::test]
    async fn retryable_failures_requeue_then_dead_letter() {
        let executor = Arc::new(CountingExecutor {
            calls: AtomicUsize::new(0),
            fail: true,
        });
        let worker = Worker::new(Arc::new(LeaseManager::new(900)), executor.clone(), 1);
        worker.enqueue(queued()).expect("enqueue");
        let first = worker.tick(Utc::now()).await.expect("tick");
        assert_eq!(first.requeued, 1);
        let second = worker.tick(Utc::now()).await.expect("tick");
        assert_eq!(second.requeued, 1);
        let third = worker.tick(Utc::now()).await.expect("tick");
        assert_eq!(third.failed, 1);
        assert_eq!(worker.queue_depth(), 0);
        assert_eq!(executor.calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn active_lease_keeps_the_run_queued() {
        let executor = Arc::new(CountingExecutor {
            calls: AtomicUsize::new(0),
            fail: false,
        });
        let leases = Arc::new(LeaseManager::new(900));
        let run = queued();
        let now = Utc::now();
        leases
            .acquire(run.card_id, RunId::generate(), AgentId::generate(), now)
            .expect("lease");
        let worker = Worker::new(leases, executor.clone(), 1);
        worker.enqueue(run).expect("enqueue");
        let report = worker.tick(now).await.expect("tick");
        assert_eq!(report.dispatched, 0);
        assert_eq!(worker.queue_depth(), 1);
        assert_eq!(executor.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn pipeline_executor_reaches_reflect_with_a_trace() {
        let worker = Worker::new(
            Arc::new(LeaseManager::new(900)),
            Arc::new(PipelineExecutor),
            1,
        );
        worker.enqueue(queued()).expect("enqueue");
        let report = worker.tick(Utc::now()).await.expect("tick");
        assert_eq!(report.completed, 1);
        let pipeline = PipelineExecutor.execute(&queued()).await.expect("pipeline");
        assert_eq!(pipeline.stage, PipelineStage::Reflect);
        assert_eq!(pipeline.trace().len(), 12);
        assert_eq!(pipeline.steps.len(), 1);
        assert!(
            pipeline
                .trace_id
                .contains(&pipeline.run_id.as_uuid().to_string())
        );
    }
}
