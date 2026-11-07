//! AI-Lab `runner` binary.
//!
//! The sandbox control plane lands with WP-14. WP-03 delivers the platform
//! health probe and keeps the process alive for the container health model.
#![forbid(unsafe_code)]

use std::time::Duration;

#[allow(clippy::bool_to_int_with_if)]
fn main() {
    if std::env::args().any(|arg| arg == "healthcheck") {
        let healthy = ailab_infrastructure::health::require_env("AILAB_RUNNER_TOKEN")
            && ailab_infrastructure::health::check_urls(&["DOCKER_HOST"], Duration::from_secs(3));
        std::process::exit(if healthy { 0 } else { 1 });
    }

    loop {
        std::thread::sleep(Duration::from_secs(3600));
    }
}
