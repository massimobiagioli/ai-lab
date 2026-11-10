//! Postgres-backed run store for the edge API.

use std::collections::BTreeMap;
use std::sync::Mutex;

use ailab_domain::{EventId, LabId, RunId};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use ailab_application::{EventEnvelope, Page, RunCursor, RunStore, RunView, StepView, StoreError};

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
            created_at: row.try_get::<DateTime<Utc>, _>("created_at").ok()?,
        })
    }

    async fn fetch_run(&self, id: Uuid) -> Option<RunView> {
        let row = sqlx::query(
            "SELECT id, card_id, lab_id, status, attempt, started_at, ended_at, cost_microusd, created_at \
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

    async fn list_by_lab(
        &self,
        lab_id: &str,
        limit: u32,
        cursor: Option<RunCursor>,
    ) -> Page<RunView> {
        let Ok(lab) = Uuid::parse_str(lab_id) else {
            return Page {
                data: Vec::new(),
                next_cursor: None,
                total_estimate: 0,
            };
        };
        let limit = i64::from(limit);
        let cursor_ts = cursor.as_ref().map(|cursor| cursor.created_at);
        let cursor_id = cursor
            .as_ref()
            .and_then(|cursor| Uuid::parse_str(&cursor.id).ok());
        let rows = sqlx::query(
            "SELECT id, card_id, lab_id, status, attempt, started_at, ended_at, cost_microusd, created_at \
             FROM runs WHERE lab_id = $1 \
               AND ($2::timestamptz IS NULL OR (created_at, id) < ($2, $3::uuid)) \
             ORDER BY created_at DESC, id DESC LIMIT $4",
        )
        .bind(lab)
        .bind(cursor_ts)
        .bind(cursor_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();
        let total: i64 = sqlx::query_scalar("SELECT count(*) FROM runs WHERE lab_id = $1")
            .bind(lab)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        let data: Vec<RunView> = rows.iter().filter_map(Self::run_from_row).collect();
        let next_cursor = (data.len() == usize::try_from(limit).unwrap_or(usize::MAX))
            .then(|| {
                data.last()
                    .map(|run| format!("{}|{}", run.created_at.to_rfc3339(), run.id))
            })
            .flatten();
        Page {
            data,
            next_cursor,
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
        let _ = sqlx::query(
            "INSERT INTO outbox (id, aggregate_type, aggregate_id, event_type, payload) \
             VALUES ($1, 'run', $2, 'delivery.run.queued', \
               jsonb_build_object('run_id', $2::text, 'lab_id', $3::text, 'card_id', $4::text))",
        )
        .bind(Uuid::now_v7())
        .bind(run_id)
        .bind(lab_id)
        .bind(card)
        .execute(&self.pool)
        .await;
        self.fetch_run(run_id).await.ok_or(StoreError::Conflict)
    }

    async fn events_after(&self, lab_id: &str, since: i64) -> Vec<EventEnvelope> {
        let Ok(lab) = Uuid::parse_str(lab_id) else {
            return Vec::new();
        };
        let rows = sqlx::query(
            "SELECT id, event_type, seq, lab_id, occurred_at, payload \
             FROM domain_events WHERE lab_id = $1 AND seq > $2 ORDER BY seq LIMIT 1000",
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

#[derive(Default)]
struct StoreData {
    runs: BTreeMap<String, RunView>,
    steps: BTreeMap<String, Vec<StepView>>,
    events: Vec<EventEnvelope>,
}

/// In-memory store used by tests/dev.
#[derive(Default)]
pub struct InMemoryRunStore {
    data: Mutex<StoreData>,
}

impl InMemoryRunStore {
    /// New empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed a run (used by tests and the demo).
    pub fn insert(&self, run: RunView, steps: Vec<StepView>) {
        if let Ok(mut data) = self.data.lock() {
            data.steps.insert(run.id.clone(), steps);
            data.runs.insert(run.id.clone(), run);
        }
    }
}

#[async_trait]
impl RunStore for InMemoryRunStore {
    async fn get(&self, id: &str) -> Option<RunView> {
        self.data.lock().ok()?.runs.get(id).cloned()
    }

    async fn steps(&self, id: &str) -> Option<Vec<StepView>> {
        self.data.lock().ok()?.steps.get(id).cloned()
    }

    async fn cancel(&self, id: &str) -> Result<RunView, StoreError> {
        let mut data = self.data.lock().map_err(|_| StoreError::Conflict)?;
        let run = data.runs.get_mut(id).ok_or(StoreError::NotFound)?;
        if run.state != "queued" && run.state != "running" {
            return Err(StoreError::Conflict);
        }
        "cancelled".clone_into(&mut run.state);
        run.ended_at = Some(Utc::now());
        let view = run.clone();
        drop(data);
        Ok(view)
    }

    async fn list_by_lab(
        &self,
        lab_id: &str,
        limit: u32,
        cursor: Option<RunCursor>,
    ) -> Page<RunView> {
        let Some(data) = self.data.lock().ok() else {
            return Page {
                data: Vec::new(),
                next_cursor: None,
                total_estimate: 0,
            };
        };
        let mut all: Vec<RunView> = data
            .runs
            .values()
            .filter(|run| run.lab_id == lab_id)
            .cloned()
            .collect();
        all.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.id.cmp(&a.id))
        });
        if let Some(cursor) = &cursor {
            all.retain(|run| {
                run.created_at < cursor.created_at
                    || (run.created_at == cursor.created_at && run.id < cursor.id)
            });
        }
        let limit = usize::try_from(limit).unwrap_or(usize::MAX);
        let page: Vec<RunView> = all.iter().take(limit).cloned().collect();
        let next_cursor = (page.len() == limit && !page.is_empty()).then(|| {
            let last = page.last().expect("non-empty");
            format!("{}|{}", last.created_at.to_rfc3339(), last.id)
        });
        Page {
            data: page,
            next_cursor,
            total_estimate: u64::try_from(all.len()).unwrap_or(u64::MAX),
        }
    }

    async fn enqueue_card(&self, card_id: &str) -> Result<RunView, StoreError> {
        let mut data = self.data.lock().map_err(|_| StoreError::Conflict)?;
        let run = RunView {
            id: RunId::generate().as_uuid().to_string(),
            card_id: card_id.to_owned(),
            lab_id: "default".to_owned(),
            state: "queued".to_owned(),
            attempt: 1,
            started_at: None,
            ended_at: None,
            cost_microusd: 0,
            created_at: Utc::now(),
        };
        data.runs.insert(run.id.clone(), run.clone());
        data.steps.insert(run.id.clone(), Vec::new());
        drop(data);
        Ok(run)
    }

    async fn events_after(&self, lab_id: &str, since: i64) -> Vec<EventEnvelope> {
        let Some(data) = self.data.lock().ok() else {
            return Vec::new();
        };
        ailab_application::events_after(&data.events, since)
            .into_iter()
            .filter(|event| event.lab_id.to_string() == lab_id)
            .collect()
    }

    async fn append_event(&self, event: EventEnvelope) {
        if let Ok(mut data) = self.data.lock()
            && !data
                .events
                .iter()
                .any(|existing| existing.event_id == event.event_id)
        {
            data.events.push(event);
        }
    }
}
