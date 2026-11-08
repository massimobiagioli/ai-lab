//! Postgres-backed scheduler: durable `SKIP LOCKED` leases and heartbeat reaping.
//!
//! The lease is modeled on the `runs` row (`lease_id`, `heartbeat_at`) and the
//! card's `DOING` transition in the same transaction, so two workers cannot
//! lease the same card.

use ailab_domain::{AgentId, CardId, LabId, RunId};
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
            "SELECT id, card_id, agent_id, attempt FROM runs \
             WHERE status = 'queued' ORDER BY created_at FOR UPDATE SKIP LOCKED LIMIT $1",
        )
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
            "SELECT id, project_id, assignee_id FROM cards \
             WHERE column_name = 'READY' AND assignee_id IS NOT NULL \
             ORDER BY due_date NULLS LAST FOR UPDATE SKIP LOCKED LIMIT $1",
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

    /// Reap running runs whose heartbeat is older than 45 s and return their cards.
    pub async fn reap(&self, now: DateTime<Utc>) -> Result<Vec<CardId>, sqlx::Error> {
        let threshold = now - Duration::seconds(45);
        let rows = sqlx::query(
            "UPDATE runs SET status = 'timed_out', ended_at = $1, lease_id = NULL \
             WHERE status = 'running' AND heartbeat_at < $2 RETURNING card_id",
        )
        .bind(now)
        .bind(threshold)
        .fetch_all(&self.pool)
        .await?;
        let mut cards = Vec::new();
        for row in rows {
            let card_id: Uuid = row.get("card_id");
            if let Ok(card) = CardId::new(card_id) {
                cards.push(card);
            }
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

    /// Run one tick: reap stale runs, lease up to concurrency runs and execute.
    pub async fn tick(&self, now: DateTime<Utc>) -> Result<PostgresTickReport, sqlx::Error> {
        let reaped = self.scheduler.reap(now).await?.len();
        let runs = self.scheduler.acquire_batch(now, self.concurrency).await?;
        let mut report = PostgresTickReport {
            reaped,
            ..PostgresTickReport::default()
        };
        for run in runs {
            let queued = crate::worker::QueuedRun::new(run.run_id, run.card_id, run.agent_id, now);
            match self.executor.execute(&queued).await {
                Ok(_) => {
                    self.scheduler.finish(run.run_id, "succeeded", now).await?;
                    report.completed += 1;
                }
                Err(error) if error.retryable() => {
                    self.scheduler.finish(run.run_id, "failed", now).await?;
                    report.failed += 1;
                }
                Err(_) => {
                    self.scheduler.finish(run.run_id, "failed", now).await?;
                    report.failed += 1;
                }
            }
        }
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
