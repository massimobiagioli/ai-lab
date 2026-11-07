//! Aggregate base contract.

use crate::event::DomainEvent;
use crate::value::Version;

/// Aggregate root contract: identity, version and uncommitted events.
pub trait Aggregate {
    /// Identity type.
    type Id;

    /// Aggregate identity.
    fn id(&self) -> &Self::Id;

    /// Current version.
    fn version(&self) -> Version;

    /// Events produced but not yet committed.
    fn uncommitted_events(&self) -> &[DomainEvent];

    /// Clear the uncommitted events after a successful commit.
    fn mark_events_committed(&mut self);
}
