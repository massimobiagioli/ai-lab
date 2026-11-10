//! Delivery read models and run-store port.

use ailab_domain::{EventId, LabId};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Default page size.
pub const DEFAULT_LIMIT: u32 = 50;
/// Maximum page size.
pub const MAX_LIMIT: u32 = 200;

/// Cursor pagination query.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PageQuery {
    /// Requested limit.
    pub limit: Option<u32>,
    /// Opaque cursor.
    pub cursor: Option<String>,
}

impl PageQuery {
    /// Normalize the limit to `1..=200`, defaulting to 50.
    pub fn normalize(&self) -> Result<u32, String> {
        match self.limit {
            None => Ok(DEFAULT_LIMIT),
            Some(0) => Err("limit must be at least 1".to_owned()),
            Some(limit) if limit > MAX_LIMIT => Err(format!("limit must be at most {MAX_LIMIT}")),
            Some(limit) => Ok(limit),
        }
    }
}

/// A page of results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    /// Data.
    pub data: Vec<T>,
    /// Next cursor.
    pub next_cursor: Option<String>,
    /// Total estimate.
    pub total_estimate: u64,
}

/// Persisted event envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
pub struct EventEnvelope {
    /// Event id.
    pub event_id: EventId,
    /// Event name.
    pub event: String,
    /// Per-lab sequence.
    pub seq: i64,
    /// Lab id.
    pub lab_id: LabId,
    /// Occurred at.
    pub occurred_at: DateTime<Utc>,
    /// Payload.
    pub payload: serde_json::Value,
}

/// Return events strictly after `since`, in order, with duplicate ids removed.
#[must_use]
pub fn events_after(events: &[EventEnvelope], since: i64) -> Vec<EventEnvelope> {
    let mut seen = std::collections::HashSet::new();
    let mut output: Vec<EventEnvelope> = events
        .iter()
        .filter(|event| event.seq > since && seen.insert(event.event_id))
        .cloned()
        .collect();
    output.sort_by_key(|event| event.seq);
    output
}

/// Run summary.
#[derive(Debug, Clone, Serialize)]
pub struct RunView {
    /// Run id.
    pub id: String,
    /// Card id.
    pub card_id: String,
    /// Lab id.
    pub lab_id: String,
    /// State.
    pub state: String,
    /// Attempt.
    pub attempt: u8,
    /// Started at.
    pub started_at: Option<DateTime<Utc>>,
    /// Ended at.
    pub ended_at: Option<DateTime<Utc>>,
    /// Cost so far.
    pub cost_microusd: i64,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Run step.
#[derive(Debug, Clone, Serialize)]
pub struct StepView {
    /// Sequence.
    pub seq: u32,
    /// Stage.
    pub stage: String,
    /// Kind.
    pub kind: String,
    /// Exit code.
    pub exit_code: Option<i32>,
    /// Output.
    pub output: String,
}

/// Store failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreError {
    /// Not found.
    NotFound,
    /// Invalid state for the operation.
    Conflict,
}

/// Run keyset cursor.
#[derive(Debug, Clone)]
pub struct RunCursor {
    /// Creation timestamp of the last row.
    pub created_at: DateTime<Utc>,
    /// Run id of the last row.
    pub id: String,
}

/// Run store port (in-memory or Postgres).
#[async_trait]
pub trait RunStore: Send + Sync {
    /// Readiness probe (dependency check).
    async fn ready(&self) -> bool {
        true
    }
    /// Read a run.
    async fn get(&self, id: &str) -> Option<RunView>;
    /// Read the ordered steps.
    async fn steps(&self, id: &str) -> Option<Vec<StepView>>;
    /// Cancel a queued/running run.
    async fn cancel(&self, id: &str) -> Result<RunView, StoreError>;
    /// List runs for a lab.
    async fn list_by_lab(
        &self,
        lab_id: &str,
        limit: u32,
        cursor: Option<RunCursor>,
    ) -> Page<RunView>;
    /// Enqueue a run for a card.
    async fn enqueue_card(&self, card_id: &str) -> Result<RunView, StoreError>;
    /// Events after a sequence.
    async fn events_after(&self, lab_id: &str, since: i64) -> Vec<EventEnvelope>;
    /// Append an event to the backfill buffer.
    async fn append_event(&self, event: EventEnvelope);
}
