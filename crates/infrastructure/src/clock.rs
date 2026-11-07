//! System clock adapter.

use ailab_application::Clock;
use chrono::{DateTime, Utc};

/// UTC wall clock.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
