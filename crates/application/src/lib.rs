//! AI-Lab application kernel: ports, problem mapping, secrets and config.
#![forbid(unsafe_code)]

pub mod config;
pub mod ports;
pub mod problem;
pub mod qa;
pub mod secret;

pub use config::{Config, ConfigError, KNOWN_VARIABLES, unknown_variables};
pub use ports::{
    BusError, Clock, EventBus, IdGenerator, Outbox, Repository, RepositoryError, TransactionManager,
};
pub use problem::{ProblemDetails, Violation};
pub use qa::{MergeChecks, MergeDecision, MergeViolation, ReviewVerdict, evaluate_merge};
pub use secret::Secret;
