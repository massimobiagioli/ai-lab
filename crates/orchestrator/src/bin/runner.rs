//! AI-Lab `runner` binary: authenticated sandbox control-plane API.
#![forbid(unsafe_code)]

use std::time::Duration;

use std::sync::Arc;

use ailab_application::Secret;
use ailab_orchestrator::backend::{InMemoryBackend, SandboxBackend};
use ailab_orchestrator::docker_backend::DockerBackend;
use ailab_orchestrator::runner_api::{RunnerState, router};

fn main() {
    if std::env::args().any(|arg| arg == "healthcheck") {
        let healthy = ailab_infrastructure::health::require_env("AILAB_RUNNER_TOKEN")
            && ailab_infrastructure::health::check_urls(&["DOCKER_HOST"], Duration::from_secs(3));
        std::process::exit(i32::from(!healthy));
    }

    let token = match std::env::var("AILAB_RUNNER_TOKEN") {
        Ok(value) if value.trim().len() >= 32 => value,
        _ => {
            eprintln!("runner: AILAB_RUNNER_TOKEN (>= 32 bytes) is required");
            std::process::exit(78);
        }
    };
    let bind = std::env::var("AILAB_RUNNER_BIND").unwrap_or_else(|_| "0.0.0.0:8081".to_owned());

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("runner: cannot start the async runtime: {error}");
            std::process::exit(1);
        }
    };
    let backend = if std::env::var("AILAB_RUNNER_BACKEND").as_deref() == Ok("memory") {
        Arc::new(InMemoryBackend::new()) as Arc<dyn SandboxBackend>
    } else {
        match DockerBackend::from_env() {
            Ok(backend) => Arc::new(backend) as Arc<dyn SandboxBackend>,
            Err(error) => {
                eprintln!("runner: docker backend unavailable: {error}");
                std::process::exit(1);
            }
        }
    };
    runtime.block_on(async move {
        let app = router(RunnerState::with_backend(Secret::new(token), backend));
        let listener = match tokio::net::TcpListener::bind(&bind).await {
            Ok(listener) => listener,
            Err(error) => {
                eprintln!("runner: cannot bind {bind}: {error}");
                std::process::exit(1);
            }
        };
        if let Err(error) = axum::serve(listener, app).await {
            eprintln!("runner: server error: {error}");
            std::process::exit(1);
        }
    });
}
