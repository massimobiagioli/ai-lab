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
    /// Optional egress proxy URL.
    #[serde(default)]
    pub egress_proxy: Option<String>,
    /// Optional short-lived SCM token (write-only).
    #[serde(default)]
    pub scm_token: Option<String>,
    /// Acting role (`tech_lead` | `analyst` | `dev` | `qa`).
    #[serde(default)]
    pub role: Option<String>,
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
    /// Optional registry tool name (defaults to `shell.run`).
    #[serde(default)]
    pub tool: Option<String>,
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
    /// Denied rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    /// Denied tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    /// Denied argument.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argument: Option<String>,
}

/// API error.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
    rule: Option<String>,
    tool: Option<String>,
    argument: Option<String>,
}

impl ApiError {
    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "UNAUTHORIZED",
            message: "missing or invalid runner token".to_owned(),
            rule: None,
            tool: None,
            argument: None,
        }
    }

    fn validation(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "VALIDATION_FAILED",
            message: message.into(),
            rule: None,
            tool: None,
            argument: None,
        }
    }

    fn policy(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: "POLICY_DENIED",
            message: message.into(),
            rule: None,
            tool: None,
            argument: None,
        }
    }

    fn policy_denial(denial: crate::tools::ToolDenial) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: "POLICY_DENIED",
            message: denial.reason,
            rule: Some(denial.rule),
            tool: Some(denial.tool),
            argument: Some(denial.argument),
        }
    }

    fn unsupported_tool(tool: &str) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "UNSUPPORTED_TOOL",
            message: format!("tool `{tool}` has no runner executor"),
            rule: None,
            tool: None,
            argument: None,
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "INTERNAL",
            message: message.into(),
            rule: None,
            tool: None,
            argument: None,
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
                rule: self.rule,
                tool: self.tool,
                argument: self.argument,
            }),
        )
            .into_response()
    }
}

impl From<BackendError> for ApiError {
    fn from(error: BackendError) -> Self {
        match error {
            BackendError::Validation(message) => Self::validation(message),
            BackendError::Policy(message) => Self::policy(message),
            BackendError::NotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "NOT_FOUND",
                message: "sandbox not found".to_owned(),
                rule: None,
                tool: None,
                argument: None,
            },
            BackendError::Timeout => Self {
                status: StatusCode::GATEWAY_TIMEOUT,
                code: "TOOL_TIMEOUT",
                message: "tool execution timed out".to_owned(),
                rule: None,
                tool: None,
                argument: None,
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
    roles: Arc<std::sync::Mutex<std::collections::BTreeMap<String, AgentRole>>>,
    denials: Arc<std::sync::Mutex<Vec<AuditDenial>>>,
}

/// One recorded policy denial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditDenial {
    /// Sandbox.
    pub sandbox_id: String,
    /// Rule.
    pub rule: String,
    /// Tool.
    pub tool: String,
    /// Argument.
    pub argument: String,
    /// Reason.
    pub reason: String,
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
        Self {
            backend,
            token,
            roles: Arc::new(std::sync::Mutex::new(std::collections::BTreeMap::new())),
            denials: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    fn set_role(&self, sandbox_id: &str, role: AgentRole) {
        if let Ok(mut roles) = self.roles.lock() {
            roles.insert(sandbox_id.to_owned(), role);
        }
    }

    fn role_of(&self, sandbox_id: &str) -> Option<AgentRole> {
        self.roles
            .lock()
            .ok()
            .and_then(|roles| roles.get(sandbox_id).copied())
    }

    fn record_denial(&self, denial: &crate::tools::ToolDenial, sandbox_id: &str) {
        if let Ok(mut denials) = self.denials.lock() {
            denials.push(AuditDenial {
                sandbox_id: sandbox_id.to_owned(),
                rule: denial.rule.clone(),
                tool: denial.tool.clone(),
                argument: denial.argument.clone(),
                reason: denial.reason.clone(),
            });
        }
    }

    /// Recorded policy denials (audit sink).
    #[must_use]
    pub fn denials(&self) -> Vec<AuditDenial> {
        self.denials
            .lock()
            .map(|denials| denials.clone())
            .unwrap_or_default()
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
    let mut spec = SandboxSpec::new(
        request.run_id,
        request.image,
        request.worktree_ref,
        request.cpu,
        request.memory_mb,
        request.timeout_s,
        request.egress,
    )
    .map_err(|error| ApiError::validation(error.to_string()))?;
    if let Some(proxy) = request.egress_proxy {
        spec = spec.with_egress_proxy(proxy);
    }
    if let Some(token) = request.scm_token {
        spec = spec.with_scm_token(ailab_application::Secret::new(token));
    }
    let role = parse_role(request.role.as_deref());
    let created = state
        .backend
        .create(&spec, role)
        .await
        .map_err(ApiError::from)?;
    state.set_role(&created.sandbox_id, role);
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
    let tool = request.tool.as_deref().unwrap_or("shell.run");
    if tool != "shell.run" {
        return Err(ApiError::unsupported_tool(tool));
    }
    let role = state
        .role_of(&id)
        .ok_or_else(|| ApiError::policy("unknown sandbox role"))?;
    let registry = crate::tools::ToolRegistry::v1();
    if let Err(denial) = registry.authorize(
        role,
        "shell.run",
        &serde_json::json!({"argv": request.argv, "path": request.cwd}),
    ) {
        state.record_denial(&denial, &id);
        return Err(ApiError::policy_denial(denial));
    }
    let timeout = request
        .timeout_s
        .min(registry.get("shell.run").map_or(600, |spec| spec.timeout_s));
    let call = ToolCall {
        argv: request.argv,
        cwd: request.cwd,
        writes: Vec::new(),
    };
    let mut outcome = state
        .backend
        .exec(&id, &call, timeout)
        .await
        .map_err(ApiError::from)?;
    outcome.duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    Ok(Json(outcome))
}

fn parse_role(value: Option<&str>) -> AgentRole {
    match value.unwrap_or("dev") {
        "tech_lead" | "techlead" => AgentRole::TechLead,
        "analyst" => AgentRole::Analyst,
        "qa" => AgentRole::Qa,
        _ => AgentRole::Dev,
    }
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
            "memory_mb": 1024,
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
        let state = state();
        let router = router(state.clone());
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
    async fn exec_enforces_role_and_tool_mapping() {
        let state = state();
        let router = router(state.clone());
        let created = router
            .clone()
            .oneshot(request("POST", "/v1/sandboxes", Some(&create_body())))
            .await
            .expect("response");
        let id = body_json(created).await["sandbox_id"]
            .as_str()
            .expect("id")
            .to_owned();

        // A non-shell tool must not smuggle arbitrary argv through.
        let bypass = router
            .clone()
            .oneshot(request(
                "POST",
                &format!("/v1/sandboxes/{id}/exec"),
                Some(r#"{"argv":["curl","https://example.test"],"cwd":"workspace","timeout_s":10,"tool":"git.status"}"#),
            ))
            .await
            .expect("response");
        assert_eq!(bypass.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let curl = router
            .clone()
            .oneshot(request(
                "POST",
                &format!("/v1/sandboxes/{id}/exec"),
                Some(
                    r#"{"argv":["curl","https://example.test"],"cwd":"workspace","timeout_s":10}"#,
                ),
            ))
            .await
            .expect("response");
        assert_eq!(curl.status(), StatusCode::FORBIDDEN);

        // An analyst may not run shell commands.
        let analyst_body = serde_json::json!({
            "run_id": "run-2",
            "image": "ailab/runner@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "worktree_ref": "worktrees/run-2",
            "cpu": 1000,
            "memory_mb": 1024,
            "timeout_s": 900,
            "egress": {"Allowlist": ["crates.io"]},
            "role": "analyst"
        })
        .to_string();
        let analyst = router
            .clone()
            .oneshot(request("POST", "/v1/sandboxes", Some(&analyst_body)))
            .await
            .expect("response");
        let analyst_id = body_json(analyst).await["sandbox_id"]
            .as_str()
            .expect("id")
            .to_owned();
        assert_eq!(state.role_of(&analyst_id), Some(AgentRole::Analyst));
        let denied = router
            .oneshot(request(
                "POST",
                &format!("/v1/sandboxes/{analyst_id}/exec"),
                Some(r#"{"argv":["cargo","test"],"cwd":"workspace","timeout_s":10}"#),
            ))
            .await
            .expect("response");
        assert_eq!(denied.status(), StatusCode::FORBIDDEN);
        let body = denied.into_body().collect().await.expect("body").to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(value["code"], "POLICY_DENIED");
        assert_eq!(value["rule"], "ROLE_NOT_ALLOWED");
        assert!(
            state
                .denials()
                .iter()
                .any(|denial| denial.rule == "ROLE_NOT_ALLOWED")
        );
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
