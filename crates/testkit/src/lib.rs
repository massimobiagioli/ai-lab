//! AI-Lab test kit: deterministic fakes and golden fixtures.
#![forbid(unsafe_code)]

pub mod fakes;
pub mod fixtures;
pub mod repository;

pub use fakes::{FixedClock, RecordingEventBus, SequentialIdGenerator};
pub use fixtures::golden_events;
pub use repository::{FakeAggregate, FakeRepository};
