//! AI-Lab `worker` binary: scheduler, leases and the run pipeline.
#![forbid(unsafe_code)]

use std::sync::Arc;
use std::time::Duration;

use ailab_llm::{FakeLlm, LlmProvider};
use ailab_orchestrator::backend::{InMemoryBackend, SandboxBackend};
use ailab_orchestrator::postgres::{PostgresCancelProbe, PostgresScheduler, PostgresWorker};
use ailab_orchestrator::runner_client::HttpSandboxBackend;
use ailab_orchestrator::wired::WiredExecutor;
use ailab_orchestrator::worker::RunExecutor;
use ailab_scm::{FakeScm, RepoRef, ScmProvider};

#[allow(clippy::bool_to_int_with_if)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|arg| arg == "healthcheck") {
        let healthy = ailab_infrastructure::health::check_urls(
            &["AILAB_DATABASE_URL", "AILAB_REDIS_URL", "AILAB_RUNNER_URL"],
            Duration::from_secs(3),
        );
        std::process::exit(if healthy { 0 } else { 1 });
    }

    if std::env::var("AILAB_DEV_FAKES").is_err() {
        return Err(
            "real LLM/SCM adapters are not wired; set AILAB_DEV_FAKES=1 for the in-memory harness"
                .into(),
        );
    }
    let Ok(database_url) = std::env::var("AILAB_DATABASE_URL") else {
        loop {
            tokio::time::sleep(Duration::from_secs(3_600)).await;
        }
    };
    let scheduler = PostgresScheduler::connect(&database_url).await?;
    let llm: Arc<dyn LlmProvider> = Arc::new(FakeLlm::new(Vec::new()));
    let scm: Arc<dyn ScmProvider> = Arc::new(FakeScm::default());
    let sandbox: Arc<dyn SandboxBackend> = match HttpSandboxBackend::from_env() {
        Some(client) => Arc::new(client?),
        None => Arc::new(InMemoryBackend::new()),
    };
    let probe = Arc::new(PostgresCancelProbe::new(scheduler.pool().clone()));
    let executor: Arc<dyn RunExecutor> = Arc::new(
        WiredExecutor::new(
            llm,
            scm,
            sandbox,
            RepoRef::parse("ai-lab/demo")?,
            "WP-00",
            "agent-dev",
            "agent-qa",
        )
        .with_cancel_probe(probe),
    );
    let worker = PostgresWorker::new(scheduler, executor, 4);
    loop {
        if let Err(error) = worker.tick(chrono::Utc::now()).await {
            eprintln!("worker tick failed: {error}");
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
