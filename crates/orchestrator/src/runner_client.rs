//! HTTP client for the runner control plane (ADR-0023).

use std::time::Duration;

use ailab_application::Secret;
use ailab_domain::AgentRole;
use async_trait::async_trait;

use crate::backend::{BackendError, CreatedSandbox, ExecOutcome, SandboxBackend, SandboxView};
use crate::runner_api::{CreateSandboxRequest, ExecRequest};
use crate::sandbox::{SandboxSpec, ToolCall};

/// Client for `/v1/sandboxes` on the runner.
pub struct HttpSandboxBackend {
    client: reqwest::Client,
    base_url: String,
    token: Secret<String>,
}

impl HttpSandboxBackend {
    /// Build the client.
    pub fn new(base_url: impl Into<String>, token: Secret<String>) -> Result<Self, BackendError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|error| BackendError::Internal(error.to_string()))?;
        Ok(Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            token,
        })
    }

    /// Build from `AILAB_RUNNER_URL` and `AILAB_RUNNER_TOKEN`.
    pub fn from_env() -> Option<Result<Self, BackendError>> {
        let url = std::env::var("AILAB_RUNNER_URL").ok()?;
        let token = std::env::var("AILAB_RUNNER_TOKEN").ok()?;
        Some(Self::new(url, Secret::new(token)))
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        self.client
            .request(method, format!("{}{path}", self.base_url))
            .bearer_auth(self.token.expose())
    }

    async fn parse<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<T, BackendError> {
        let status = response.status();
        if status.is_success() {
            return response
                .json::<T>()
                .await
                .map_err(|error| BackendError::Internal(error.to_string()));
        }
        let body = response.text().await.unwrap_or_default();
        match status.as_u16() {
            403 => Err(BackendError::Policy(body)),
            404 => Err(BackendError::NotFound),
            422 => Err(BackendError::Validation(body)),
            504 => Err(BackendError::Timeout),
            _ => Err(BackendError::Internal(format!("runner {status}: {body}"))),
        }
    }
}

const fn role_name(role: AgentRole) -> &'static str {
    match role {
        AgentRole::TechLead => "tech_lead",
        AgentRole::Analyst => "analyst",
        AgentRole::Dev => "dev",
        AgentRole::Qa => "qa",
    }
}

#[async_trait]
impl SandboxBackend for HttpSandboxBackend {
    async fn create(
        &self,
        spec: &SandboxSpec,
        role: AgentRole,
    ) -> Result<CreatedSandbox, BackendError> {
        let request = CreateSandboxRequest {
            run_id: spec.run_id.clone(),
            image: spec.image.clone(),
            worktree_ref: spec.worktree_ref.clone(),
            cpu: spec.cpu_millis,
            memory_mb: spec.memory_mb,
            timeout_s: spec.timeout_s,
            egress: spec.egress.clone(),
            egress_proxy: spec.egress_proxy.clone(),
            scm_token: spec
                .scm_token
                .as_ref()
                .map(|token| token.expose().to_owned()),
            role: Some(role_name(role).to_owned()),
        };
        let response = self
            .request(reqwest::Method::POST, "/v1/sandboxes")
            .json(&request)
            .send()
            .await
            .map_err(|error| BackendError::Internal(error.to_string()))?;
        Self::parse(response).await
    }

    async fn status(&self, sandbox_id: &str) -> Result<SandboxView, BackendError> {
        let response = self
            .request(reqwest::Method::GET, &format!("/v1/sandboxes/{sandbox_id}"))
            .send()
            .await
            .map_err(|error| BackendError::Internal(error.to_string()))?;
        Self::parse(response).await
    }

    async fn logs(&self, sandbox_id: &str) -> Result<Vec<String>, BackendError> {
        let response = self
            .request(
                reqwest::Method::GET,
                &format!("/v1/sandboxes/{sandbox_id}/logs"),
            )
            .send()
            .await
            .map_err(|error| BackendError::Internal(error.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                403 => BackendError::Policy("logs denied".to_owned()),
                404 => BackendError::NotFound,
                504 => BackendError::Timeout,
                _ => BackendError::Internal(format!("runner {status}")),
            });
        }
        let body = response
            .text()
            .await
            .map_err(|error| BackendError::Internal(error.to_string()))?;
        Ok(body
            .lines()
            .filter_map(|line| line.strip_prefix("data: "))
            .map(ToOwned::to_owned)
            .collect())
    }

    async fn exec(
        &self,
        sandbox_id: &str,
        call: &ToolCall,
        timeout_s: u64,
    ) -> Result<ExecOutcome, BackendError> {
        let request = ExecRequest {
            argv: call.argv.clone(),
            cwd: call.cwd.clone(),
            timeout_s,
            env_allowlist: Vec::new(),
            tool: Some("shell.run".to_owned()),
        };
        let response = self
            .request(
                reqwest::Method::POST,
                &format!("/v1/sandboxes/{sandbox_id}/exec"),
            )
            .json(&request)
            .send()
            .await
            .map_err(|error| BackendError::Internal(error.to_string()))?;
        Self::parse(response).await
    }

    async fn delete(&self, sandbox_id: &str) -> Result<(), BackendError> {
        let response = self
            .request(
                reqwest::Method::DELETE,
                &format!("/v1/sandboxes/{sandbox_id}"),
            )
            .send()
            .await
            .map_err(|error| BackendError::Internal(error.to_string()))?;
        if response.status().is_success() {
            Ok(())
        } else {
            Self::parse::<serde_json::Value>(response).await.map(|_| ())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::InMemoryBackend;
    use crate::runner_api::{RunnerState, router};
    use crate::sandbox::{EgressPolicy, SandboxSpec};
    use std::sync::Arc;

    #[tokio::test]
    async fn matches_the_runner_contract() {
        let token = Secret::new("x".repeat(32));
        let app = router(RunnerState::with_backend(
            token.clone(),
            Arc::new(InMemoryBackend::new()),
        ));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        let backend = HttpSandboxBackend::new(format!("http://{addr}"), token).expect("client");
        let spec = SandboxSpec::new(
            "http-run-1",
            "ailab/runner@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "worktrees/http-run-1",
            1_000,
            1_024,
            60,
            EgressPolicy::Disabled,
        )
        .expect("spec");
        let created = backend.create(&spec, AgentRole::Dev).await.expect("create");
        let outcome = backend
            .exec(
                &created.sandbox_id,
                &ToolCall {
                    argv: vec!["cargo".to_owned(), "test".to_owned()],
                    cwd: "workspace".to_owned(),
                    writes: Vec::new(),
                },
                10,
            )
            .await
            .expect("exec");
        assert_eq!(outcome.exit_code, 0);
        let view = backend.status(&created.sandbox_id).await.expect("status");
        assert_eq!(view.sandbox_id, created.sandbox_id);
        let logs = backend.logs(&created.sandbox_id).await.expect("logs");
        assert!(logs.iter().any(|line| line.contains("exec")));
        backend.delete(&created.sandbox_id).await.expect("delete");
    }
}
