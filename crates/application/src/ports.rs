//! Ports implemented by adapters and composed by the API and worker.

use ailab_domain::{Aggregate, DomainEvent};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Repository failure.
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    /// Domain rule rejected the operation.
    #[error(transparent)]
    Domain(#[from] ailab_domain::DomainError),
    /// Optimistic concurrency lost.
    #[error("version conflict: expected {expected}, current {current}")]
    VersionConflict {
        /// Expected version.
        expected: u32,
        /// Current version.
        current: u32,
    },
    /// Storage failure.
    #[error("storage error: {0}")]
    Storage(String),
}

/// Event bus failure.
#[derive(Debug, thiserror::Error)]
pub enum BusError {
    /// Transport failure.
    #[error("event bus error: {0}")]
    Transport(String),
}

/// Aggregate repository with optimistic concurrency.
#[async_trait]
pub trait Repository<A>: Send + Sync
where
    A: Aggregate + Send + Sync,
    A::Id: Sync,
{
    /// Load an aggregate by id.
    async fn load(&self, id: &A::Id) -> Result<Option<A>, RepositoryError>;
    /// Save the aggregate and its uncommitted events atomically.
    async fn save(&self, aggregate: &mut A) -> Result<(), RepositoryError>;
}

/// Wall clock.
pub trait Clock: Send + Sync {
    /// Current UTC time.
    fn now(&self) -> DateTime<Utc>;
}

/// Identity generator.
pub trait IdGenerator: Send + Sync {
    /// New `UUIDv7`.
    fn new_uuid(&self) -> Uuid;
}

/// Event publisher.
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish one event.
    async fn publish(&self, event: &DomainEvent) -> Result<(), BusError>;
}

/// Transactional outbox.
#[async_trait]
pub trait Outbox: Send + Sync {
    /// Append events to the outbox.
    async fn append(&self, events: &[DomainEvent]) -> Result<(), RepositoryError>;
    /// Return events that are not yet acknowledged.
    async fn pending(&self) -> Result<Vec<DomainEvent>, RepositoryError>;
    /// Acknowledge one event after it is published.
    async fn acknowledge(&self, event_id: Uuid) -> Result<(), RepositoryError>;
}

/// Transaction manager port.
pub trait TransactionManager: Send + Sync {
    /// Transaction handle.
    type Tx: Send;

    /// Begin a transaction.
    fn begin(&self) -> Result<Self::Tx, RepositoryError>;
    /// Commit a transaction.
    fn commit(&self, tx: Self::Tx) -> Result<(), RepositoryError>;
}
