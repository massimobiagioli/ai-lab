//! Axum edge router: health, runs, Problem Details, rate limits and WS.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ailab_application::{ProblemDetails, Secret};
use ailab_domain::{DomainError, RunId};
use async_trait::async_trait;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::time::interval;

use crate::{
    EventEnvelope, Page, PageQuery, RateLimitKind, RateLimitPolicy, WsClientMessage,
    WsServerMessage, hello,
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

/// Run store port (in-memory or Postgres).
#[async_trait]
pub trait RunStore: Send + Sync {
    /// Readiness probe (dependency check).
    async fn ready(&self) -> bool {
        true
    }
    /// Read a run.
    async fn get(&self, id: &str) -> Option<RunView>;
    /// Read the ordered steps.
    async fn steps(&self, id: &str) -> Option<Vec<StepView>>;
    /// Cancel a queued/running run.
    async fn cancel(&self, id: &str) -> Result<RunView, StoreError>;
    /// List runs for a lab.
    async fn list_by_lab(&self, lab_id: &str, limit: u32, offset: usize) -> Page<RunView>;
    /// Enqueue a run for a card.
    async fn enqueue_card(&self, card_id: &str) -> Result<RunView, StoreError>;
    /// Events after a sequence.
    async fn events_after(&self, lab_id: &str, since: i64) -> Vec<EventEnvelope>;
    /// Append an event to the backfill buffer.
    async fn append_event(&self, event: EventEnvelope);
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

#[async_trait]
impl RunStore for InMemoryRunStore {
    async fn get(&self, id: &str) -> Option<RunView> {
        self.data.lock().ok()?.runs.get(id).cloned()
    }

    async fn steps(&self, id: &str) -> Option<Vec<StepView>> {
        self.data.lock().ok()?.steps.get(id).cloned()
    }

    async fn cancel(&self, id: &str) -> Result<RunView, StoreError> {
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

    async fn list_by_lab(&self, lab_id: &str, limit: u32, offset: usize) -> Page<RunView> {
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

    async fn enqueue_card(&self, card_id: &str) -> Result<RunView, StoreError> {
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

    async fn events_after(&self, lab_id: &str, since: i64) -> Vec<EventEnvelope> {
        let Some(data) = self.data.lock().ok() else {
            return Vec::new();
        };
        crate::events_after(&data.events, since)
            .into_iter()
            .filter(|event| event.lab_id.to_string() == lab_id)
            .collect()
    }

    async fn append_event(&self, event: EventEnvelope) {
        if let Ok(mut data) = self.data.lock()
            && !data
                .events
                .iter()
                .any(|existing| existing.event_id == event.event_id)
        {
            data.events.push(event);
        }
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

struct StoredResponse {
    status: u16,
    body: serde_json::Value,
    at: Instant,
}

#[derive(Default)]
struct IdempotencyStore {
    entries: Mutex<BTreeMap<String, StoredResponse>>,
    in_flight: Mutex<std::collections::BTreeSet<String>>,
}

impl IdempotencyStore {
    fn get(&self, key: &str) -> Option<(u16, serde_json::Value)> {
        let mut entries = self.entries.lock().ok()?;
        entries.retain(|_, response| response.at.elapsed() < Duration::from_hours(24));
        entries
            .get(key)
            .map(|response| (response.status, response.body.clone()))
    }

    fn try_start(&self, key: &str) -> bool {
        self.in_flight
            .lock()
            .is_ok_and(|mut in_flight| in_flight.insert(key.to_owned()))
    }

    fn finish(&self, key: &str) {
        if let Ok(mut in_flight) = self.in_flight.lock() {
            in_flight.remove(key);
        }
    }

    fn put(&self, key: &str, status: u16, body: serde_json::Value) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.insert(
                key.to_owned(),
                StoredResponse {
                    status,
                    body,
                    at: Instant::now(),
                },
            );
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
    auth: Option<Secret<String>>,
    idempotency: Arc<IdempotencyStore>,
    hub: broadcast::Sender<EventEnvelope>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    /// Build with the Postgres store when `AILAB_DATABASE_URL` is set, else the
    /// in-memory store, and the environment token.
    #[must_use]
    pub fn new() -> Self {
        let token = std::env::var("AILAB_API_TOKEN")
            .ok()
            .filter(|value| !value.is_empty());
        if let Ok(url) = std::env::var("AILAB_DATABASE_URL")
            && !url.is_empty()
            && let Ok(pool) = sqlx::postgres::PgPoolOptions::new()
                .max_connections(10)
                .connect_lazy(&url)
        {
            return Self::with_store_and_token(
                Arc::new(crate::postgres_store::PostgresRunStore::new(pool)),
                token,
            );
        }
        Self::with_store_and_token(Arc::new(InMemoryRunStore::new()), token)
    }

    /// Build with a specific store.
    #[must_use]
    pub fn with_store(store: Arc<dyn RunStore>) -> Self {
        Self::with_store_and_token(store, None)
    }

    /// Build with a specific store and optional bearer token.
    #[must_use]
    pub fn with_store_and_token(store: Arc<dyn RunStore>, token: Option<String>) -> Self {
        let (hub, _) = broadcast::channel(1_024);
        Self {
            store,
            limiter: Arc::new(RateLimiter::new(RateLimitPolicy::default())),
            auth: token.map(Secret::new),
            idempotency: Arc::new(IdempotencyStore::default()),
            hub,
        }
    }

    /// Publish an event to live WebSocket subscribers.
    pub fn publish(&self, event: EventEnvelope) -> usize {
        self.hub.send(event).unwrap_or(0)
    }

    /// Subscribe to live events.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.hub.subscribe()
    }

    /// Persist an event for backfill and broadcast it to live subscribers.
    pub async fn ingest(&self, event: EventEnvelope) -> usize {
        self.store.append_event(event.clone()).await;
        self.hub.send(event).unwrap_or(0)
    }
}

/// Drain the outbox and publish its events to the API hub and backfill store.
pub async fn relay_once(
    outbox: &dyn ailab_application::Outbox,
    state: &AppState,
) -> Result<usize, String> {
    let pending = outbox.pending().await.map_err(|error| error.to_string())?;
    let mut published = 0;
    for event in &pending {
        state
            .ingest(EventEnvelope {
                event_id: event.event_id,
                event: event.event_type.clone(),
                seq: event.seq.unwrap_or(0),
                lab_id: event.lab_id,
                occurred_at: event.occurred_at,
                payload: event.payload.clone(),
            })
            .await;
        outbox
            .acknowledge(event.event_id.as_uuid())
            .await
            .map_err(|error| error.to_string())?;
        published += 1;
    }
    Ok(published)
}

/// Build the edge router.
pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/api/v1/runs/{id}", get(get_run))
        .route("/api/v1/runs/{id}/steps", get(get_steps))
        .route("/api/v1/runs/{id}/cancel", post(cancel_run))
        .route("/api/v1/labs/{id}/runs", get(list_runs))
        .route("/api/v1/labs/{id}/metrics", get(get_metrics))
        .route("/api/v1/cards/{id}/runs", post(enqueue_run))
        .route("/api/v1/events", get(list_events))
        .route("/api/v1/openapi.json", get(openapi))
        .route("/api/v1/telemetry", post(ingest_telemetry))
        .route("/api/v1/telemetry/frontend", post(ingest_telemetry))
        .route("/api/v1/providers", get(not_implemented))
        .route("/api/v1/providers/{id}", get(not_implemented))
        .route("/api/v1/models", get(not_implemented))
        .route("/api/v1/projects", get(not_implemented))
        .route("/api/v1/projects/{id}/board", get(not_implemented))
        .route("/api/v1/cards/{id}", get(not_implemented))
        .route("/api/v1/cards/{id}/transition", post(not_implemented))
        .route("/api/v1/health/providers", get(health_providers))
        .route("/api/v1/health/scm", get(health_scm))
        .route("/ws", get(ws_upgrade))
        .route("/api/v1/ws", get(ws_upgrade))
        .layer(middleware::from_fn_with_state(state.clone(), auth));
    Router::new()
        .route("/api/v1/health/live", get(health_live))
        .route("/api/v1/health/ready", get(health_ready))
        .merge(protected)
        .fallback(not_found)
        .layer(axum::extract::DefaultBodyLimit::max(1_048_576))
        .layer(middleware::from_fn(request_id))
        .with_state(state)
}

async fn request_id(mut request: Request, next: Next) -> Response {
    let id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map_or_else(
            || {
                format!(
                    "{:x}",
                    chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
                )
            },
            ToOwned::to_owned,
        );
    request.extensions_mut().insert(RequestId(id.clone()));
    let mut response = next.run(request).await;
    if let Ok(value) = axum::http::HeaderValue::from_str(&id) {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

/// Request correlation id.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

async fn auth(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let Some(expected) = state.auth.as_ref() else {
        return next.run(request).await;
    };
    let header = request
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    let query = request.uri().query().and_then(|query| {
        query
            .split('&')
            .find_map(|pair| pair.strip_prefix("token="))
    });
    if ailab_infrastructure::exposure::token_matches(expected.expose(), header.or(query)) {
        next.run(request).await
    } else {
        unauthorized()
    }
}

async fn not_found() -> Response {
    let mut details = ProblemDetails::from(DomainError::NotFound);
    details.trace_id = Some(format!(
        "{:x}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    details.instance = Some("about:blank".to_owned());
    (
        StatusCode::NOT_FOUND,
        [(axum::http::header::CONTENT_TYPE, "application/problem+json")],
        Json(details),
    )
        .into_response()
}

fn unauthorized() -> Response {
    let mut problem = ProblemDetails::from(DomainError::policy(
        "UNAUTHORIZED",
        "missing or invalid API token",
    ));
    problem.status = 401;
    problem.trace_id = Some(format!(
        "{:x}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    problem.instance = Some("about:blank".to_owned());
    (
        StatusCode::UNAUTHORIZED,
        [
            (axum::http::header::CONTENT_TYPE, "application/problem+json"),
            (axum::http::header::WWW_AUTHENTICATE, "Bearer"),
        ],
        Json(problem),
    )
        .into_response()
}

fn idempotency_key(headers: &HeaderMap, method: &str, path: &str) -> Option<String> {
    let raw = headers.get("idempotency-key")?.to_str().ok()?.trim();
    if !(8..=128).contains(&raw.len()) {
        return None;
    }
    Some(format!("{method}:{path}:{raw}"))
}

fn replay(status: u16, body: serde_json::Value) -> Response {
    let status = StatusCode::from_u16(status).unwrap_or(StatusCode::OK);
    (
        status,
        [(
            axum::http::header::HeaderName::from_static("idempotent-replay"),
            "true",
        )],
        Json(body),
    )
        .into_response()
}

fn problem(error: DomainError) -> Response {
    let status = StatusCode::from_u16(error.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut details = ProblemDetails::from(error);
    details.trace_id = Some(format!(
        "{:x}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    details.instance = Some("about:blank".to_owned());
    (
        status,
        [(axum::http::header::CONTENT_TYPE, "application/problem+json")],
        Json(details),
    )
        .into_response()
}

const B64_ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn b64encode(input: &[u8]) -> String {
    let mut output = String::new();
    for chunk in input.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        let chars = [
            B64_ALPHABET[((n >> 18) & 63) as usize],
            B64_ALPHABET[((n >> 12) & 63) as usize],
            B64_ALPHABET[((n >> 6) & 63) as usize],
            B64_ALPHABET[(n & 63) as usize],
        ];
        output.push(chars[0] as char);
        output.push(chars[1] as char);
        if chunk.len() > 1 {
            output.push(chars[2] as char);
        }
        if chunk.len() > 2 {
            output.push(chars[3] as char);
        }
    }
    output
}

fn b64decode(input: &str) -> Option<Vec<u8>> {
    let mut values = Vec::new();
    for byte in input.bytes() {
        let value = B64_ALPHABET
            .iter()
            .position(|&candidate| candidate == byte)?;
        values.push(u8::try_from(value).ok()?);
    }
    let mut output = Vec::new();
    for chunk in values.chunks(4) {
        if chunk.len() < 2 {
            return None;
        }
        let mut n = 0_u32;
        for (index, value) in chunk.iter().enumerate() {
            n |= u32::from(*value) << (18 - 6 * index);
        }
        output.push(u8::try_from((n >> 16) & 0xff).ok()?);
        if chunk.len() > 2 {
            output.push(u8::try_from((n >> 8) & 0xff).ok()?);
        }
        if chunk.len() > 3 {
            output.push(u8::try_from(n & 0xff).ok()?);
        }
    }
    Some(output)
}

fn encode_cursor(offset: usize) -> String {
    b64encode(offset.to_string().as_bytes())
}

fn decode_cursor(cursor: &str) -> Option<usize> {
    std::str::from_utf8(&b64decode(cursor)?).ok()?.parse().ok()
}

fn rate_key(headers: &HeaderMap) -> String {
    if std::env::var("AILAB_TRUSTED_PROXY").is_err() {
        return "local".to_owned();
    }
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
        let mut details =
            ProblemDetails::from(DomainError::policy("RATE_LIMITED", "rate limit exceeded"));
        details.status = 429;
        details.trace_id = Some(format!(
            "{:x}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        details.instance = Some("about:blank".to_owned());
        Err((
            StatusCode::TOO_MANY_REQUESTS,
            [
                (axum::http::header::RETRY_AFTER, "60"),
                (axum::http::header::CONTENT_TYPE, "application/problem+json"),
            ],
            Json(details),
        )
            .into_response())
    }
}

async fn health_live() -> StatusCode {
    StatusCode::OK
}

async fn not_implemented() -> Response {
    let mut details = ProblemDetails::from(DomainError::validation(
        "endpoint",
        "this endpoint is not implemented yet",
    ));
    details.status = 501;
    "NOT_IMPLEMENTED".clone_into(&mut details.code);
    details.trace_id = Some(format!(
        "{:x}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    details.instance = Some("about:blank".to_owned());
    (
        StatusCode::NOT_IMPLEMENTED,
        [(axum::http::header::CONTENT_TYPE, "application/problem+json")],
        Json(details),
    )
        .into_response()
}

async fn health_providers() -> Response {
    Json(serde_json::json!({"status": "ok", "providers": []})).into_response()
}

async fn health_scm() -> Response {
    Json(serde_json::json!({"status": "ok", "scm": "github"})).into_response()
}

async fn health_ready(State(state): State<AppState>) -> Response {
    if state.store.ready().await {
        StatusCode::OK.into_response()
    } else {
        let mut details = ProblemDetails::from(DomainError::policy(
            "NOT_READY",
            "a required dependency is unavailable",
        ));
        details.status = 503;
        details.trace_id = Some(format!(
            "{:x}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        details.instance = Some("about:blank".to_owned());
        (
            StatusCode::SERVICE_UNAVAILABLE,
            [(axum::http::header::CONTENT_TYPE, "application/problem+json")],
            Json(details),
        )
            .into_response()
    }
}

async fn get_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(response) = enforce(&state, &headers, RateLimitKind::Read) {
        return response;
    }
    state.store.get(&id).await.map_or_else(
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
    let Some(steps) = state.store.steps(&id).await else {
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
    let path = format!("/runs/{id}/cancel");
    let key = idempotency_key(&headers, "POST", &path);
    if let Some(key) = key.as_deref() {
        if let Some((status, body)) = state.idempotency.get(key) {
            return replay(status, body);
        }
        if !state.idempotency.try_start(key) {
            return problem(DomainError::policy(
                "IDEMPOTENCY_IN_PROGRESS",
                "a request with this key is in progress",
            ));
        }
    }
    match state.store.cancel(&id).await {
        Ok(run) => {
            let body = serde_json::to_value(&run).unwrap_or(serde_json::Value::Null);
            if let Some(key) = key.as_deref() {
                state.idempotency.put(key, 200, body.clone());
                state.idempotency.finish(key);
            }
            Json(body).into_response()
        }
        Err(StoreError::NotFound) => {
            if let Some(key) = key.as_deref() {
                state.idempotency.finish(key);
            }
            problem(DomainError::NotFound)
        }
        Err(StoreError::Conflict) => {
            if let Some(key) = key.as_deref() {
                state.idempotency.finish(key);
            }
            problem(DomainError::policy(
                "INVALID_RUN_STATE",
                "the run cannot be cancelled in its current state",
            ))
        }
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
    let offset = match query.cursor.as_deref() {
        Some(cursor) => match decode_cursor(cursor) {
            Some(offset) => offset,
            None => {
                return problem(DomainError::validation("cursor", "invalid cursor"));
            }
        },
        None => 0,
    };
    let mut page = state.store.list_by_lab(&id, limit, offset).await;
    page.next_cursor = page
        .next_cursor
        .map(|_| encode_cursor(offset + usize::try_from(limit).unwrap_or(0)));
    Json(page).into_response()
}

async fn enqueue_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(response) = enforce(&state, &headers, RateLimitKind::Write) {
        return response;
    }
    let path = format!("/cards/{id}/runs");
    let key = idempotency_key(&headers, "POST", &path);
    if let Some(key) = key.as_deref() {
        if let Some((status, body)) = state.idempotency.get(key) {
            return replay(status, body);
        }
        if !state.idempotency.try_start(key) {
            return problem(DomainError::policy(
                "IDEMPOTENCY_IN_PROGRESS",
                "a request with this key is in progress",
            ));
        }
    }
    match state.store.enqueue_card(&id).await {
        Ok(run) => {
            let body = serde_json::to_value(&run).unwrap_or(serde_json::Value::Null);
            if let Some(key) = key.as_deref() {
                state.idempotency.put(key, 201, body.clone());
                state.idempotency.finish(key);
            }
            (StatusCode::ACCEPTED, Json(body)).into_response()
        }
        Err(StoreError::NotFound) => {
            if let Some(key) = key.as_deref() {
                state.idempotency.finish(key);
            }
            problem(DomainError::NotFound)
        }
        Err(StoreError::Conflict) => {
            if let Some(key) = key.as_deref() {
                state.idempotency.finish(key);
            }
            problem(DomainError::policy(
                "RUN_NOT_ALLOWED",
                "a run cannot be queued for this card",
            ))
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct WsQuery {
    #[serde(default)]
    lab_id: String,
    #[serde(default)]
    since: i64,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct EventsQuery {
    #[serde(default)]
    lab_id: String,
    #[serde(default)]
    since: i64,
    #[serde(default)]
    after: Option<i64>,
    #[serde(default)]
    limit: Option<u32>,
}

/// The generated `OpenAPI` document.
const OPENAPI: &str = include_str!("../../../.docs/product/api/openapi.json");

async fn openapi(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(response) = enforce(&state, &headers, RateLimitKind::Read) {
        return response;
    }
    (
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        OPENAPI,
    )
        .into_response()
}

/// Frontend telemetry payload.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryEvent {
    /// Level.
    pub level: String,
    /// Message.
    pub message: String,
    /// Stack trace.
    #[serde(default)]
    pub stack: Option<String>,
    /// Route.
    pub route: String,
    /// Build.
    pub build: String,
    /// Trace id.
    #[serde(default)]
    pub trace_id: Option<String>,
}

async fn ingest_telemetry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<TelemetryEvent>,
) -> Response {
    if let Err(response) = enforce(&state, &headers, RateLimitKind::Write) {
        return response;
    }
    if !matches!(body.level.as_str(), "debug" | "info" | "warn" | "error") {
        return problem(DomainError::validation(
            "telemetry.level",
            "must be debug, info, warn or error",
        ));
    }
    if body.message.is_empty() || body.message.len() > 1_024 {
        return problem(DomainError::validation(
            "telemetry.message",
            "must be 1-1024 characters",
        ));
    }
    if body.route.is_empty() || body.build.is_empty() {
        return problem(DomainError::validation(
            "telemetry.route",
            "route and build are required",
        ));
    }
    StatusCode::ACCEPTED.into_response()
}

async fn list_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<EventsQuery>,
) -> Response {
    if let Err(response) = enforce(&state, &headers, RateLimitKind::Read) {
        return response;
    }
    let lab_id = if query.lab_id.is_empty() {
        "default"
    } else {
        &query.lab_id
    };
    let after = query.after.unwrap_or(query.since);
    let limit = query.limit.unwrap_or(200).clamp(1, 1_000) as usize;
    let mut events = state.store.events_after(lab_id, after).await;
    events.truncate(limit);
    Json(events).into_response()
}

async fn get_metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(response) = enforce(&state, &headers, RateLimitKind::Read) {
        return response;
    }
    let runs = state
        .store
        .list_by_lab(&id, u32::MAX, 0)
        .await
        .total_estimate;
    Json(serde_json::json!({ "lab_id": id, "runs": runs })).into_response()
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, query))
}

#[allow(clippy::too_many_lines)]
async fn handle_socket(mut socket: WebSocket, state: AppState, query: WsQuery) {
    let lab_id = if query.lab_id.is_empty() {
        "default".to_owned()
    } else {
        query.lab_id
    };
    let backfill = state.store.events_after(&lab_id, query.since).await;
    let seq = backfill.last().map_or(query.since, |event| event.seq);
    for event in &backfill {
        let message = WsServerMessage::event(event);
        let Ok(body) = serde_json::to_string(&message) else {
            continue;
        };
        if socket.send(Message::Text(body.into())).await.is_err() {
            return;
        }
    }
    let mut topics: Vec<String> = vec!["system".to_owned()];
    let handshake = hello(Utc::now(), seq, topics.clone());
    if let Ok(text) = serde_json::to_string(&handshake)
        && socket.send(Message::Text(text.into())).await.is_err()
    {
        return;
    }
    let mut events = state.subscribe();
    let mut heartbeat = interval(Duration::from_secs(10));
    let mut last_seen = Instant::now();
    loop {
        tokio::select! {
            message = socket.recv() => {
                match message {
                    Some(Ok(Message::Text(text))) => {
                        last_seen = Instant::now();
                        match serde_json::from_str::<WsClientMessage>(text.as_str()) {
                            Ok(WsClientMessage::Subscribe {
                                topics: subscribed,
                            }) => {
                                topics.clone_from(&subscribed);
                                if let Ok(body) = serde_json::to_string(&WsServerMessage::Ack {
                                    topics: subscribed,
                                }) {
                                    let _ = socket.send(Message::Text(body.into())).await;
                                }
                            }
                            Ok(WsClientMessage::Unsubscribe {
                                topics: removed,
                            }) => {
                                topics.retain(|topic| !removed.contains(topic));
                            }
                            Ok(WsClientMessage::Ping { ts }) => {
                                if let Ok(body) =
                                    serde_json::to_string(&WsServerMessage::Pong { ts })
                                {
                                    let _ = socket.send(Message::Text(body.into())).await;
                                }
                            }
                            Ok(WsClientMessage::Ack { .. }) => {}
                            Err(_) => {
                                let error = WsServerMessage::Error {
                                    code: "BAD_MESSAGE".to_owned(),
                                    detail: "invalid websocket message".to_owned(),
                                };
                                if let Ok(body) = serde_json::to_string(&error) {
                                    let _ = socket.send(Message::Text(body.into())).await;
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Pong(_))) => last_seen = Instant::now(),
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            event = events.recv() => {
                match event {
                    Ok(event) => {
                        let mut candidates = vec![
                            if event.event.starts_with("delivery.run") {
                                "run".to_owned()
                            } else if event.event.starts_with("kanban") {
                                "kanban".to_owned()
                            } else {
                                "system".to_owned()
                            },
                            format!("lab:{}", event.lab_id),
                        ];
                        if let Some(run_id) = event.payload.get("run_id").and_then(|v| v.as_str()) {
                            candidates.push(format!("run:{run_id}"));
                        }
                        if let Some(card_id) =
                            event.payload.get("card_id").and_then(|v| v.as_str())
                        {
                            candidates.push(format!("card:{card_id}"));
                        }
                        if !topics.iter().any(|subscribed| candidates.contains(subscribed)) {
                            continue;
                        }
                        let message = WsServerMessage::event(&event);
                        if let Ok(body) = serde_json::to_string(&message)
                            && socket.send(Message::Text(body.into())).await.is_err()
                        {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_skipped)) => {
                        let resync = WsServerMessage::Resync { after: seq };
                        if let Ok(body) = serde_json::to_string(&resync) {
                            let _ = socket.send(Message::Text(body.into())).await;
                        }
                        let _ = socket
                            .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                                code: 1013,
                                reason: "resync".into(),
                            })))
                            .await;
                        break;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            _ = heartbeat.tick() => {
                if last_seen.elapsed() > Duration::from_secs(25) {
                    let _ = socket
                        .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                            code: 1013,
                            reason: "heartbeat timeout".into(),
                        })))
                        .await;
                    break;
                }
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ailab_application::Outbox;
    use ailab_domain::{Actor, DomainEvent, LabId};
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
        request_with(method, uri, &[])
    }

    fn request_with(method: &str, uri: &str, headers: &[(&str, &str)]) -> Request<Body> {
        let mut builder = Request::builder().method(method).uri(uri);
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        builder.body(Body::empty()).expect("request")
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
        assert_eq!(enqueued.status(), StatusCode::ACCEPTED);
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
    async fn openapi_and_telemetry_endpoints() {
        let app = router(seeded());
        let spec = app
            .clone()
            .oneshot(request("GET", "/api/v1/openapi.json"))
            .await
            .expect("response");
        assert_eq!(spec.status(), StatusCode::OK);
        let body = spec.into_body().collect().await.expect("body").to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(value["openapi"], "3.1.0");
        let telemetry = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/telemetry")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"level":"info","message":"ui.render","route":"/lab","build":"1.0.0"}"#,
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(telemetry.status(), StatusCode::ACCEPTED);
        let invalid = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/telemetry")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"level":"info","message":"","route":"/lab","build":"1.0.0"}"#,
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(invalid.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn relay_drains_the_outbox_and_backfills_events() {
        let state = AppState::with_store(Arc::new(InMemoryRunStore::new()));
        let outbox = ailab_infrastructure::outbox::InMemoryOutbox::new();
        let lab_id = LabId::generate();
        let event = DomainEvent::new(
            "kanban.card.moved",
            lab_id,
            Actor::system(),
            serde_json::json!({"x": 1}),
            Utc::now(),
        )
        .expect("event");
        outbox.append(&[event]).await.expect("append");
        let mut rx = state.subscribe();
        let published = relay_once(&outbox, &state).await.expect("relay");
        assert_eq!(published, 1);
        let received = rx.try_recv().expect("live event");
        assert_eq!(received.event, "kanban.card.moved");
        assert!(outbox.pending().await.expect("pending").is_empty());
        let app = router(state);
        let response = app
            .oneshot(request(
                "GET",
                &format!("/api/v1/events?lab_id={lab_id}&since=0"),
            ))
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(value.as_array().map(Vec::len), Some(1));
    }

    #[tokio::test]
    async fn auth_is_required_when_configured() {
        let store = InMemoryRunStore::new();
        let token = "x".repeat(32);
        let state = AppState::with_store_and_token(Arc::new(store), Some(token.clone()));
        let app = router(state);
        let unauthenticated = app
            .clone()
            .oneshot(request("GET", "/api/v1/runs/run-1"))
            .await
            .expect("response");
        assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);
        let health = app
            .clone()
            .oneshot(request("GET", "/api/v1/health/live"))
            .await
            .expect("response");
        assert_eq!(health.status(), StatusCode::OK);
        let authenticated = app
            .oneshot(request_with(
                "GET",
                "/api/v1/runs/run-1",
                &[("authorization", &format!("Bearer {token}"))],
            ))
            .await
            .expect("response");
        assert_eq!(authenticated.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn idempotency_key_replays_the_first_response() {
        let app = router(AppState::with_store(Arc::new(InMemoryRunStore::new())));
        let first = app
            .clone()
            .oneshot(request_with(
                "POST",
                "/api/v1/cards/card-1/runs",
                &[("idempotency-key", "idem-key-1")],
            ))
            .await
            .expect("response");
        assert_eq!(first.status(), StatusCode::ACCEPTED);
        let first_body = first.into_body().collect().await.expect("body").to_bytes();
        let first_value: serde_json::Value = serde_json::from_slice(&first_body).expect("json");
        let second = app
            .oneshot(request_with(
                "POST",
                "/api/v1/cards/card-1/runs",
                &[("idempotency-key", "idem-key-1")],
            ))
            .await
            .expect("response");
        let second_body = second.into_body().collect().await.expect("body").to_bytes();
        let second_value: serde_json::Value = serde_json::from_slice(&second_body).expect("json");
        assert_eq!(first_value["id"], second_value["id"]);
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
