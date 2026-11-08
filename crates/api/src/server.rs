//! Axum edge router: health, runs, Problem Details, rate limits and WS.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use ailab_application::ProblemDetails;
use ailab_domain::{DomainError, RunId};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{
    EventEnvelope, Page, PageQuery, RateLimitKind, RateLimitPolicy, WsClientMessage, hello,
};

/// Run summary.
#[derive(Debug, Clone, Serialize)]
pub struct RunView {
    /// Run id.
    pub id: String,
    /// Card id.
    pub card_id: String,
    /// Lab id.
    pub lab_id: String,
    /// State.
    pub state: String,
    /// Attempt.
    pub attempt: u8,
    /// Started at.
    pub started_at: Option<DateTime<Utc>>,
    /// Ended at.
    pub ended_at: Option<DateTime<Utc>>,
    /// Cost so far.
    pub cost_microusd: i64,
}

/// Run step.
#[derive(Debug, Clone, Serialize)]
pub struct StepView {
    /// Sequence.
    pub seq: u32,
    /// Stage.
    pub stage: String,
    /// Kind.
    pub kind: String,
    /// Exit code.
    pub exit_code: Option<i32>,
    /// Output.
    pub output: String,
}

/// Store failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreError {
    /// Not found.
    NotFound,
    /// Invalid state for the operation.
    Conflict,
}

/// Run store port (the Postgres adapter is WP-13).
pub trait RunStore: Send + Sync {
    /// Read a run.
    fn get(&self, id: &str) -> Option<RunView>;
    /// Read the ordered steps.
    fn steps(&self, id: &str) -> Option<Vec<StepView>>;
    /// Cancel a queued/running run.
    fn cancel(&self, id: &str) -> Result<RunView, StoreError>;
    /// List runs for a lab.
    fn list_by_lab(&self, lab_id: &str, limit: u32, offset: usize) -> Page<RunView>;
    /// Enqueue a run for a card.
    fn enqueue_card(&self, card_id: &str) -> Result<RunView, StoreError>;
    /// Events after a sequence.
    fn events_after(&self, since: i64) -> Vec<EventEnvelope>;
}

#[derive(Default)]
struct StoreData {
    runs: BTreeMap<String, RunView>,
    steps: BTreeMap<String, Vec<StepView>>,
    events: Vec<EventEnvelope>,
}

/// In-memory store used by tests/dev.
#[derive(Default)]
pub struct InMemoryRunStore {
    data: Mutex<StoreData>,
}

impl InMemoryRunStore {
    /// New empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed a run (used by tests and the demo).
    pub fn insert(&self, run: RunView, steps: Vec<StepView>) {
        if let Ok(mut data) = self.data.lock() {
            data.steps.insert(run.id.clone(), steps);
            data.runs.insert(run.id.clone(), run);
        }
    }
}

impl RunStore for InMemoryRunStore {
    fn get(&self, id: &str) -> Option<RunView> {
        self.data.lock().ok()?.runs.get(id).cloned()
    }

    fn steps(&self, id: &str) -> Option<Vec<StepView>> {
        self.data.lock().ok()?.steps.get(id).cloned()
    }

    fn cancel(&self, id: &str) -> Result<RunView, StoreError> {
        let mut data = self.data.lock().map_err(|_| StoreError::Conflict)?;
        let run = data.runs.get_mut(id).ok_or(StoreError::NotFound)?;
        if run.state != "queued" && run.state != "running" {
            return Err(StoreError::Conflict);
        }
        "cancelled".clone_into(&mut run.state);
        run.ended_at = Some(Utc::now());
        let view = run.clone();
        drop(data);
        Ok(view)
    }

    fn list_by_lab(&self, lab_id: &str, limit: u32, offset: usize) -> Page<RunView> {
        let Some(data) = self.data.lock().ok() else {
            return Page {
                data: Vec::new(),
                next_cursor: None,
                total_estimate: 0,
            };
        };
        let all: Vec<RunView> = data
            .runs
            .values()
            .filter(|run| run.lab_id == lab_id)
            .cloned()
            .collect();
        let limit = usize::try_from(limit).unwrap_or(usize::MAX);
        let page: Vec<RunView> = all.iter().skip(offset).take(limit).cloned().collect();
        let next = offset + page.len();
        Page {
            data: page,
            next_cursor: (next < all.len()).then(|| next.to_string()),
            total_estimate: u64::try_from(all.len()).unwrap_or(u64::MAX),
        }
    }

    fn enqueue_card(&self, card_id: &str) -> Result<RunView, StoreError> {
        let mut data = self.data.lock().map_err(|_| StoreError::Conflict)?;
        let run = RunView {
            id: RunId::generate().as_uuid().to_string(),
            card_id: card_id.to_owned(),
            lab_id: "default".to_owned(),
            state: "queued".to_owned(),
            attempt: 1,
            started_at: None,
            ended_at: None,
            cost_microusd: 0,
        };
        data.runs.insert(run.id.clone(), run.clone());
        data.steps.insert(run.id.clone(), Vec::new());
        drop(data);
        Ok(run)
    }

    fn events_after(&self, since: i64) -> Vec<EventEnvelope> {
        let Some(data) = self.data.lock().ok() else {
            return Vec::new();
        };
        crate::events_after(&data.events, since)
    }
}

/// Rate limiter.
pub struct RateLimiter {
    policy: RateLimitPolicy,
    windows: Mutex<BTreeMap<String, (u32, Instant)>>,
}

impl RateLimiter {
    /// New limiter.
    #[must_use]
    pub const fn new(policy: RateLimitPolicy) -> Self {
        Self {
            policy,
            windows: Mutex::new(BTreeMap::new()),
        }
    }

    fn allow(&self, key: &str, kind: RateLimitKind) -> bool {
        let Ok(mut windows) = self.windows.lock() else {
            return false;
        };
        let entry = windows
            .entry(format!("{key}:{kind:?}"))
            .or_insert_with(|| (0, Instant::now()));
        if entry.1.elapsed().as_secs() >= 60 {
            *entry = (0, Instant::now());
        }
        if self.policy.allowed(kind, entry.0) {
            entry.0 += 1;
            true
        } else {
            false
        }
    }
}

/// Application state.
#[derive(Clone)]
pub struct AppState {
    /// Run store.
    pub store: Arc<dyn RunStore>,
    /// Rate limiter.
    pub limiter: Arc<RateLimiter>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    /// Build with the in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            store: Arc::new(InMemoryRunStore::new()),
            limiter: Arc::new(RateLimiter::new(RateLimitPolicy::default())),
        }
    }

    /// Build with a specific store.
    #[must_use]
    pub fn with_store(store: Arc<dyn RunStore>) -> Self {
        Self {
            store,
            limiter: Arc::new(RateLimiter::new(RateLimitPolicy::default())),
        }
    }
}

/// Build the edge router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/health/live", get(health_live))
        .route("/api/v1/health/ready", get(health_ready))
        .route("/api/v1/runs/{id}", get(get_run))
        .route("/api/v1/runs/{id}/steps", get(get_steps))
        .route("/api/v1/runs/{id}/cancel", post(cancel_run))
        .route("/api/v1/labs/{id}/runs", get(list_runs))
        .route("/api/v1/labs/{id}/metrics", get(get_metrics))
        .route("/api/v1/cards/{id}/runs", post(enqueue_run))
        .route("/ws", get(ws_upgrade))
        .with_state(state)
}

fn problem(error: DomainError) -> Response {
    let status = StatusCode::from_u16(error.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(ProblemDetails::from(error))).into_response()
}

fn rate_key(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|value| value.to_str().ok())
        .unwrap_or("local")
        .split(',')
        .next()
        .unwrap_or("local")
        .trim()
        .to_owned()
}

#[allow(clippy::result_large_err)]
fn enforce(state: &AppState, headers: &HeaderMap, kind: RateLimitKind) -> Result<(), Response> {
    if state.limiter.allow(&rate_key(headers), kind) {
        Ok(())
    } else {
        Err(problem(DomainError::policy(
            "RATE_LIMITED",
            "rate limit exceeded",
        )))
    }
}

async fn health_live() -> StatusCode {
    StatusCode::OK
}

async fn health_ready() -> StatusCode {
    StatusCode::OK
}

async fn get_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(response) = enforce(&state, &headers, RateLimitKind::Read) {
        return response;
    }
    state.store.get(&id).map_or_else(
        || problem(DomainError::NotFound),
        |run| Json(run).into_response(),
    )
}

async fn get_steps(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<PageQuery>,
) -> Response {
    if let Err(response) = enforce(&state, &headers, RateLimitKind::Read) {
        return response;
    }
    let limit = match query.normalize() {
        Ok(limit) => limit,
        Err(message) => return problem(DomainError::validation("limit", message)),
    };
    let Some(steps) = state.store.steps(&id) else {
        return problem(DomainError::NotFound);
    };
    let offset = query
        .cursor
        .and_then(|cursor| cursor.parse::<usize>().ok())
        .unwrap_or(0);
    let data: Vec<StepView> = steps
        .iter()
        .skip(offset)
        .take(usize::try_from(limit).unwrap_or(usize::MAX))
        .cloned()
        .collect();
    let next = offset + data.len();
    let total = u64::try_from(steps.len()).unwrap_or(u64::MAX);
    Json(Page {
        data,
        next_cursor: (next < steps.len()).then(|| next.to_string()),
        total_estimate: total,
    })
    .into_response()
}

async fn cancel_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(response) = enforce(&state, &headers, RateLimitKind::Write) {
        return response;
    }
    match state.store.cancel(&id) {
        Ok(run) => Json(run).into_response(),
        Err(StoreError::NotFound) => problem(DomainError::NotFound),
        Err(StoreError::Conflict) => problem(DomainError::policy(
            "INVALID_RUN_STATE",
            "the run cannot be cancelled in its current state",
        )),
    }
}

async fn list_runs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<PageQuery>,
) -> Response {
    if let Err(response) = enforce(&state, &headers, RateLimitKind::Read) {
        return response;
    }
    let limit = match query.normalize() {
        Ok(limit) => limit,
        Err(message) => return problem(DomainError::validation("limit", message)),
    };
    let offset = query
        .cursor
        .and_then(|cursor| cursor.parse::<usize>().ok())
        .unwrap_or(0);
    Json(state.store.list_by_lab(&id, limit, offset)).into_response()
}

async fn enqueue_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(response) = enforce(&state, &headers, RateLimitKind::Write) {
        return response;
    }
    match state.store.enqueue_card(&id) {
        Ok(run) => (StatusCode::CREATED, Json(run)).into_response(),
        Err(StoreError::NotFound) => problem(DomainError::NotFound),
        Err(StoreError::Conflict) => problem(DomainError::policy(
            "RUN_NOT_ALLOWED",
            "a run cannot be queued for this card",
        )),
    }
}

async fn get_metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(response) = enforce(&state, &headers, RateLimitKind::Read) {
        return response;
    }
    let runs = state.store.list_by_lab(&id, u32::MAX, 0).total_estimate;
    Json(serde_json::json!({ "lab_id": id, "runs": runs })).into_response()
}

async fn ws_upgrade(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let seq = state
        .store
        .events_after(0)
        .last()
        .map_or(0, |event| event.seq);
    let handshake = hello(Utc::now(), seq, vec!["system".to_owned()]);
    if let Ok(text) = serde_json::to_string(&handshake)
        && socket.send(Message::Text(text.into())).await.is_err()
    {
        return;
    }
    while let Some(Ok(message)) = socket.recv().await {
        match message {
            Message::Text(text) => {
                if let Ok(WsClientMessage::Subscribe { topics }) =
                    serde_json::from_str(text.as_str())
                {
                    let reply = hello(Utc::now(), seq, topics);
                    if let Ok(body) = serde_json::to_string(&reply) {
                        let _ = socket.send(Message::Text(body.into())).await;
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn seeded() -> AppState {
        let store = InMemoryRunStore::new();
        store.insert(
            RunView {
                id: "run-1".to_owned(),
                card_id: "card-1".to_owned(),
                lab_id: "lab-1".to_owned(),
                state: "queued".to_owned(),
                attempt: 1,
                started_at: None,
                ended_at: None,
                cost_microusd: 0,
            },
            vec![StepView {
                seq: 1,
                stage: "lease".to_owned(),
                kind: "tool".to_owned(),
                exit_code: Some(0),
                output: "ok".to_owned(),
            }],
        );
        AppState::with_store(Arc::new(store))
    }

    fn request(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .expect("request")
    }

    #[tokio::test]
    async fn health_and_run_endpoints() {
        let app = router(seeded());
        for uri in ["/api/v1/health/live", "/api/v1/health/ready"] {
            let response = app
                .clone()
                .oneshot(request("GET", uri))
                .await
                .expect("response");
            assert_eq!(response.status(), StatusCode::OK);
        }
        let run = app
            .clone()
            .oneshot(request("GET", "/api/v1/runs/run-1"))
            .await
            .expect("response");
        assert_eq!(run.status(), StatusCode::OK);
        let steps = app
            .clone()
            .oneshot(request("GET", "/api/v1/runs/run-1/steps"))
            .await
            .expect("response");
        assert_eq!(steps.status(), StatusCode::OK);
        let missing = app
            .oneshot(request("GET", "/api/v1/runs/missing"))
            .await
            .expect("response");
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn cancel_and_enqueue() {
        let app = router(seeded());
        let cancelled = app
            .clone()
            .oneshot(request("POST", "/api/v1/runs/run-1/cancel"))
            .await
            .expect("response");
        assert_eq!(cancelled.status(), StatusCode::OK);
        let enqueued = app
            .clone()
            .oneshot(request("POST", "/api/v1/cards/card-1/runs"))
            .await
            .expect("response");
        assert_eq!(enqueued.status(), StatusCode::CREATED);
        let runs = app
            .oneshot(request("GET", "/api/v1/labs/lab-1/runs"))
            .await
            .expect("response");
        assert_eq!(runs.status(), StatusCode::OK);
        let body = runs.into_body().collect().await.expect("body").to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert!(value["data"].is_array());
    }

    #[tokio::test]
    async fn problem_details_payload_is_rfc9457() {
        let app = router(seeded());
        let response = app
            .oneshot(request("GET", "/api/v1/runs/missing"))
            .await
            .expect("response");
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(value["status"], 404);
        assert_eq!(value["code"], "NOT_FOUND");
        assert!(value["type"].is_string());
    }
}
