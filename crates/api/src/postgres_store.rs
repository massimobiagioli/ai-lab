//! Postgres-backed run store for the edge API.

use ailab_domain::{EventId, LabId, RunId};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::EventEnvelope;
use crate::Page;
use crate::server::{RunStore, RunView, StepView, StoreError};

/// Postgres run store.
#[derive(Debug, Clone)]
pub struct PostgresRunStore {
    pool: PgPool,
}

impl PostgresRunStore {
    /// Build from an existing pool.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn run_from_row(row: &sqlx::postgres::PgRow) -> Option<RunView> {
        Some(RunView {
            id: row.try_get::<Uuid, _>("id").ok()?.to_string(),
            card_id: row.try_get::<Uuid, _>("card_id").ok()?.to_string(),
            lab_id: row.try_get::<Uuid, _>("lab_id").ok()?.to_string(),
            state: row.try_get::<String, _>("status").ok()?,
            attempt: u8::try_from(row.try_get::<i16, _>("attempt").ok()?).ok()?,
            started_at: row.try_get::<Option<DateTime<Utc>>, _>("started_at").ok()?,
            ended_at: row.try_get::<Option<DateTime<Utc>>, _>("ended_at").ok()?,
            cost_microusd: row.try_get::<i64, _>("cost_microusd").ok()?,
        })
    }

    async fn fetch_run(&self, id: Uuid) -> Option<RunView> {
        let row = sqlx::query(
            "SELECT id, card_id, lab_id, status, attempt, started_at, ended_at, cost_microusd \
             FROM runs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .ok()??;
        Self::run_from_row(&row)
    }
}

#[async_trait]
impl RunStore for PostgresRunStore {
    async fn ready(&self) -> bool {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .is_ok()
    }

    async fn get(&self, id: &str) -> Option<RunView> {
        let uuid = Uuid::parse_str(id).ok()?;
        self.fetch_run(uuid).await
    }

    async fn steps(&self, id: &str) -> Option<Vec<StepView>> {
        let uuid = Uuid::parse_str(id).ok()?;
        let rows = sqlx::query(
            "SELECT seq, step_type, summary, detail, duration_ms \
             FROM run_steps WHERE run_id = $1 ORDER BY seq",
        )
        .bind(uuid)
        .fetch_all(&self.pool)
        .await
        .ok()?;
        let mut steps = Vec::new();
        for row in rows {
            let seq = u32::try_from(row.try_get::<i32, _>("seq").ok()?).ok()?;
            let detail: serde_json::Value = row.try_get("detail").unwrap_or_default();
            steps.push(StepView {
                seq,
                stage: row.try_get::<String, _>("step_type").ok()?,
                kind: row.try_get::<String, _>("summary").ok()?,
                exit_code: detail["exit_code"]
                    .as_i64()
                    .and_then(|code| i32::try_from(code).ok()),
                output: detail["output"]
                    .as_str()
                    .map(ToOwned::to_owned)
                    .unwrap_or_default(),
            });
        }
        Some(steps)
    }

    async fn cancel(&self, id: &str) -> Result<RunView, StoreError> {
        let uuid = Uuid::parse_str(id).map_err(|_| StoreError::NotFound)?;
        let row = sqlx::query(
            "UPDATE runs SET status = 'cancelled', ended_at = now() \
             WHERE id = $1 AND status IN ('queued','running') \
             RETURNING id, card_id, lab_id, status, attempt, started_at, ended_at, cost_microusd",
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StoreError::Conflict)?;
        if let Some(row) = row {
            Self::run_from_row(&row).ok_or(StoreError::Conflict)
        } else if self.fetch_run(uuid).await.is_some() {
            Err(StoreError::Conflict)
        } else {
            Err(StoreError::NotFound)
        }
    }

    async fn list_by_lab(&self, lab_id: &str, limit: u32, offset: usize) -> Page<RunView> {
        let Ok(lab) = Uuid::parse_str(lab_id) else {
            return Page {
                data: Vec::new(),
                next_cursor: None,
                total_estimate: 0,
            };
        };
        let limit = i64::from(limit);
        let offset = i64::try_from(offset).unwrap_or(0);
        let rows = sqlx::query(
            "SELECT id, card_id, lab_id, status, attempt, started_at, ended_at, cost_microusd \
             FROM runs WHERE lab_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(lab)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();
        let total: i64 = sqlx::query_scalar("SELECT count(*) FROM runs WHERE lab_id = $1")
            .bind(lab)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        let data: Vec<RunView> = rows.iter().filter_map(Self::run_from_row).collect();
        let next = usize::try_from(offset).unwrap_or(0) + data.len();
        Page {
            data,
            next_cursor: (next < usize::try_from(total).unwrap_or(0)).then(|| next.to_string()),
            total_estimate: u64::try_from(total).unwrap_or(0),
        }
    }

    async fn enqueue_card(&self, card_id: &str) -> Result<RunView, StoreError> {
        let card = Uuid::parse_str(card_id).map_err(|_| StoreError::NotFound)?;
        let row = sqlx::query(
            "SELECT c.project_id, p.lab_id, c.assignee_id FROM cards c \
             JOIN projects p ON p.id = c.project_id WHERE c.id = $1",
        )
        .bind(card)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StoreError::Conflict)?
        .ok_or(StoreError::NotFound)?;
        let project_id: Uuid = row
            .try_get("project_id")
            .map_err(|_| StoreError::Conflict)?;
        let lab_id: Uuid = row.try_get("lab_id").map_err(|_| StoreError::Conflict)?;
        let agent_id: Option<Uuid> = row
            .try_get("assignee_id")
            .map_err(|_| StoreError::Conflict)?;
        let agent_id = agent_id.ok_or(StoreError::Conflict)?;
        let attempt: i16 =
            sqlx::query_scalar("SELECT COALESCE(MAX(attempt), 0) + 1 FROM runs WHERE card_id = $1")
                .bind(card)
                .fetch_one(&self.pool)
                .await
                .map_err(|_| StoreError::Conflict)?;
        let run_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO runs (id, card_id, project_id, lab_id, agent_id, attempt, status) \
             VALUES ($1, $2, $3, $4, $5, $6, 'queued')",
        )
        .bind(run_id)
        .bind(card)
        .bind(project_id)
        .bind(lab_id)
        .bind(agent_id)
        .bind(attempt)
        .execute(&self.pool)
        .await
        .map_err(|_| StoreError::Conflict)?;
        self.fetch_run(run_id).await.ok_or(StoreError::Conflict)
    }

    async fn events_after(&self, lab_id: &str, since: i64) -> Vec<EventEnvelope> {
        let Ok(lab) = Uuid::parse_str(lab_id) else {
            return Vec::new();
        };
        let rows = sqlx::query(
            "SELECT id, event_type, seq, lab_id, occurred_at, payload \
             FROM domain_events WHERE lab_id = $1 AND seq > $2 ORDER BY seq LIMIT 200",
        )
        .bind(lab)
        .bind(since)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();
        let mut events = Vec::new();
        for row in rows {
            let Ok(id) = row.try_get::<Uuid, _>("id") else {
                continue;
            };
            let Ok(lab) = row.try_get::<Uuid, _>("lab_id") else {
                continue;
            };
            let Ok(event_id) = EventId::new(id) else {
                continue;
            };
            let Ok(lab_id) = LabId::new(lab) else {
                continue;
            };
            events.push(EventEnvelope {
                event_id,
                event: row.try_get::<String, _>("event_type").unwrap_or_default(),
                seq: row.try_get::<i64, _>("seq").unwrap_or(0),
                lab_id,
                occurred_at: row
                    .try_get::<DateTime<Utc>, _>("occurred_at")
                    .unwrap_or_else(|_| Utc::now()),
                payload: row.try_get("payload").unwrap_or_default(),
            });
        }
        events
    }

    async fn append_event(&self, _event: EventEnvelope) {
        // Events are persisted by the transactional outbox; the API reads them.
    }
}

/// Validate that a run id parses.
#[must_use]
pub fn parse_run_id(value: &str) -> Option<RunId> {
    Uuid::parse_str(value)
        .ok()
        .and_then(|uuid| RunId::new(uuid).ok())
}
