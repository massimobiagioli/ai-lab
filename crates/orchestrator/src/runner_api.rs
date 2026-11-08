//! Runner control-plane HTTP API (WP-14 §3).
//!
//! The API is authenticated by `AILAB_RUNNER_TOKEN` on the private runner
//! network. Sandboxes are idempotent by `run_id`. The backend is pluggable:
//! in-memory for tests/dev and Docker for the runtime.

use std::sync::Arc;
use std::time::Instant;

use ailab_application::Secret;
use ailab_domain::AgentRole;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::backend::{BackendError, CreatedSandbox, ExecOutcome, SandboxBackend, SandboxView};
use crate::sandbox::{EgressPolicy, SandboxSpec, ToolCall};

/// Create-sandbox request.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSandboxRequest {
    /// Run id (idempotency key).
    pub run_id: String,
    /// Digest-pinned image.
    pub image: String,
    /// Worktree reference.
    pub worktree_ref: String,
    /// CPU millicores.
    pub cpu: u32,
    /// Memory MB.
    pub memory_mb: u32,
    /// Timeout seconds.
    pub timeout_s: u64,
    /// Egress policy.
    pub egress: EgressPolicy,
}

/// Create-sandbox response.
pub type CreateSandboxResponse = CreatedSandbox;
/// Sandbox status response.
pub type SandboxStatus = SandboxView;
/// Exec response.
pub type ExecResponse = ExecOutcome;

/// Exec request.
#[derive(Debug, Clone, Deserialize)]
pub struct ExecRequest {
    /// Command and arguments.
    pub argv: Vec<String>,
    /// Working directory.
    pub cwd: String,
    /// Timeout seconds.
    pub timeout_s: u64,
    /// Environment allowlist (names only).
    #[serde(default)]
    pub env_allowlist: Vec<String>,
}

/// Log query.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LogQuery {
    /// Follow (accepted; the backends return the current buffer).
    #[serde(default)]
    pub follow: bool,
}

/// API error payload.
#[derive(Debug, Clone, Serialize)]
pub struct ApiErrorBody {
    /// Stable code.
    pub code: &'static str,
    /// Message.
    pub message: String,
}

/// API error.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "UNAUTHORIZED",
            message: "missing or invalid runner token".to_owned(),
        }
    }

    fn validation(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "VALIDATION_FAILED",
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "INTERNAL",
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorBody {
                code: self.code,
                message: self.message,
            }),
        )
            .into_response()
    }
}

impl From<BackendError> for ApiError {
    fn from(error: BackendError) -> Self {
        match error {
            BackendError::Validation(message) => Self::validation(message),
            BackendError::Policy(message) => Self {
                status: StatusCode::FORBIDDEN,
                code: "POLICY_DENIED",
                message,
            },
            BackendError::NotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "NOT_FOUND",
                message: "sandbox not found".to_owned(),
            },
            BackendError::Internal(message) => Self::internal(message),
        }
    }
}

/// Shared runner state.
#[derive(Clone)]
pub struct RunnerState {
    backend: Arc<dyn SandboxBackend>,
    token: Secret<String>,
}

impl RunnerState {
    /// Build the state with the shared runner token and the in-memory backend.
    #[must_use]
    pub fn new(token: Secret<String>) -> Self {
        Self::with_backend(token, Arc::new(crate::backend::InMemoryBackend::new()))
    }

    /// Build the state with a specific backend.
    #[must_use]
    pub fn with_backend(token: Secret<String>, backend: Arc<dyn SandboxBackend>) -> Self {
        Self { backend, token }
    }

    fn authorize(&self, headers: &HeaderMap) -> Result<(), ApiError> {
        let provided = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "));
        if ailab_infrastructure::exposure::token_matches(self.token.expose(), provided) {
            Ok(())
        } else {
            Err(ApiError::unauthorized())
        }
    }
}

/// Build the runner router.
pub fn router(state: RunnerState) -> Router {
    Router::new()
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_ready))
        .route("/v1/sandboxes", post(create_sandbox))
        .route(
            "/v1/sandboxes/{id}",
            get(get_sandbox).delete(delete_sandbox),
        )
        .route("/v1/sandboxes/{id}/logs", get(get_logs))
        .route("/v1/sandboxes/{id}/exec", post(exec_command))
        .with_state(state)
}

async fn health_live() -> StatusCode {
    StatusCode::OK
}

async fn health_ready() -> StatusCode {
    StatusCode::OK
}

async fn create_sandbox(
    State(state): State<RunnerState>,
    headers: HeaderMap,
    Json(request): Json<CreateSandboxRequest>,
) -> Result<(StatusCode, Json<CreateSandboxResponse>), ApiError> {
    state.authorize(&headers)?;
    let spec = SandboxSpec::new(
        request.run_id,
        request.image,
        request.worktree_ref,
        request.cpu,
        request.memory_mb,
        request.timeout_s,
        request.egress,
    )
    .map_err(|error| ApiError::validation(error.to_string()))?;
    let created = state
        .backend
        .create(&spec, AgentRole::Dev)
        .await
        .map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(created)))
}

async fn get_sandbox(
    State(state): State<RunnerState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<SandboxStatus>, ApiError> {
    state.authorize(&headers)?;
    Ok(Json(
        state.backend.status(&id).await.map_err(ApiError::from)?,
    ))
}

async fn delete_sandbox(
    State(state): State<RunnerState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.authorize(&headers)?;
    state.backend.delete(&id).await.map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_logs(
    State(state): State<RunnerState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(_query): Query<LogQuery>,
) -> Result<Response, ApiError> {
    state.authorize(&headers)?;
    let lines = state.backend.logs(&id).await.map_err(ApiError::from)?;
    let mut body = String::new();
    for line in lines {
        body.push_str("data: ");
        body.push_str(&line);
        body.push_str("\n\n");
    }
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
        body,
    )
        .into_response())
}

async fn exec_command(
    State(state): State<RunnerState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<ExecRequest>,
) -> Result<Json<ExecResponse>, ApiError> {
    state.authorize(&headers)?;
    let start = Instant::now();
    let call = ToolCall {
        argv: request.argv,
        cwd: request.cwd,
        writes: Vec::new(),
    };
    let mut outcome = state
        .backend
        .exec(&id, &call, request.timeout_s)
        .await
        .map_err(ApiError::from)?;
    outcome.duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    Ok(Json(outcome))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    const TOKEN: &str = "0123456789abcdef0123456789abcdef";

    fn state() -> RunnerState {
        RunnerState::new(Secret::new(TOKEN.to_owned()))
    }

    fn request(method: &str, uri: &str, body: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("authorization", format!("Bearer {TOKEN}"));
        if body.is_some() {
            builder = builder.header("content-type", "application/json");
        }
        builder
            .body(Body::from(body.unwrap_or("").to_owned()))
            .expect("request")
    }

    fn create_body() -> String {
        serde_json::json!({
            "run_id": "run-1",
            "image": "ailab/runner@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "worktree_ref": "worktrees/run-1",
            "cpu": 1000,
            "memory_mb": 512,
            "timeout_s": 900,
            "egress": {"Allowlist": ["crates.io"]}
        })
        .to_string()
    }

    async fn body_json(response: Response) -> serde_json::Value {
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        serde_json::from_slice(&bytes).expect("json")
    }

    #[tokio::test]
    async fn health_endpoints_are_open() {
        let router = router(state());
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health/live")
                    .body(Body::empty())
                    .expect("req"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .expect("req"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn unauthenticated_requests_are_rejected() {
        let router = router(state());
        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/sandboxes")
                    .header("content-type", "application/json")
                    .body(Body::from(create_body()))
                    .expect("req"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn create_is_idempotent_by_run_id() {
        let router = router(state());
        let first = router
            .clone()
            .oneshot(request("POST", "/v1/sandboxes", Some(&create_body())))
            .await
            .expect("response");
        assert_eq!(first.status(), StatusCode::CREATED);
        let id = body_json(first).await["sandbox_id"]
            .as_str()
            .expect("id")
            .to_owned();
        let second = router
            .oneshot(request("POST", "/v1/sandboxes", Some(&create_body())))
            .await
            .expect("response");
        assert_eq!(second.status(), StatusCode::CREATED);
        assert_eq!(body_json(second).await["sandbox_id"], id);
    }

    #[tokio::test]
    async fn status_logs_exec_and_delete() {
        let router = router(state());
        let created = router
            .clone()
            .oneshot(request("POST", "/v1/sandboxes", Some(&create_body())))
            .await
            .expect("response");
        let id = body_json(created).await["sandbox_id"]
            .as_str()
            .expect("id")
            .to_owned();
        let status = router
            .clone()
            .oneshot(request("GET", &format!("/v1/sandboxes/{id}"), None))
            .await
            .expect("response");
        assert_eq!(status.status(), StatusCode::OK);
        let logs = router
            .clone()
            .oneshot(request("GET", &format!("/v1/sandboxes/{id}/logs"), None))
            .await
            .expect("response");
        assert_eq!(logs.status(), StatusCode::OK);
        assert_eq!(
            logs.headers()
                .get("content-type")
                .and_then(|value| value.to_str().ok()),
            Some("text/event-stream")
        );
        let exec = router
            .clone()
            .oneshot(request(
                "POST",
                &format!("/v1/sandboxes/{id}/exec"),
                Some(r#"{"argv":["cargo","test"],"cwd":"workspace","timeout_s":60,"env_allowlist":[]}"#),
            ))
            .await
            .expect("response");
        assert_eq!(exec.status(), StatusCode::OK);
        let deleted = router
            .clone()
            .oneshot(request("DELETE", &format!("/v1/sandboxes/{id}"), None))
            .await
            .expect("response");
        assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
        let missing = router
            .oneshot(request("GET", &format!("/v1/sandboxes/{id}/logs"), None))
            .await
            .expect("response");
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn forbidden_commands_are_denied() {
        let router = router(state());
        let created = router
            .clone()
            .oneshot(request("POST", "/v1/sandboxes", Some(&create_body())))
            .await
            .expect("response");
        let id = body_json(created).await["sandbox_id"]
            .as_str()
            .expect("id")
            .to_owned();
        let exec = router
            .oneshot(request(
                "POST",
                &format!("/v1/sandboxes/{id}/exec"),
                Some(r#"{"argv":["sudo","rm","-rf","/"],"cwd":"workspace","timeout_s":60,"env_allowlist":[]}"#),
            ))
            .await
            .expect("response");
        assert_eq!(exec.status(), StatusCode::FORBIDDEN);
    }
}
