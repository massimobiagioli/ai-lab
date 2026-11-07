//! AI-Lab `worker` binary.
//!
//! The scheduler and run pipeline land with WP-13. WP-03 delivers the platform
//! health probe and keeps the process alive for the container health model.
#![forbid(unsafe_code)]

use std::time::Duration;

#[allow(clippy::bool_to_int_with_if)]
fn main() {
    if std::env::args().any(|arg| arg == "healthcheck") {
        let healthy = ailab_infrastructure::health::check_urls(
            &["AILAB_DATABASE_URL", "AILAB_REDIS_URL", "AILAB_RUNNER_URL"],
            Duration::from_secs(3),
        );
        std::process::exit(if healthy { 0 } else { 1 });
    }

    loop {
        std::thread::sleep(Duration::from_secs(3600));
    }
}
