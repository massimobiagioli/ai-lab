//! `UUIDv7` identity generator adapter.

use ailab_application::IdGenerator;
use uuid::Uuid;

/// Time-ordered `UUIDv7` generator.
#[derive(Debug, Default, Clone, Copy)]
pub struct Uuid7Generator;

impl IdGenerator for Uuid7Generator {
    fn new_uuid(&self) -> Uuid {
        Uuid::now_v7()
    }
}
