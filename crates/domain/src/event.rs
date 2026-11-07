//! Domain event envelope and the core event catalog.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;
use crate::ids::{EventId, LabId};
use crate::value::Actor;

/// Core event catalog: `(name, schema_version)`.
pub const EVENT_CATALOG: &[(&str, u32)] = &[
    ("inventory.provider.created", 1),
    ("inventory.provider.updated", 1),
    ("inventory.agent.created", 1),
    ("inventory.agent.cv_updated", 1),
    ("inventory.project.connected", 1),
    ("lab.agent.status_changed", 1),
    ("lab.budget.changed", 1),
    ("kanban.card.created", 1),
    ("kanban.card.moved", 1),
    ("kanban.card.updated", 1),
    ("kanban.card.blocked", 1),
    ("kanban.card.unblocked", 1),
    ("delivery.run.queued", 1),
    ("delivery.run.started", 1),
    ("delivery.run.step", 1),
    ("delivery.run.completed", 1),
    ("delivery.run.failed", 1),
    ("delivery.pr.opened", 1),
    ("delivery.review.submitted", 1),
    ("delivery.merge.completed", 1),
    ("memory.document.created", 1),
    ("memory.compaction.completed", 1),
    ("system.budget.exhausted", 1),
    ("system.health.changed", 1),
    ("security.auth_denied", 1),
    ("system.audit.recorded", 1),
];

/// Look up the schema version of a catalog event.
#[must_use]
pub fn schema_version(event_type: &str) -> Option<u32> {
    EVENT_CATALOG
        .iter()
        .find(|(name, _)| *name == event_type)
        .map(|(_, version)| *version)
}

/// True when the event type is in the catalog.
#[must_use]
pub fn is_known_event(event_type: &str) -> bool {
    schema_version(event_type).is_some()
}

/// Immutable domain event envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    /// Unique event identifier (dedupe key).
    pub event_id: EventId,
    /// Catalog event name.
    pub event_type: String,
    /// Lab scope.
    pub lab_id: LabId,
    /// Per-lab sequence assigned on commit.
    pub seq: Option<i64>,
    /// Server timestamp.
    pub occurred_at: DateTime<Utc>,
    /// Actor.
    pub actor: Actor,
    /// JSON payload; object shape.
    pub payload: serde_json::Value,
    /// Schema version.
    pub schema_version: u32,
}

impl DomainEvent {
    /// Build a validated event.
    pub fn new(
        event_type: impl Into<String>,
        lab_id: LabId,
        actor: Actor,
        payload: serde_json::Value,
        occurred_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let event_type = event_type.into();
        let version = schema_version(&event_type).ok_or_else(|| {
            DomainError::validation("event_type", format!("unknown event {event_type}"))
        })?;
        actor.validate()?;
        if !payload.is_object() {
            return Err(DomainError::validation(
                "event.payload",
                "payload must be a JSON object",
            ));
        }
        Ok(Self {
            event_id: EventId::new(Uuid::now_v7()).unwrap_or_else(|_| EventId::generate()),
            event_type,
            lab_id,
            seq: None,
            occurred_at,
            actor,
            payload,
            schema_version: version,
        })
    }

    /// Assign the committed per-lab sequence.
    pub fn assign_seq(&mut self, seq: i64) -> Result<(), DomainError> {
        if seq <= 0 {
            return Err(DomainError::validation("event.seq", "must be positive"));
        }
        self.seq = Some(seq);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::AgentId;
    use serde_json::json;

    fn lab() -> LabId {
        LabId::generate()
    }

    #[test]
    fn catalog_has_at_least_twenty_entries() {
        assert!(EVENT_CATALOG.len() >= 20);
        assert!(
            EVENT_CATALOG
                .iter()
                .all(|(name, version)| !name.is_empty() && *version >= 1)
        );
        assert!(is_known_event("kanban.card.moved"));
        assert!(!is_known_event("unknown.event"));
    }

    #[test]
    fn event_rejects_unknown_type_and_non_object_payload() {
        assert!(
            DomainEvent::new(
                "unknown.event",
                lab(),
                Actor::system(),
                json!({}),
                Utc::now()
            )
            .is_err()
        );
        assert!(
            DomainEvent::new(
                "kanban.card.moved",
                lab(),
                Actor::system(),
                json!([1, 2]),
                Utc::now()
            )
            .is_err()
        );
    }

    #[test]
    fn event_round_trip_and_seq() {
        let mut event = DomainEvent::new(
            "kanban.card.moved",
            lab(),
            Actor::agent(AgentId::generate()),
            json!({"card_id": "x"}),
            Utc::now(),
        )
        .expect("valid");
        event.assign_seq(7).expect("seq");
        let encoded = serde_json::to_string(&event).expect("serialize");
        let decoded: DomainEvent = serde_json::from_str(&encoded).expect("deserialize");
        assert_eq!(decoded.seq, Some(7));
        assert_eq!(decoded.schema_version, 1);
    }
}
