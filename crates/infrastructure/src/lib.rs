//! AI-Lab `infrastructure` crate.
//!
//! Adapters and cross-cutting runtime helpers: repositories, Redis, clock,
//! identifiers, health probes, cryptography and security primitives.
#![forbid(unsafe_code)]

pub mod clock;
pub mod crypto;
pub mod exposure;
pub mod health;
pub mod ids;
pub mod management_memory;
pub mod management_postgres;
pub mod outbox;
pub mod postgres;
pub mod redact;
pub mod redis_keys;
pub mod run_store;
pub mod security_headers;
