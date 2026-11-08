//! Sandbox backend abstraction: in-memory reference backend and the contract
//! used by the Docker backend.

use std::collections::BTreeMap;
use std::sync::Mutex;

use ailab_domain::AgentRole;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use thiserror::Error;

use crate::sandbox::{Sandbox, SandboxSpec, SandboxState, ToolCall, ToolPolicy};

/// Backend failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BackendError {
    /// Invalid request.
    #[error("validation failed: {0}")]
    Validation(String),
    /// Policy denial.
    #[error("policy denied: {0}")]
    Policy(String),
    /// The sandbox does not exist.
    #[error("sandbox not found")]
    NotFound,
    /// The command timed out.
    #[error("tool timeout")]
    Timeout,
    /// Backend failure.
    #[error("backend error: {0}")]
    Internal(String),
}

/// A created sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatedSandbox {
    /// Sandbox id.
    pub sandbox_id: String,
    /// Container id.
    pub container_id: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Sandbox status view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SandboxView {
    /// Sandbox id.
    pub sandbox_id: String,
    /// State.
    pub status: SandboxState,
    /// Exit code.
    pub exit_code: Option<i32>,
    /// Start timestamp.
    pub started_at: DateTime<Utc>,
    /// End timestamp.
    pub ended_at: Option<DateTime<Utc>>,
}

/// Exec result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExecOutcome {
    /// Exit code.
    pub exit_code: i32,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Duration in milliseconds.
    pub duration_ms: u64,
}

/// Sandbox backend port.
#[async_trait]
pub trait SandboxBackend: Send + Sync {
    /// Create a sandbox idempotently by `run_id`.
    async fn create(
        &self,
        spec: &SandboxSpec,
        role: AgentRole,
    ) -> Result<CreatedSandbox, BackendError>;
    /// Status.
    async fn status(&self, sandbox_id: &str) -> Result<SandboxView, BackendError>;
    /// Logs.
    async fn logs(&self, sandbox_id: &str) -> Result<Vec<String>, BackendError>;
    /// Execute an allowlisted call.
    async fn exec(
        &self,
        sandbox_id: &str,
        call: &ToolCall,
        timeout_s: u64,
    ) -> Result<ExecOutcome, BackendError>;
    /// Delete the sandbox.
    async fn delete(&self, sandbox_id: &str) -> Result<(), BackendError>;
}

struct MemoryRecord {
    sandbox: Sandbox,
    role: AgentRole,
    id: String,
    created_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
    logs: Vec<String>,
}

/// In-memory reference backend used by tests and the local development profile.
#[derive(Default)]
pub struct InMemoryBackend {
    inner: Mutex<BTreeMap<String, MemoryRecord>>,
    by_run: Mutex<BTreeMap<String, String>>,
}

impl InMemoryBackend {
    /// New empty backend.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SandboxBackend for InMemoryBackend {
    async fn create(
        &self,
        spec: &SandboxSpec,
        role: AgentRole,
    ) -> Result<CreatedSandbox, BackendError> {
        if let Ok(by_run) = self.by_run.lock()
            && let Some(existing) = by_run.get(&spec.run_id)
            && let Ok(inner) = self.inner.lock()
            && let Some(record) = inner.get(existing)
        {
            return Ok(CreatedSandbox {
                sandbox_id: record.id.clone(),
                container_id: record.id.clone(),
                created_at: record.created_at,
            });
        }
        let id = uuid::Uuid::new_v4().to_string();
        let mut sandbox = Sandbox::new(spec.clone());
        sandbox.mark_running();
        let created_at = Utc::now();
        let record = MemoryRecord {
            sandbox,
            role,
            id: id.clone(),
            created_at,
            ended_at: None,
            logs: vec!["sandbox created".to_owned()],
        };
        self.inner
            .lock()
            .map_err(|_| BackendError::Internal("sandbox lock poisoned".to_owned()))?
            .insert(id.clone(), record);
        self.by_run
            .lock()
            .map_err(|_| BackendError::Internal("sandbox lock poisoned".to_owned()))?
            .insert(spec.run_id.clone(), id.clone());
        Ok(CreatedSandbox {
            sandbox_id: id.clone(),
            container_id: id,
            created_at,
        })
    }

    async fn status(&self, sandbox_id: &str) -> Result<SandboxView, BackendError> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| BackendError::Internal("sandbox lock poisoned".to_owned()))?;
        let record = inner.get(sandbox_id).ok_or(BackendError::NotFound)?;
        let view = SandboxView {
            sandbox_id: record.id.clone(),
            status: record.sandbox.state(),
            exit_code: record.sandbox.exit_code(),
            started_at: record.created_at,
            ended_at: record.ended_at,
        };
        drop(inner);
        Ok(view)
    }

    async fn logs(&self, sandbox_id: &str) -> Result<Vec<String>, BackendError> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| BackendError::Internal("sandbox lock poisoned".to_owned()))?;
        let record = inner.get(sandbox_id).ok_or(BackendError::NotFound)?;
        let logs = record.logs.clone();
        drop(inner);
        Ok(logs)
    }

    async fn exec(
        &self,
        sandbox_id: &str,
        call: &ToolCall,
        _timeout_s: u64,
    ) -> Result<ExecOutcome, BackendError> {
        let start = std::time::Instant::now();
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| BackendError::Internal("sandbox lock poisoned".to_owned()))?;
        let record = inner.get_mut(sandbox_id).ok_or(BackendError::NotFound)?;
        let policy = ToolPolicy::for_role(record.role);
        record
            .sandbox
            .exec(call, &policy)
            .map_err(|error| BackendError::Policy(error.to_string()))?;
        record.logs.push(format!("exec: {}", call.argv.join(" ")));
        drop(inner);
        Ok(ExecOutcome {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
        })
    }

    async fn delete(&self, sandbox_id: &str) -> Result<(), BackendError> {
        let record = self
            .inner
            .lock()
            .map_err(|_| BackendError::Internal("sandbox lock poisoned".to_owned()))?
            .remove(sandbox_id)
            .ok_or(BackendError::NotFound)?;
        if let Ok(mut by_run) = self.by_run.lock() {
            by_run.remove(&record.sandbox.spec().run_id);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::EgressPolicy;

    fn spec() -> SandboxSpec {
        SandboxSpec::new(
            "run-1",
            "ailab/runner@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "worktrees/run-1",
            1_000,
            1_024,
            900,
            EgressPolicy::Allowlist(vec!["crates.io".to_owned()]),
        )
        .expect("spec")
    }

    #[tokio::test]
    async fn create_is_idempotent_and_lifecycle_works() {
        let backend = InMemoryBackend::new();
        let first = backend
            .create(&spec(), AgentRole::Dev)
            .await
            .expect("create");
        let second = backend
            .create(&spec(), AgentRole::Dev)
            .await
            .expect("create");
        assert_eq!(first.sandbox_id, second.sandbox_id);
        let status = backend.status(&first.sandbox_id).await.expect("status");
        assert_eq!(status.status, SandboxState::Running);
        let logs = backend.logs(&first.sandbox_id).await.expect("logs");
        assert_ne!(logs.len(), 0);
        backend.delete(&first.sandbox_id).await.expect("delete");
        assert_eq!(
            backend.status(&first.sandbox_id).await,
            Err(BackendError::NotFound)
        );
    }

    #[tokio::test]
    async fn exec_enforces_the_tool_policy() {
        let backend = InMemoryBackend::new();
        let created = backend
            .create(&spec(), AgentRole::Dev)
            .await
            .expect("create");
        let call = ToolCall {
            argv: vec![
                "sudo".to_owned(),
                "rm".to_owned(),
                "-rf".to_owned(),
                "/".to_owned(),
            ],
            cwd: "workspace".to_owned(),
            writes: vec![],
        };
        assert!(matches!(
            backend.exec(&created.sandbox_id, &call, 10).await,
            Err(BackendError::Policy(_))
        ));
    }
}
