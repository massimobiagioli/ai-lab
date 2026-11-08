//! AI-Lab orchestrator: scheduler, run pipeline, sandbox policy and sandbox
//! control-plane types. The runner binary is built from this crate.
#![forbid(unsafe_code)]

pub mod backend;
pub mod context;
pub mod docker_backend;
pub mod orchestration;
pub mod pipeline;
pub mod runner_api;
pub mod sandbox;
pub mod wired;
pub mod worker;
