//! In-memory transactional outbox used by the kernel and tests.
//!
//! The Postgres adapter lands with WP-13; this implementation proves the
//! sequencing, dedupe and at-least-once semantics.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, MutexGuard};

use ailab_application::{Outbox, RepositoryError};
use ailab_domain::{DomainEvent, LabId};
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Default)]
struct State {
    pending: Vec<DomainEvent>,
    seen: HashSet<Uuid>,
    next_seq: HashMap<LabId, i64>,
}

/// In-memory outbox with per-lab sequencing and event-id deduplication.
#[derive(Default)]
pub struct InMemoryOutbox {
    state: Mutex<State>,
}

impl InMemoryOutbox {
    /// Create an empty outbox.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> Result<MutexGuard<'_, State>, RepositoryError> {
        self.state
            .lock()
            .map_err(|_| RepositoryError::Storage("outbox lock poisoned".to_owned()))
    }
}

#[async_trait]
impl Outbox for InMemoryOutbox {
    async fn append(&self, events: &[DomainEvent]) -> Result<(), RepositoryError> {
        let mut state = self.lock()?;
        for source in events {
            let event_id = source.event_id.as_uuid();
            if !state.seen.insert(event_id) {
                continue;
            }
            let mut event = source.clone();
            if event.seq.is_none() {
                let next = state.next_seq.entry(event.lab_id).or_insert(0);
                *next += 1;
                event.assign_seq(*next)?;
            }
            state.pending.push(event);
        }
        drop(state);
        Ok(())
    }

    async fn pending(&self) -> Result<Vec<DomainEvent>, RepositoryError> {
        let pending = {
            let state = self.lock()?;
            state.pending.clone()
        };
        Ok(pending)
    }

    async fn acknowledge(&self, event_id: Uuid) -> Result<(), RepositoryError> {
        let mut state = self.lock()?;
        state
            .pending
            .retain(|event| event.event_id.as_uuid() != event_id);
        drop(state);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ailab_domain::{Actor, LabId};
    use serde_json::json;

    fn event(lab: LabId, name: &str) -> DomainEvent {
        DomainEvent::new(
            name,
            lab,
            Actor::system(),
            json!({"fixture": true}),
            chrono::Utc::now(),
        )
        .expect("valid event")
    }

    #[test]
    fn assigns_monotonic_sequences_per_lab() {
        let outbox = InMemoryOutbox::new();
        let lab = LabId::generate();
        let first = event(lab, "kanban.card.created");
        let second = event(lab, "kanban.card.moved");
        futures::executor::block_on(outbox.append(&[first, second])).expect("append");
        let pending = futures::executor::block_on(outbox.pending()).expect("pending");
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].seq, Some(1));
        assert_eq!(pending[1].seq, Some(2));
    }

    #[test]
    fn sequences_are_independent_across_labs() {
        let outbox = InMemoryOutbox::new();
        let first_lab = LabId::generate();
        let second_lab = LabId::generate();
        futures::executor::block_on(outbox.append(&[event(first_lab, "lab.budget.changed")]))
            .expect("append");
        futures::executor::block_on(outbox.append(&[event(second_lab, "lab.budget.changed")]))
            .expect("append");
        let pending = futures::executor::block_on(outbox.pending()).expect("pending");
        assert_eq!(pending[0].seq, Some(1));
        assert_eq!(pending[1].seq, Some(1));
    }

    #[test]
    fn duplicate_event_id_is_ignored() {
        let outbox = InMemoryOutbox::new();
        let lab = LabId::generate();
        let event = event(lab, "kanban.card.created");
        futures::executor::block_on(outbox.append(&[event.clone(), event])).expect("append");
        assert_eq!(
            futures::executor::block_on(outbox.pending())
                .expect("pending")
                .len(),
            1
        );
    }

    #[test]
    fn at_least_once_until_acknowledged() {
        let outbox = InMemoryOutbox::new();
        let lab = LabId::generate();
        let event = event(lab, "system.health.changed");
        let event_id = event.event_id.as_uuid();
        futures::executor::block_on(outbox.append(&[event])).expect("append");
        assert_eq!(
            futures::executor::block_on(outbox.pending())
                .expect("pending")
                .len(),
            1
        );
        futures::executor::block_on(outbox.pending()).expect("pending again");
        futures::executor::block_on(outbox.acknowledge(event_id)).expect("ack");
        assert_eq!(
            futures::executor::block_on(outbox.pending())
                .expect("pending")
                .len(),
            0
        );
    }

    #[test]
    fn concurrent_writers_get_unique_sequences() {
        use std::sync::Arc;
        use std::thread;

        let outbox = Arc::new(InMemoryOutbox::new());
        let lab = LabId::generate();
        let writers: Vec<_> = (0..20)
            .map(|index| {
                let outbox = Arc::clone(&outbox);
                thread::spawn(move || {
                    let event = event(
                        lab,
                        if index % 2 == 0 {
                            "kanban.card.created"
                        } else {
                            "kanban.card.moved"
                        },
                    );
                    futures::executor::block_on(outbox.append(&[event])).expect("append");
                })
            })
            .collect();
        for writer in writers {
            writer.join().expect("writer");
        }
        let mut seqs: Vec<i64> = futures::executor::block_on(outbox.pending())
            .expect("pending")
            .into_iter()
            .filter_map(|event| event.seq)
            .collect();
        seqs.sort_unstable();
        assert_eq!(seqs, (1..=20).collect::<Vec<i64>>());
    }
}
