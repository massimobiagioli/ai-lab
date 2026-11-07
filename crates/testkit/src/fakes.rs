//! Deterministic fakes for clocks, ids and event buses.

use std::sync::Mutex;

use ailab_application::{BusError, Clock, EventBus, IdGenerator};
use ailab_domain::DomainEvent;
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use uuid::Uuid;

/// Fixed clock.
#[derive(Debug, Clone, Copy)]
pub struct FixedClock {
    /// Fixed instant.
    pub now: DateTime<Utc>,
}

impl FixedClock {
    /// Build a clock at the given instant.
    #[must_use]
    pub const fn new(now: DateTime<Utc>) -> Self {
        Self { now }
    }

    /// Fixed instant at 2026-01-01T00:00:00Z.
    #[must_use]
    pub fn epoch() -> Self {
        let now = Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
            .single()
            .expect("valid timestamp");
        Self { now }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.now
    }
}

/// Monotonic UUID generator.
#[derive(Debug, Default)]
pub struct SequentialIdGenerator {
    next: Mutex<u128>,
}

impl SequentialIdGenerator {
    /// Create a generator starting at 1.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl IdGenerator for SequentialIdGenerator {
    fn new_uuid(&self) -> Uuid {
        let mut next = self.next.lock().expect("id lock");
        *next += 1;
        Uuid::from_u128(*next)
    }
}

/// Event bus that records published events.
#[derive(Debug, Default)]
pub struct RecordingEventBus {
    events: Mutex<Vec<DomainEvent>>,
}

impl RecordingEventBus {
    /// Create an empty bus.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot the published events.
    #[must_use]
    pub fn published(&self) -> Vec<DomainEvent> {
        self.events.lock().expect("bus lock").clone()
    }
}

#[async_trait]
impl EventBus for RecordingEventBus {
    async fn publish(&self, event: &DomainEvent) -> Result<(), BusError> {
        self.events.lock().expect("bus lock").push(event.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ailab_domain::{Actor, LabId};
    use serde_json::json;

    #[test]
    fn clock_is_fixed() {
        let clock = FixedClock::epoch();
        assert_eq!(clock.now(), clock.now());
    }

    #[test]
    fn ids_are_sequential() {
        let generator = SequentialIdGenerator::new();
        let first = generator.new_uuid();
        let second = generator.new_uuid();
        assert_ne!(first, second);
        assert!(first < second);
    }

    #[test]
    fn bus_records_events() {
        let bus = RecordingEventBus::new();
        let event = DomainEvent::new(
            "system.health.changed",
            LabId::generate(),
            Actor::system(),
            json!({"status": "ok"}),
            Utc::now(),
        )
        .expect("valid");
        futures::executor::block_on(bus.publish(&event)).expect("publish");
        assert_eq!(bus.published().len(), 1);
    }
}
