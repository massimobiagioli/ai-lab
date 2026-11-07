//! AI-Lab domain kernel: identifiers, value objects, errors, events and the
//! aggregate contract. This crate performs no I/O.
#![forbid(unsafe_code)]

pub mod aggregate;
pub mod delivery;
pub mod error;
pub mod event;
pub mod ids;
pub mod inventory;
pub mod kanban;
pub mod lab;
pub mod memory;
pub mod value;

pub use aggregate::Aggregate;
pub use error::DomainError;
pub use event::{DomainEvent, EVENT_CATALOG, is_known_event, schema_version};
pub use ids::{
    AgentId, CardId, CardKey, ChunkId, CvVersionId, DocumentId, EventId, LabId, ModelId, PlanId,
    ProjectId, ProviderId, RunId, ScmCredentialId, SowId, WorkspaceId,
};
pub use value::{
    Actor, ActorType, AgentRole, AutonomyLevel, ContentHash, DueDate, Estimate, IdempotencyKey,
    KanbanColumn, Money, Priority, Slug, TraceId, Version,
};
