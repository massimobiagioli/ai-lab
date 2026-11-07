//! Fake aggregate and repository that enforce optimistic concurrency.

use std::sync::Mutex;

use ailab_application::{Repository, RepositoryError};
use ailab_domain::{Aggregate, DomainEvent, Version};
use async_trait::async_trait;
use uuid::Uuid;

/// Minimal aggregate for port tests.
#[derive(Debug, Clone)]
pub struct FakeAggregate {
    /// Identity.
    pub id: Uuid,
    /// Current version.
    pub version: Version,
    /// Uncommitted events.
    pub events: Vec<DomainEvent>,
}

impl FakeAggregate {
    /// New aggregate at version 1.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            version: Version::new(1).expect("version 1 is valid"),
            events: Vec::new(),
        }
    }
}

impl Aggregate for FakeAggregate {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> Version {
        self.version
    }

    fn uncommitted_events(&self) -> &[DomainEvent] {
        &self.events
    }

    fn mark_events_committed(&mut self) {
        self.events.clear();
    }
}

/// In-memory repository that returns a version conflict on a stale save.
#[derive(Debug, Default)]
pub struct FakeRepository {
    stored: Mutex<Option<FakeAggregate>>,
}

impl FakeRepository {
    /// Create an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Repository<FakeAggregate> for FakeRepository {
    async fn load(&self, id: &Uuid) -> Result<Option<FakeAggregate>, RepositoryError> {
        let value = {
            let stored = self.stored.lock().expect("repository lock");
            stored
                .as_ref()
                .filter(|aggregate| &aggregate.id == id)
                .cloned()
        };
        Ok(value)
    }

    async fn save(&self, aggregate: &mut FakeAggregate) -> Result<(), RepositoryError> {
        let mut stored = self.stored.lock().expect("repository lock");
        if let Some(current) = stored.as_ref()
            && aggregate.version.get() <= current.version.get()
        {
            return Err(RepositoryError::VersionConflict {
                expected: current.version.get() + 1,
                current: current.version.get(),
            });
        }
        aggregate.version = aggregate.version.next().map_err(RepositoryError::Domain)?;
        aggregate.mark_events_committed();
        *stored = Some(aggregate.clone());
        drop(stored);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_save_returns_version_conflict() {
        let repository = FakeRepository::new();
        let mut aggregate = FakeAggregate::new(Uuid::now_v7());
        futures::executor::block_on(repository.save(&mut aggregate)).expect("first save");
        let result = futures::executor::block_on(repository.save(&mut aggregate));
        assert!(matches!(
            result,
            Err(RepositoryError::VersionConflict { .. })
        ));
    }
}
