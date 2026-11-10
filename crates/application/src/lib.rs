//! AI-Lab application kernel: ports, problem mapping, secrets and config.
#![forbid(unsafe_code)]

pub mod config;
pub mod delivery;
pub mod management;
pub mod ports;
pub mod problem;
pub mod qa;
pub mod secret;

pub use config::{Config, ConfigError, KNOWN_VARIABLES, unknown_variables};
pub use delivery::{
    DEFAULT_LIMIT, EventEnvelope, MAX_LIMIT, Page, PageQuery, RunCursor, RunStore, RunView,
    StepView, StoreError, events_after,
};
pub use management::{
    CreateAgentInput, CreateLabRequest, CreateProjectRequest, CreateProviderRequest,
    ManagementError, ManagementService, RotateProviderKeyRequest, UpdateAgentRequest,
    UpdateProjectRequest,
};
pub use ports::{
    BusError, Clock, EventBus, IdGenerator, Outbox, Repository, RepositoryError, TransactionManager,
};
pub use problem::{ProblemDetails, Violation};
pub use qa::{
    AcceptanceResult, AcceptanceStatus, Finding, FindingSeverity, GateState, HardStop,
    HardStopInputs, HumanGate, MergeChecks, MergeDecision, MergeViolation, ReviewPipeline,
    ReviewRecord, ReviewVerdict, ReworkLoop, ReworkOutcome, evaluate_hard_stops, evaluate_merge,
    evaluate_pre_merge,
};
pub use secret::Secret;
