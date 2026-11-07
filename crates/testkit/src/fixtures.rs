//! Golden event fixtures, one per catalog entry.

use ailab_domain::{Actor, DomainEvent, EVENT_CATALOG, LabId};
use chrono::{TimeZone, Utc};
use serde_json::json;

/// One valid event per catalog entry.
#[must_use]
pub fn golden_events() -> Vec<DomainEvent> {
    let lab = LabId::generate();
    let occurred_at = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
        .expect("valid timestamp");
    EVENT_CATALOG
        .iter()
        .map(|(name, _)| {
            DomainEvent::new(
                *name,
                lab,
                Actor::system(),
                json!({"fixture": name}),
                occurred_at,
            )
            .expect("catalog event is valid")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalog_event_has_a_golden_fixture() {
        let events = golden_events();
        assert_eq!(events.len(), EVENT_CATALOG.len());
        assert!(events.len() >= 20);
        for event in &events {
            assert!(event.payload.is_object());
            assert_eq!(event.schema_version, 1);
        }
    }
}
