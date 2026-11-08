//! Postgres-backed scheduler: durable `SKIP LOCKED` leases and heartbeat reaping.
//!
//! The lease is modeled on the `runs` row (`lease_id`, `heartbeat_at`) and the
//! card's `DOING` transition in the same transaction, so two workers cannot
//! lease the same card.

use ailab_domain::{AgentId, CardId, LabId, RunId};
use serde_json::json;

use crate::pipeline::RunStep;
use crate::worker::CancelProbe;
use chrono::{DateTime, Duration, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// A run acquired by a worker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledRun {
    /// Run id.
    pub run_id: RunId,
    /// Card id.
    pub card_id: CardId,
    /// Agent id.
    pub agent_id: AgentId,
    /// Attempt.
    pub attempt: u8,
    /// Lab scope.
    pub lab_id: LabId,
}

/// Durable scheduler over Postgres.
#[derive(Debug, Clone)]
pub struct PostgresScheduler {
    pool: PgPool,
}

impl PostgresScheduler {
    /// Connect to the database.
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    /// Build from an existing pool.
    #[must_use]
    pub const fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Acquire up to `limit` READY cards, transition them to DOING and insert a
    /// running attempt with a lease, all in one transaction.
    #[allow(clippy::too_many_lines)]
    pub async fn acquire_batch(
        &self,
        now: DateTime<Utc>,
        limit: usize,
    ) -> Result<Vec<ScheduledRun>, sqlx::Error> {
        let limit = i64::try_from(limit).unwrap_or(1);
        let mut tx = self.pool.begin().await?;
        let mut scheduled = Vec::new();
        // Claim explicitly queued runs first (API `POST /cards/{id}/runs`).
        let queued = sqlx::query(
            "SELECT r.id, r.card_id, r.agent_id, r.attempt, r.lab_id FROM runs r \
             JOIN cards c ON c.id = r.card_id JOIN agents a ON a.id = r.agent_id \
             JOIN projects p ON p.id = c.project_id JOIN labs l ON l.id = p.lab_id \
             WHERE r.status = 'queued' AND r.created_at <= $1 AND r.attempt <= 4 \
               AND c.blocked_reason IS NULL AND c.spent_microusd < c.budget_microusd \
               AND (SELECT COALESCE(SUM(cl.cost_microusd), 0) FROM cost_ledger cl WHERE cl.lab_id = l.id) < l.monthly_budget_microusd \
               AND (SELECT COUNT(*) FROM runs r2 WHERE r2.agent_id = a.id AND r2.status = 'running') < a.max_concurrent_cards \
               AND COALESCE((l.settings->>'paused')::boolean, false) = false \
               AND a.status IN ('idle','working') AND a.retired_at IS NULL \
             ORDER BY r.created_at FOR UPDATE OF r SKIP LOCKED LIMIT $2",
        )
        .bind(now)
        .bind(limit)
        .fetch_all(&mut *tx)
        .await?;
        for run in queued {
            let run_id: Uuid = run.get("id");
            let card_id: Uuid = run.get("card_id");
            let agent_id: Uuid = run.get("agent_id");
            let attempt: i16 = run.get("attempt");
            let lab_id: Uuid = run.get("lab_id");
            sqlx::query(
                "UPDATE runs SET status = 'running', lease_id = $2, started_at = $3, heartbeat_at = $3 WHERE id = $1",
            )
            .bind(run_id)
            .bind(Uuid::new_v4())
            .bind(now)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "UPDATE cards SET column_name = 'DOING', version = version + 1 WHERE id = $1",
            )
            .bind(card_id)
            .execute(&mut *tx)
            .await?;
            scheduled.push(ScheduledRun {
                run_id: RunId::new(run_id).map_err(|_| sqlx::Error::RowNotFound)?,
                card_id: CardId::new(card_id).map_err(|_| sqlx::Error::RowNotFound)?,
                agent_id: AgentId::new(agent_id).map_err(|_| sqlx::Error::RowNotFound)?,
                attempt: u8::try_from(attempt).unwrap_or(1),
                lab_id: LabId::new(lab_id).map_err(|_| sqlx::Error::RowNotFound)?,
            });
        }
        let remaining = limit.saturating_sub(i64::try_from(scheduled.len()).unwrap_or(0));
        if remaining <= 0 {
            tx.commit().await?;
            return Ok(scheduled);
        }
        let cards = sqlx::query(
            "SELECT c.id, c.project_id, c.assignee_id FROM cards c \
             JOIN agents a ON a.id = c.assignee_id \
             JOIN projects p ON p.id = c.project_id JOIN labs l ON l.id = p.lab_id \
             WHERE c.column_name = 'READY' AND c.assignee_id IS NOT NULL \
               AND c.blocked_reason IS NULL AND c.priority <> 'wont' \
               AND c.spent_microusd < c.budget_microusd \
               AND (SELECT COALESCE(SUM(cl.cost_microusd), 0) FROM cost_ledger cl WHERE cl.lab_id = l.id) < l.monthly_budget_microusd \
               AND (SELECT COUNT(*) FROM runs r2 WHERE r2.agent_id = a.id AND r2.status = 'running') < a.max_concurrent_cards \
               AND COALESCE((l.settings->>'paused')::boolean, false) = false \
               AND a.status IN ('idle','working') AND a.retired_at IS NULL \
               AND COALESCE((SELECT MAX(r.attempt) FROM runs r WHERE r.card_id = c.id), 0) <= 4 \
             ORDER BY c.due_date NULLS LAST FOR UPDATE OF c SKIP LOCKED LIMIT $1",
        )
        .bind(remaining)
        .fetch_all(&mut *tx)
        .await?;
        for card in cards {
            let card_id: Uuid = card.get("id");
            let project_id: Uuid = card.get("project_id");
            let agent_id: Uuid = card.get("assignee_id");
            let lab_id: Uuid = sqlx::query_scalar("SELECT lab_id FROM projects WHERE id = $1")
                .bind(project_id)
                .fetch_one(&mut *tx)
                .await?;
            let attempt: i16 = sqlx::query_scalar(
                "SELECT COALESCE(MAX(attempt), 0) + 1 FROM runs WHERE card_id = $1",
            )
            .bind(card_id)
            .fetch_one(&mut *tx)
            .await?;
            let run_id = Uuid::now_v7();
            sqlx::query(
                "INSERT INTO runs (id, card_id, project_id, lab_id, agent_id, attempt, status, lease_id, started_at, heartbeat_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, 'running', $7, $8, $8)",
            )
            .bind(run_id)
            .bind(card_id)
            .bind(project_id)
            .bind(lab_id)
            .bind(agent_id)
            .bind(attempt)
            .bind(Uuid::new_v4())
            .bind(now)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "UPDATE cards SET column_name = 'DOING', version = version + 1 WHERE id = $1",
            )
            .bind(card_id)
            .execute(&mut *tx)
            .await?;
            scheduled.push(ScheduledRun {
                run_id: RunId::new(run_id).map_err(|_| sqlx::Error::RowNotFound)?,
                card_id: CardId::new(card_id).map_err(|_| sqlx::Error::RowNotFound)?,
                agent_id: AgentId::new(agent_id).map_err(|_| sqlx::Error::RowNotFound)?,
                attempt: u8::try_from(attempt).unwrap_or(1),
                lab_id: LabId::new(lab_id).map_err(|_| sqlx::Error::RowNotFound)?,
            });
        }
        tx.commit().await?;
        Ok(scheduled)
    }

    /// Refresh the heartbeat of a running run.
    pub async fn heartbeat(&self, run_id: RunId, now: DateTime<Utc>) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE runs SET heartbeat_at = $2 WHERE id = $1 AND status = 'running'")
            .bind(run_id.as_uuid())
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Mark a run finished and clear its lease.
    pub async fn finish(
        &self,
        run_id: RunId,
        status: &str,
        now: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE runs SET status = $2, ended_at = $3, lease_id = NULL WHERE id = $1")
            .bind(run_id.as_uuid())
            .bind(status)
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Complete a successful run and move its card to DONE with the merge SHA.
    pub async fn complete_success(
        &self,
        run_id: RunId,
        merge_sha: Option<&str>,
        trace_id: Option<&str>,
        now: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "UPDATE runs SET status = 'succeeded', ended_at = $2, lease_id = NULL, \
             merge_sha = $3, trace_id = $4 WHERE id = $1",
        )
        .bind(run_id.as_uuid())
        .bind(now)
        .bind(merge_sha)
        .bind(trace_id)
        .execute(&mut *tx)
        .await?;
        if let Some(merge_sha) = merge_sha {
            sqlx::query(
                "UPDATE cards SET column_name = 'DONE', merge_sha = $2, version = version + 1                  WHERE id = (SELECT card_id FROM runs WHERE id = $1)                    AND column_name IN ('REVIEW','DOING')",
            )
            .bind(run_id.as_uuid())
            .bind(merge_sha)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Persist the pipeline steps for a run.
    pub async fn persist_steps(&self, run_id: RunId, steps: &[RunStep]) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        for step in steps {
            let step_type = match step.kind.as_str() {
                "shell.exec" => "tool_call",
                "verification" => "verification",
                "commit" => "commit",
                "push" => "push",
                "pr" => "pr",
                "review" | "merge" => "review",
                _ => "command",
            };
            sqlx::query(
                "INSERT INTO run_steps (id, run_id, seq, step_type, summary, detail, duration_ms)                  VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (run_id, seq) DO NOTHING",
            )
            .bind(Uuid::new_v4())
            .bind(run_id.as_uuid())
            .bind(i32::try_from(step.seq).unwrap_or(0))
            .bind(step_type)
            .bind(format!(
                "{:?} {} exit {:?}",
                step.stage, step.kind, step.exit_code
            ))
            .bind(json!({
                "args_hash": step.args_hash,
                "output": step.output,
                "trace_id": step.trace_id,
                "exit_code": step.exit_code
            }))
            .bind(i32::try_from(step.duration_ms).unwrap_or(i32::MAX))
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Persist the run trace id.
    pub async fn set_trace(&self, run_id: RunId, trace_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE runs SET trace_id = $2 WHERE id = $1")
            .bind(run_id.as_uuid())
            .bind(trace_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Persist a typed error code on a run.
    pub async fn set_error(&self, run_id: RunId, code: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE runs SET error_code = $2 WHERE id = $1")
            .bind(run_id.as_uuid())
            .bind(code)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Pause a lab (persisted in `labs.settings`).
    pub async fn pause_lab(&self, lab_id: LabId) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE labs SET settings = jsonb_set(settings, '{paused}', 'true'::jsonb, true) \
             WHERE id = $1",
        )
        .bind(lab_id.as_uuid())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Dead-letter a run after the retry limit and move its card to ICEBOX.
    pub async fn dead_letter(&self, run_id: RunId, card_id: CardId) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "UPDATE runs SET status = 'failed', lease_id = NULL, ended_at = now() WHERE id = $1",
        )
        .bind(run_id.as_uuid())
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE cards SET column_name = 'ICEBOX', \
             blocked_reason = jsonb_build_object('category','risk','text','retry limit reached') \
             WHERE id = $1",
        )
        .bind(card_id.as_uuid())
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Requeue a retryable run and return its card to READY.
    pub async fn requeue(
        &self,
        run_id: RunId,
        card_id: CardId,
        available_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "UPDATE runs SET status = 'queued', lease_id = NULL, heartbeat_at = NULL,              attempt = attempt + 1, created_at = $2 WHERE id = $1",
        )
        .bind(run_id.as_uuid())
        .bind(available_at)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE cards SET column_name = 'DOING' WHERE id = $1")
            .bind(card_id.as_uuid())
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Reap running runs whose heartbeat is older than 45 s and return their cards.
    pub async fn reap(&self, now: DateTime<Utc>) -> Result<Vec<CardId>, sqlx::Error> {
        let threshold = now - Duration::seconds(45);
        let rows = sqlx::query(
            "UPDATE runs SET status = 'timed_out', ended_at = $1, lease_id = NULL \
             WHERE status = 'running' AND heartbeat_at < $2 \
             RETURNING id, card_id, lab_id",
        )
        .bind(now)
        .bind(threshold)
        .fetch_all(&self.pool)
        .await?;
        let mut cards = Vec::new();
        let mut ids = Vec::new();
        for row in rows {
            let card_id: Uuid = row.get("card_id");
            let run_id: Uuid = row.get("id");
            let lab_id: Uuid = row.get("lab_id");
            if let Ok(card) = CardId::new(card_id) {
                cards.push(card);
                ids.push(card_id);
                if let (Ok(run), Ok(lab)) = (RunId::new(run_id), LabId::new(lab_id)) {
                    let _ = self
                        .emit_event_raw("delivery.run.timed_out", run, lab, card, now)
                        .await;
                }
            }
        }
        if !ids.is_empty() {
            sqlx::query("UPDATE cards SET column_name = 'READY' WHERE id = ANY($1)")
                .bind(&ids)
                .execute(&self.pool)
                .await?;
        }
        Ok(cards)
    }

    /// Insert a `delivery.run.*` event into the outbox for the relay.
    pub async fn emit_event(
        &self,
        event_type: &str,
        run: &ScheduledRun,
        now: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO outbox (id, aggregate_type, aggregate_id, event_type, payload, available_at) \
             VALUES ($1, 'run', $2, $3, jsonb_build_object('run_id', $2::text, 'lab_id', $4::text, 'card_id', $5::text), $6)",
        )
        .bind(Uuid::new_v4())
        .bind(run.run_id.as_uuid())
        .bind(event_type)
        .bind(run.lab_id.as_uuid())
        .bind(run.card_id.as_uuid())
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Emit an event without a full scheduled-run record.
    pub async fn emit_event_raw(
        &self,
        event_type: &str,
        run_id: RunId,
        lab_id: LabId,
        card_id: CardId,
        now: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO outbox (id, aggregate_type, aggregate_id, event_type, payload, available_at) \
             VALUES ($1, 'run', $2, $3, jsonb_build_object('run_id', $2::text, 'lab_id', $4::text, 'card_id', $5::text), $6)",
        )
        .bind(Uuid::new_v4())
        .bind(run_id.as_uuid())
        .bind(event_type)
        .bind(lab_id.as_uuid())
        .bind(card_id.as_uuid())
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Latest run for a card and its merge SHA.
    pub async fn latest_run(
        &self,
        card_id: CardId,
    ) -> Result<Option<(RunId, Option<String>, LabId, i16)>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, merge_sha, lab_id, attempt FROM runs WHERE card_id = $1 \
             ORDER BY attempt DESC LIMIT 1",
        )
        .bind(card_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else { return Ok(None) };
        let id: Uuid = row.get("id");
        let merge_sha: Option<String> = row.get("merge_sha");
        let lab_id: Uuid = row.get("lab_id");
        let attempt: i16 = row.get("attempt");
        Ok(Some((
            RunId::new(id).map_err(|_| sqlx::Error::RowNotFound)?,
            merge_sha,
            LabId::new(lab_id).map_err(|_| sqlx::Error::RowNotFound)?,
            attempt,
        )))
    }

    /// Relay unpublished outbox rows into `domain_events` with a per-lab `seq`.
    pub async fn relay_outbox(&self, limit: usize) -> Result<usize, sqlx::Error> {
        let limit = i64::try_from(limit).unwrap_or(50);
        let mut tx = self.pool.begin().await?;
        let rows = sqlx::query(
            "SELECT id, aggregate_type, aggregate_id, event_type, payload FROM outbox \
             WHERE published_at IS NULL ORDER BY created_at FOR UPDATE SKIP LOCKED LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&mut *tx)
        .await?;
        let mut published = 0;
        for row in &rows {
            let id: Uuid = row.get("id");
            let aggregate_type: String = row.get("aggregate_type");
            let aggregate_id: Uuid = row.get("aggregate_id");
            let event_type: String = row.get("event_type");
            let payload: serde_json::Value = row.get("payload");
            let Some(lab_id) = payload["lab_id"]
                .as_str()
                .and_then(|value| Uuid::parse_str(value).ok())
            else {
                continue;
            };
            sqlx::query("SELECT id FROM labs WHERE id = $1 FOR UPDATE")
                .bind(lab_id)
                .fetch_optional(&mut *tx)
                .await?;
            let seq: i64 = sqlx::query_scalar(
                "SELECT COALESCE(MAX(seq), 0) + 1 FROM domain_events WHERE lab_id = $1",
            )
            .bind(lab_id)
            .fetch_one(&mut *tx)
            .await?;
            sqlx::query(
                "INSERT INTO domain_events (id, lab_id, seq, event_type, aggregate_type, aggregate_id, actor_type, payload) \
                 VALUES ($1, $2, $3, $4, $5, $6, 'system', $7)",
            )
            .bind(id)
            .bind(lab_id)
            .bind(seq)
            .bind(&event_type)
            .bind(&aggregate_type)
            .bind(aggregate_id)
            .bind(&payload)
            .execute(&mut *tx)
            .await?;
            sqlx::query("UPDATE outbox SET published_at = now() WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            published += 1;
        }
        tx.commit().await?;
        Ok(published)
    }

    /// Return the pool for the run store.
    #[must_use]
    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// Postgres-backed worker tick report.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PostgresTickReport {
    /// Runs reaped after a stale heartbeat.
    pub reaped: usize,
    /// Runs acquired and executed.
    pub completed: usize,
    /// Runs finished as failed.
    pub failed: usize,
    /// Runs cancelled at a tool boundary.
    pub cancelled: usize,
}

/// Cancellation probe backed by the `runs` table.
#[derive(Clone)]
pub struct PostgresCancelProbe {
    pool: PgPool,
}

impl PostgresCancelProbe {
    /// Build the probe.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CancelProbe for PostgresCancelProbe {
    async fn is_cancelled(&self, run_id: RunId) -> bool {
        sqlx::query_scalar::<_, String>("SELECT status FROM runs WHERE id = $1")
            .bind(run_id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .ok()
            .flatten()
            .is_some_and(|status| status == "cancelled")
    }
}

/// Worker that leases and dispatches durable runs.
pub struct PostgresWorker {
    scheduler: PostgresScheduler,
    executor: std::sync::Arc<dyn crate::worker::RunExecutor>,
    concurrency: usize,
}

impl PostgresWorker {
    /// Build the worker.
    pub fn new(
        scheduler: PostgresScheduler,
        executor: std::sync::Arc<dyn crate::worker::RunExecutor>,
        concurrency: usize,
    ) -> Self {
        Self {
            scheduler,
            executor,
            concurrency: concurrency.max(1),
        }
    }

    async fn persist_failed(&self, run: &ScheduledRun) -> Result<(), sqlx::Error> {
        if let Some(pipeline) = self.executor.last_pipeline() {
            self.scheduler
                .persist_steps(run.run_id, &pipeline.steps)
                .await?;
            self.scheduler
                .set_trace(run.run_id, &pipeline.trace_id)
                .await?;
        }
        Ok(())
    }

    /// Run one tick: reap stale runs, lease up to concurrency runs and execute.
    #[allow(clippy::too_many_lines)]
    pub async fn tick(&self, now: DateTime<Utc>) -> Result<PostgresTickReport, sqlx::Error> {
        let reaped_cards = self.scheduler.reap(now).await?;
        for card in &reaped_cards {
            if let Some((run_id, merge_sha, _lab_id, attempt)) =
                self.scheduler.latest_run(*card).await?
            {
                let action = self.executor.recover(*card, run_id).await;
                let recovered_sha = action.as_ref().and_then(|(action, sha)| {
                    (*action == crate::pipeline::RecoveryAction::Done)
                        .then(|| sha.clone())
                        .flatten()
                });
                let final_sha = merge_sha.clone().or(recovered_sha);
                if final_sha.is_some() {
                    self.scheduler
                        .complete_success(run_id, final_sha.as_deref(), None, now)
                        .await?;
                } else {
                    let backoff = match attempt {
                        1 => 30,
                        2 => 120,
                        _ => 600,
                    };
                    self.scheduler
                        .requeue(run_id, *card, now + Duration::seconds(backoff))
                        .await?;
                }
            }
        }
        let reaped = reaped_cards.len();
        let runs = self.scheduler.acquire_batch(now, self.concurrency).await?;
        let mut report = PostgresTickReport {
            reaped,
            ..PostgresTickReport::default()
        };
        for run in runs {
            self.scheduler
                .emit_event("delivery.run.started", &run, now)
                .await?;
            let queued = crate::worker::QueuedRun::new(run.run_id, run.card_id, run.agent_id, now);
            let heartbeat = {
                let scheduler = self.scheduler.clone();
                let run_id = run.run_id;
                tokio::spawn(async move {
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
                        if scheduler.heartbeat(run_id, Utc::now()).await.is_err() {
                            break;
                        }
                    }
                })
            };
            let result = self.executor.execute(&queued).await;
            heartbeat.abort();
            match result {
                Ok(pipeline) => {
                    self.scheduler
                        .persist_steps(run.run_id, &pipeline.steps)
                        .await?;
                    self.scheduler
                        .emit_event_raw(
                            "delivery.run.step",
                            run.run_id,
                            run.lab_id,
                            run.card_id,
                            now,
                        )
                        .await?;
                    let merge_sha = pipeline
                        .steps
                        .iter()
                        .rev()
                        .find(|step| step.kind == "merge")
                        .map(|step| step.output.clone());
                    self.scheduler
                        .complete_success(
                            run.run_id,
                            merge_sha.as_deref(),
                            Some(&pipeline.trace_id),
                            now,
                        )
                        .await?;
                    for step in &pipeline.steps {
                        let event = match step.kind.as_str() {
                            "pr" => "delivery.pr.opened",
                            "review" => "delivery.review.submitted",
                            "merge" => "delivery.merge.completed",
                            _ => continue,
                        };
                        self.scheduler
                            .emit_event_raw(event, run.run_id, run.lab_id, run.card_id, now)
                            .await?;
                    }
                    self.scheduler
                        .emit_event("delivery.run.completed", &run, now)
                        .await?;
                    report.completed += 1;
                }
                Err(crate::worker::ExecError::Cancelled) => {
                    self.persist_failed(&run).await?;
                    self.scheduler.finish(run.run_id, "cancelled", now).await?;
                    self.scheduler
                        .emit_event("delivery.run.cancelled", &run, now)
                        .await?;
                    report.cancelled += 1;
                }
                Err(crate::worker::ExecError::Budget) => {
                    self.persist_failed(&run).await?;
                    self.scheduler
                        .set_error(run.run_id, "BUDGET_EXCEEDED")
                        .await?;
                    self.scheduler.finish(run.run_id, "failed", now).await?;
                    self.scheduler.pause_lab(run.lab_id).await?;
                    self.scheduler
                        .emit_event("system.budget.exhausted", &run, now)
                        .await?;
                    report.failed += 1;
                }
                Err(error) if error.retryable() => {
                    self.persist_failed(&run).await?;
                    if run.attempt < 4 {
                        let backoff = match run.attempt {
                            1 => 30,
                            2 => 120,
                            _ => 600,
                        };
                        self.scheduler
                            .requeue(run.run_id, run.card_id, now + Duration::seconds(backoff))
                            .await?;
                    } else {
                        self.scheduler.dead_letter(run.run_id, run.card_id).await?;
                    }
                    self.scheduler
                        .emit_event("delivery.run.failed", &run, now)
                        .await?;
                    report.failed += 1;
                }
                Err(_) => {
                    self.persist_failed(&run).await?;
                    self.scheduler.finish(run.run_id, "failed", now).await?;
                    self.scheduler
                        .emit_event("delivery.run.failed", &run, now)
                        .await?;
                    report.failed += 1;
                }
            }
        }
        let _ = self.scheduler.relay_outbox(50).await;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduled_run_is_domain_typed() {
        let run = ScheduledRun {
            run_id: RunId::generate(),
            card_id: CardId::generate(),
            agent_id: AgentId::generate(),
            attempt: 1,
            lab_id: LabId::generate(),
        };
        assert_eq!(run.attempt, 1);
    }

    /// Live Postgres test; run against the migrated database with
    /// `AILAB_TEST_DATABASE_URL` and `--ignored`.
    #[tokio::test]
    #[ignore = "requires a migrated PostgreSQL database"]
    async fn acquire_is_exclusive_and_reap_times_out() {
        let url = std::env::var("AILAB_TEST_DATABASE_URL").expect("AILAB_TEST_DATABASE_URL");
        let scheduler = PostgresScheduler::connect(&url).await.expect("connect");
        let now = Utc::now();
        let first = scheduler.acquire_batch(now, 1).await.expect("acquire");
        let second = scheduler.acquire_batch(now, 1).await.expect("acquire");
        if let Some(run) = first.first() {
            assert!(!second.iter().any(|other| other.card_id == run.card_id));
            scheduler
                .heartbeat(run.run_id, now)
                .await
                .expect("heartbeat");
            let cards = scheduler
                .reap(now + Duration::seconds(46))
                .await
                .expect("reap");
            assert!(cards.contains(&run.card_id));
        }
    }
}
