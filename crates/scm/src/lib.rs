//! AI-Lab SCM port and GitHub adapter.
//!
//! The port is provider-neutral. The GitHub adapter talks over an injectable
//! `HttpTransport`, so contract and security tests run without network.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

use ailab_application::Secret;
use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use sha2::Sha256;
use thiserror::Error;

/// SCM failure taxonomy.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ScmError {
    /// Authentication failed or the PAT expired.
    #[error("authentication failed")]
    Auth,
    /// A required scope is missing or the resource is forbidden.
    #[error("forbidden: {0}")]
    Forbidden(String),
    /// Resource not found.
    #[error("not found")]
    NotFound,
    /// Rate limited.
    #[error("rate limited")]
    RateLimited {
        /// Retry-after hint.
        retry_after_ms: Option<u64>,
    },
    /// Merge or branch conflict.
    #[error("conflict: {0}")]
    Conflict(String),
    /// Validation failure.
    #[error("validation failed: {0}")]
    Validation(String),
    /// Transport failure.
    #[error("transport error: {0}")]
    Transport(String),
    /// Duplicate or stale webhook delivery.
    #[error("replay detected")]
    Replay,
    /// Invalid webhook signature.
    #[error("invalid signature")]
    InvalidSignature,
}

impl ScmError {
    /// Stable machine code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Auth => "SCM_AUTH",
            Self::Forbidden(_) => "SCM_FORBIDDEN",
            Self::NotFound => "SCM_NOT_FOUND",
            Self::RateLimited { .. } => "SCM_RATE_LIMITED",
            Self::Conflict(_) => "SCM_CONFLICT",
            Self::Validation(_) => "SCM_VALIDATION",
            Self::Transport(_) => "SCM_TRANSPORT",
            Self::Replay => "SCM_REPLAY",
            Self::InvalidSignature => "SCM_INVALID_SIGNATURE",
        }
    }

    /// HTTP status used by the API mapping.
    #[must_use]
    pub const fn status(&self) -> u16 {
        match self {
            Self::Auth | Self::InvalidSignature => 401,
            Self::Forbidden(_) => 403,
            Self::NotFound => 404,
            Self::RateLimited { .. } => 429,
            Self::Conflict(_) | Self::Replay => 409,
            Self::Validation(_) => 422,
            Self::Transport(_) => 502,
        }
    }
}

/// `owner/repo` reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RepoRef {
    owner: String,
    name: String,
}

impl RepoRef {
    /// Parse and validate `owner/repo`.
    pub fn parse(value: &str) -> Result<Self, ScmError> {
        let (owner, name) = value
            .split_once('/')
            .ok_or_else(|| ScmError::Validation("expected owner/repo".to_owned()))?;
        if owner.is_empty() || name.is_empty() || value.contains("..") {
            return Err(ScmError::Validation(
                "invalid repository reference".to_owned(),
            ));
        }
        Ok(Self {
            owner: owner.to_owned(),
            name: name.to_owned(),
        })
    }

    /// Owner.
    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Repository name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// `owner/repo`.
    #[must_use]
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

/// Branch name, validated against traversal and unsafe characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchName(String);

impl BranchName {
    /// Build `ailab/{card_key}-{slug}`.
    pub fn for_card(card_key: &str, slug: &str) -> Result<Self, ScmError> {
        let branch = format!("ailab/{card_key}-{slug}");
        Self::parse(&branch)
    }

    /// Validate a branch name.
    pub fn parse(value: &str) -> Result<Self, ScmError> {
        if value.is_empty()
            || value.starts_with('/')
            || value.contains("..")
            || value.contains("//")
            || value
                .chars()
                .any(|c| matches!(c, ' ' | '~' | '^' | ':' | '?' | '*' | '[' | '\\'))
        {
            return Err(ScmError::Validation("invalid branch name".to_owned()));
        }
        Ok(Self(value.to_owned()))
    }

    /// Borrow the name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// File change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    /// Repository-relative path.
    pub path: String,
    /// UTF-8 content.
    pub content: String,
}

impl FileChange {
    /// Validate the path against traversal.
    pub fn new(path: impl Into<String>, content: impl Into<String>) -> Result<Self, ScmError> {
        let path = path.into();
        if path.starts_with('/') || path.contains("..") || path.contains(".git/") {
            return Err(ScmError::Validation("invalid file path".to_owned()));
        }
        Ok(Self {
            path,
            content: content.into(),
        })
    }
}

/// Credential state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialState {
    /// Active.
    Active,
    /// Expired or missing a scope.
    Expired,
}

/// Credential verification result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialStatus {
    /// State.
    pub state: CredentialState,
    /// Scopes reported by GitHub, when available.
    pub scopes: Vec<String>,
}

/// Check state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    /// All checks passed.
    Green,
    /// At least one check failed.
    Red,
    /// Checks are still running or absent.
    Pending,
}

/// CI summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckSummary {
    /// Aggregate state.
    pub status: CheckStatus,
    /// Total runs.
    pub total: u32,
    /// Passed.
    pub passed: u32,
    /// Failed.
    pub failed: u32,
    /// Pending.
    pub pending: u32,
}

/// Pull request request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequestRequest {
    /// Title.
    pub title: String,
    /// Body.
    pub body: String,
    /// Head branch.
    pub head: BranchName,
    /// Base branch.
    pub base: String,
    /// Author login.
    pub author: String,
    /// Reviewer login.
    pub reviewer: String,
}

/// Pull request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequest {
    /// Number.
    pub number: u64,
    /// HTML URL.
    pub url: String,
    /// Head branch.
    pub head: String,
    /// Base branch.
    pub base: String,
}

/// Review verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewVerdict {
    /// Approved.
    Approved,
    /// Changes requested.
    ChangesRequested,
}

/// Merge result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeOutcome {
    /// Merge SHA.
    pub merge_sha: String,
}

/// SCM port.
#[async_trait]
pub trait ScmProvider: Send + Sync {
    /// Verify the credential for a repository.
    async fn verify_credential(&self, repo: &RepoRef) -> Result<CredentialStatus, ScmError>;
    /// Create a branch.
    async fn create_branch(
        &self,
        repo: &RepoRef,
        branch: &BranchName,
        from_sha: &str,
    ) -> Result<(), ScmError>;
    /// Commit files to a branch and return the commit SHA.
    async fn commit_files(
        &self,
        repo: &RepoRef,
        branch: &BranchName,
        message: &str,
        files: &[FileChange],
    ) -> Result<String, ScmError>;
    /// Find an existing pull request for a head branch.
    async fn find_pull_request(
        &self,
        repo: &RepoRef,
        head: &BranchName,
    ) -> Result<Option<PullRequest>, ScmError>;
    /// Open a pull request.
    async fn open_pull_request(
        &self,
        repo: &RepoRef,
        request: &PullRequestRequest,
    ) -> Result<PullRequest, ScmError>;
    /// Read checks for a commit.
    async fn get_checks(&self, repo: &RepoRef, sha: &str) -> Result<CheckSummary, ScmError>;
    /// Submit a review.
    async fn submit_review(
        &self,
        repo: &RepoRef,
        pr_number: u64,
        verdict: ReviewVerdict,
        author: &str,
        reviewer: &str,
    ) -> Result<(), ScmError>;
    /// Merge a pull request with the safety guards.
    async fn merge_pull_request(
        &self,
        repo: &RepoRef,
        pr_number: u64,
        author: &str,
        reviewer: &str,
        checks: CheckStatus,
    ) -> Result<MergeOutcome, ScmError>;
    /// Comment on an issue or pull request.
    async fn comment(&self, repo: &RepoRef, number: u64, body: &str) -> Result<(), ScmError>;
}

/// In-memory SCM provider for tests and the pipeline E2E.
#[derive(Default)]
pub struct FakeScm {
    inner: Mutex<FakeScmState>,
}

#[derive(Default)]
struct FakeScmState {
    branches: Vec<String>,
    commits: Vec<String>,
    prs: Vec<PullRequest>,
    reviews: Vec<(u64, ReviewVerdict)>,
    merges: Vec<(u64, String)>,
    comments: Vec<String>,
}

impl FakeScm {
    /// Number of branches created.
    #[must_use]
    pub fn branch_count(&self) -> usize {
        self.inner.lock().map_or(0, |state| state.branches.len())
    }

    /// Number of commits.
    #[must_use]
    pub fn commit_count(&self) -> usize {
        self.inner.lock().map_or(0, |state| state.commits.len())
    }

    /// Number of pull requests.
    #[must_use]
    pub fn pr_count(&self) -> usize {
        self.inner.lock().map_or(0, |state| state.prs.len())
    }

    /// Number of merges.
    #[must_use]
    pub fn merge_count(&self) -> usize {
        self.inner.lock().map_or(0, |state| state.merges.len())
    }
}

#[async_trait]
impl ScmProvider for FakeScm {
    async fn verify_credential(&self, _repo: &RepoRef) -> Result<CredentialStatus, ScmError> {
        Ok(CredentialStatus {
            state: CredentialState::Active,
            scopes: vec!["repo".to_owned()],
        })
    }

    async fn create_branch(
        &self,
        _repo: &RepoRef,
        branch: &BranchName,
        _from_sha: &str,
    ) -> Result<(), ScmError> {
        let mut state = self
            .inner
            .lock()
            .map_err(|_| ScmError::Transport("fake scm lock poisoned".to_owned()))?;
        if !state
            .branches
            .iter()
            .any(|existing| existing == branch.as_str())
        {
            state.branches.push(branch.as_str().to_owned());
        }
        drop(state);
        Ok(())
    }

    async fn commit_files(
        &self,
        _repo: &RepoRef,
        _branch: &BranchName,
        message: &str,
        files: &[FileChange],
    ) -> Result<String, ScmError> {
        if files.is_empty() {
            return Err(ScmError::Validation("no files to commit".to_owned()));
        }
        ensure_commit_trailers(message)?;
        let mut state = self
            .inner
            .lock()
            .map_err(|_| ScmError::Transport("fake scm lock poisoned".to_owned()))?;
        let sha = format!("commit-{}", state.commits.len() + 1);
        state.commits.push(sha.clone());
        drop(state);
        Ok(sha)
    }

    async fn find_pull_request(
        &self,
        _repo: &RepoRef,
        head: &BranchName,
    ) -> Result<Option<PullRequest>, ScmError> {
        let state = self
            .inner
            .lock()
            .map_err(|_| ScmError::Transport("fake scm lock poisoned".to_owned()))?;
        Ok(state
            .prs
            .iter()
            .find(|pr| pr.head == head.as_str())
            .cloned())
    }

    async fn open_pull_request(
        &self,
        _repo: &RepoRef,
        request: &PullRequestRequest,
    ) -> Result<PullRequest, ScmError> {
        if request.author == request.reviewer {
            return Err(ScmError::Validation(
                "reviewer must differ from the author".to_owned(),
            ));
        }
        let mut state = self
            .inner
            .lock()
            .map_err(|_| ScmError::Transport("fake scm lock poisoned".to_owned()))?;
        let number = u64::try_from(state.prs.len() + 1).unwrap_or(u64::MAX);
        let pr = PullRequest {
            number,
            url: format!("https://example.test/pull/{number}"),
            head: request.head.as_str().to_owned(),
            base: request.base.clone(),
        };
        state.prs.push(pr.clone());
        drop(state);
        Ok(pr)
    }

    async fn get_checks(&self, _repo: &RepoRef, _sha: &str) -> Result<CheckSummary, ScmError> {
        Ok(CheckSummary {
            status: CheckStatus::Green,
            total: 1,
            passed: 1,
            failed: 0,
            pending: 0,
        })
    }

    async fn submit_review(
        &self,
        _repo: &RepoRef,
        pr_number: u64,
        verdict: ReviewVerdict,
        author: &str,
        reviewer: &str,
    ) -> Result<(), ScmError> {
        if author == reviewer {
            return Err(ScmError::Forbidden(
                "separation of duties: reviewer is the author".to_owned(),
            ));
        }
        let mut state = self
            .inner
            .lock()
            .map_err(|_| ScmError::Transport("fake scm lock poisoned".to_owned()))?;
        state.reviews.push((pr_number, verdict));
        drop(state);
        Ok(())
    }

    async fn merge_pull_request(
        &self,
        _repo: &RepoRef,
        _pr_number: u64,
        author: &str,
        reviewer: &str,
        checks: CheckStatus,
    ) -> Result<MergeOutcome, ScmError> {
        if author == reviewer {
            return Err(ScmError::Forbidden(
                "separation of duties: merger is the author".to_owned(),
            ));
        }
        if checks != CheckStatus::Green {
            return Err(ScmError::Conflict("checks are not green".to_owned()));
        }
        let mut state = self
            .inner
            .lock()
            .map_err(|_| ScmError::Transport("fake scm lock poisoned".to_owned()))?;
        if let Some((_, sha)) = state
            .merges
            .iter()
            .find(|(number, _)| *number == _pr_number)
        {
            return Ok(MergeOutcome {
                merge_sha: sha.clone(),
            });
        }
        let sha = format!("merge-{}", state.merges.len() + 1);
        state.merges.push((_pr_number, sha.clone()));
        drop(state);
        Ok(MergeOutcome { merge_sha: sha })
    }

    async fn comment(&self, _repo: &RepoRef, _number: u64, body: &str) -> Result<(), ScmError> {
        let mut state = self
            .inner
            .lock()
            .map_err(|_| ScmError::Transport("fake scm lock poisoned".to_owned()))?;
        state.comments.push(body.to_owned());
        drop(state);
        Ok(())
    }
}

/// PR conventions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrConventions {
    /// Card key.
    pub card_key: String,
    /// Card title.
    pub card_title: String,
    /// Summary bullets.
    pub summary: Vec<String>,
    /// Changed areas.
    pub changes: Vec<String>,
    /// Acceptance criteria.
    pub acceptance: Vec<String>,
    /// Verification evidence.
    pub verification: Vec<String>,
    /// Risks and rollback.
    pub risks: Vec<String>,
    /// Run id.
    pub run: String,
    /// Agent id.
    pub agent: String,
    /// Model name.
    pub model: String,
    /// Attempt number.
    pub attempt: u32,
}

impl PrConventions {
    /// Title: `[{CARD_KEY}] {CARD_TITLE}`.
    #[must_use]
    pub fn title(&self) -> String {
        format!("[{}] {}", self.card_key, self.card_title)
    }

    /// Body with the required sections and trailers.
    #[must_use]
    pub fn body(&self) -> String {
        let mut body = String::new();
        let _ = writeln!(body, "## Card");
        let _ = writeln!(body, "[{}] {}\n", self.card_key, self.card_title);
        let _ = writeln!(body, "## Summary");
        for item in &self.summary {
            let _ = writeln!(body, "- {item}");
        }
        let _ = writeln!(body, "\n## Changes");
        for item in &self.changes {
            let _ = writeln!(body, "- {item}");
        }
        let _ = writeln!(body, "\n## Acceptance criteria");
        for item in &self.acceptance {
            let _ = writeln!(body, "- [ ] {item}");
        }
        let _ = writeln!(body, "\n## Verification");
        for item in &self.verification {
            let _ = writeln!(body, "- {item}");
        }
        let _ = writeln!(body, "\n## Risks / rollback");
        for item in &self.risks {
            let _ = writeln!(body, "- {item}");
        }
        let _ = writeln!(body, "\n## AI-Lab trace");
        let _ = writeln!(body, "AI-Lab-Run: {}", self.run);
        let _ = writeln!(body, "AI-Lab-Card: {}", self.card_key);
        let _ = writeln!(body, "AI-Lab-Agent: {}", self.agent);
        let _ = writeln!(body, "AI-Lab-Model: {}", self.model);
        let _ = writeln!(body, "AI-Lab-Attempt: {}", self.attempt);
        body
    }
}

/// Webhook headers.
#[derive(Debug, Clone)]
pub struct WebhookHeaders {
    /// `X-GitHub-Delivery`.
    pub delivery_id: String,
    /// `X-GitHub-Event`.
    pub event: String,
    /// `X-Hub-Signature-256`.
    pub signature: String,
    /// When the API received the request.
    pub received_at: DateTime<Utc>,
}

/// Verified webhook event.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::derive_partial_eq_without_eq)]
pub struct WebhookEvent {
    /// Delivery id.
    pub delivery_id: String,
    /// Event name.
    pub event: String,
    /// Action, when present.
    pub action: Option<String>,
    /// Payload.
    pub payload: Value,
}

/// Webhook verifier with HMAC, replay window and idempotency.
pub struct WebhookVerifier {
    secret: Secret<String>,
    max_age_s: i64,
    seen: Mutex<HashSet<String>>,
}

impl WebhookVerifier {
    /// Build a verifier.
    #[must_use]
    pub fn new(secret: Secret<String>, max_age_s: i64) -> Self {
        Self {
            secret,
            max_age_s,
            seen: Mutex::new(HashSet::new()),
        }
    }

    /// Verify and parse a webhook.
    pub fn verify(
        &self,
        headers: &WebhookHeaders,
        body: &[u8],
        now: DateTime<Utc>,
    ) -> Result<WebhookEvent, ScmError> {
        if (now - headers.received_at).num_seconds() > self.max_age_s {
            return Err(ScmError::Replay);
        }
        let provided = headers
            .signature
            .strip_prefix("sha256=")
            .ok_or(ScmError::InvalidSignature)?;
        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret.expose().as_bytes())
            .map_err(|_| ScmError::InvalidSignature)?;
        mac.update(body);
        let provided_bytes = hex_decode(provided).ok_or(ScmError::InvalidSignature)?;
        mac.verify_slice(&provided_bytes)
            .map_err(|_| ScmError::InvalidSignature)?;
        {
            let mut seen = self
                .seen
                .lock()
                .map_err(|_| ScmError::Transport("webhook lock poisoned".to_owned()))?;
            if !seen.insert(headers.delivery_id.clone()) {
                return Err(ScmError::Replay);
            }
        }
        let payload: Value = serde_json::from_slice(body)
            .map_err(|error| ScmError::Validation(error.to_string()))?;
        let action = payload["action"].as_str().map(ToOwned::to_owned);
        Ok(WebhookEvent {
            delivery_id: headers.delivery_id.clone(),
            event: headers.event.clone(),
            action,
            payload,
        })
    }
}

fn ensure_commit_trailers(message: &str) -> Result<(), ScmError> {
    for trailer in ["AI-Lab-Run:", "AI-Lab-Card:", "AI-Lab-Agent:"] {
        if !message.lines().any(|line| line.starts_with(trailer)) {
            return Err(ScmError::Validation(format!(
                "missing required commit trailer: {trailer}"
            )));
        }
    }
    Ok(())
}

fn hex_decode(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    let mut output = Vec::with_capacity(value.len() / 2);
    let bytes = value.as_bytes();
    for pair in bytes.chunks(2) {
        let high = (pair[0] as char).to_digit(16)?;
        let low = (pair[1] as char).to_digit(16)?;
        output.push(u8::try_from(high * 16 + low).ok()?);
    }
    Some(output)
}

/// HTTP method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// GET.
    Get,
    /// POST.
    Post,
    /// PUT.
    Put,
}

/// HTTP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    /// Method.
    pub method: Method,
    /// URL.
    pub url: String,
    /// Headers.
    pub headers: BTreeMap<String, String>,
    /// Body.
    pub body: Option<String>,
}

/// HTTP response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    /// Status.
    pub status: u16,
    /// Body.
    pub body: String,
    /// Headers.
    pub headers: BTreeMap<String, String>,
}

/// HTTP transport port.
#[async_trait]
pub trait HttpTransport: Send + Sync {
    /// Send a request.
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse, ScmError>;
}

/// GitHub adapter over an injectable transport.
pub struct GitHubClient {
    transport: Arc<dyn HttpTransport>,
    token: Secret<String>,
    base_url: String,
}

impl GitHubClient {
    /// Build the client.
    #[must_use]
    pub fn new(
        transport: Arc<dyn HttpTransport>,
        token: Secret<String>,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            transport,
            token,
            base_url: base_url.into(),
        }
    }

    fn request(&self, method: Method, path: &str, body: Option<String>) -> HttpRequest {
        let mut headers = BTreeMap::new();
        headers.insert(
            "Authorization".to_owned(),
            format!("Bearer {}", self.token.expose()),
        );
        headers.insert(
            "Accept".to_owned(),
            "application/vnd.github+json".to_owned(),
        );
        headers.insert("X-GitHub-Api-Version".to_owned(), "2022-11-28".to_owned());
        headers.insert("Content-Type".to_owned(), "application/json".to_owned());
        HttpRequest {
            method,
            url: format!("{}{}", self.base_url.trim_end_matches('/'), path),
            headers,
            body,
        }
    }

    async fn send(&self, request: HttpRequest) -> Result<HttpResponse, ScmError> {
        let response = self.transport.send(request).await?;
        match response.status {
            200..=299 => Ok(response),
            401 => Err(ScmError::Auth),
            403 => {
                let retry = response
                    .headers
                    .get("retry-after")
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(|seconds| seconds * 1_000);
                if retry.is_some() || response.body.contains("rate limit") {
                    Err(ScmError::RateLimited {
                        retry_after_ms: retry,
                    })
                } else {
                    Err(ScmError::Forbidden(response.body))
                }
            }
            404 => Err(ScmError::NotFound),
            409 => Err(ScmError::Conflict(response.body)),
            422 => {
                if response.body.contains("already exists") {
                    Ok(response)
                } else {
                    Err(ScmError::Validation(response.body))
                }
            }
            429 => Err(ScmError::RateLimited {
                retry_after_ms: response
                    .headers
                    .get("retry-after")
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(|seconds| seconds * 1_000),
            }),
            status => Err(ScmError::Transport(format!("HTTP {status}"))),
        }
    }
}

#[async_trait]
impl ScmProvider for GitHubClient {
    async fn verify_credential(&self, repo: &RepoRef) -> Result<CredentialStatus, ScmError> {
        let response = match self
            .send(self.request(Method::Get, &format!("/repos/{}", repo.full_name()), None))
            .await
        {
            Ok(response) => response,
            Err(ScmError::Forbidden(_) | ScmError::NotFound) => {
                return Ok(CredentialStatus {
                    state: CredentialState::Expired,
                    scopes: Vec::new(),
                });
            }
            Err(error) => return Err(error),
        };
        let scopes: Vec<String> = response
            .headers
            .get("x-oauth-scopes")
            .map(|value| {
                value
                    .split(',')
                    .map(|scope| scope.trim().to_owned())
                    .filter(|scope| !scope.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        // Fine-grained PATs report no classic scopes; a present header without
        // `repo`/`public_repo` means a classic PAT lacks the required scope.
        let state = if scopes.is_empty()
            || scopes
                .iter()
                .any(|scope| scope == "repo" || scope == "public_repo")
        {
            CredentialState::Active
        } else {
            CredentialState::Expired
        };
        Ok(CredentialStatus { state, scopes })
    }

    async fn create_branch(
        &self,
        repo: &RepoRef,
        branch: &BranchName,
        from_sha: &str,
    ) -> Result<(), ScmError> {
        let body = json!({
            "ref": format!("refs/heads/{}", branch.as_str()),
            "sha": from_sha
        })
        .to_string();
        self.send(self.request(
            Method::Post,
            &format!("/repos/{}/git/refs", repo.full_name()),
            Some(body),
        ))
        .await?;
        Ok(())
    }

    #[allow(clippy::assigning_clones)]
    async fn commit_files(
        &self,
        repo: &RepoRef,
        branch: &BranchName,
        message: &str,
        files: &[FileChange],
    ) -> Result<String, ScmError> {
        if files.is_empty() {
            return Err(ScmError::Validation("no files to commit".to_owned()));
        }
        ensure_commit_trailers(message)?;
        let mut last_sha = String::new();
        for file in files {
            let body = json!({
                "message": message,
                "content": BASE64.encode(file.content.as_bytes()),
                "branch": branch.as_str()
            })
            .to_string();
            let response = self
                .send(self.request(
                    Method::Put,
                    &format!("/repos/{}/contents/{}", repo.full_name(), file.path),
                    Some(body),
                ))
                .await?;
            let value: Value = serde_json::from_str(&response.body)
                .map_err(|error| ScmError::Validation(error.to_string()))?;
            last_sha = value["commit"]["sha"]
                .as_str()
                .ok_or_else(|| ScmError::Validation("missing commit sha".to_owned()))?
                .to_owned();
        }
        Ok(last_sha)
    }

    async fn find_pull_request(
        &self,
        repo: &RepoRef,
        head: &BranchName,
    ) -> Result<Option<PullRequest>, ScmError> {
        let path = format!(
            "/repos/{}/pulls?state=all&head={}:{}",
            repo.full_name(),
            repo.owner(),
            head.as_str()
        );
        let response = self.send(self.request(Method::Get, &path, None)).await?;
        let value: Value = serde_json::from_str(&response.body)
            .map_err(|error| ScmError::Validation(error.to_string()))?;
        let Some(pr) = value.as_array().and_then(|items| items.first()) else {
            return Ok(None);
        };
        Ok(Some(PullRequest {
            number: pr["number"]
                .as_u64()
                .ok_or_else(|| ScmError::Validation("missing PR number".to_owned()))?,
            url: pr["html_url"]
                .as_str()
                .ok_or_else(|| ScmError::Validation("missing PR url".to_owned()))?
                .to_owned(),
            head: pr["head"]["ref"]
                .as_str()
                .unwrap_or(head.as_str())
                .to_owned(),
            base: pr["base"]["ref"].as_str().unwrap_or("main").to_owned(),
        }))
    }

    async fn open_pull_request(
        &self,
        repo: &RepoRef,
        request: &PullRequestRequest,
    ) -> Result<PullRequest, ScmError> {
        if request.author == request.reviewer {
            return Err(ScmError::Validation(
                "reviewer must differ from the author".to_owned(),
            ));
        }
        let body = json!({
            "title": request.title,
            "head": request.head.as_str(),
            "base": request.base,
            "body": request.body,
            "draft": false
        })
        .to_string();
        let response = self
            .send(self.request(
                Method::Post,
                &format!("/repos/{}/pulls", repo.full_name()),
                Some(body),
            ))
            .await?;
        let value: Value = serde_json::from_str(&response.body)
            .map_err(|error| ScmError::Validation(error.to_string()))?;
        Ok(PullRequest {
            number: value["number"]
                .as_u64()
                .ok_or_else(|| ScmError::Validation("missing PR number".to_owned()))?,
            url: value["html_url"]
                .as_str()
                .ok_or_else(|| ScmError::Validation("missing PR url".to_owned()))?
                .to_owned(),
            head: value["head"]["ref"].as_str().unwrap_or_default().to_owned(),
            base: value["base"]["ref"].as_str().unwrap_or_default().to_owned(),
        })
    }

    async fn get_checks(&self, repo: &RepoRef, sha: &str) -> Result<CheckSummary, ScmError> {
        let response = self
            .send(self.request(
                Method::Get,
                &format!("/repos/{}/commits/{sha}/check-runs", repo.full_name()),
                None,
            ))
            .await?;
        let value: Value = serde_json::from_str(&response.body)
            .map_err(|error| ScmError::Validation(error.to_string()))?;
        let runs = value["check_runs"]
            .as_array()
            .ok_or_else(|| ScmError::Validation("missing check_runs".to_owned()))?;
        let mut passed = 0;
        let mut failed = 0;
        let mut pending = 0;
        for run in runs {
            match run["conclusion"].as_str() {
                Some("success" | "neutral" | "skipped") => passed += 1,
                Some("failure" | "cancelled" | "timed_out" | "action_required") => failed += 1,
                _ => pending += 1,
            }
        }
        let total = u32::try_from(runs.len()).unwrap_or(u32::MAX);
        let status = if failed > 0 {
            CheckStatus::Red
        } else if pending > 0 || total == 0 {
            CheckStatus::Pending
        } else {
            CheckStatus::Green
        };
        Ok(CheckSummary {
            status,
            total,
            passed,
            failed,
            pending,
        })
    }

    async fn submit_review(
        &self,
        repo: &RepoRef,
        pr_number: u64,
        verdict: ReviewVerdict,
        author: &str,
        reviewer: &str,
    ) -> Result<(), ScmError> {
        if author == reviewer {
            return Err(ScmError::Forbidden(
                "separation of duties: reviewer is the author".to_owned(),
            ));
        }
        let event = match verdict {
            ReviewVerdict::Approved => "APPROVE",
            ReviewVerdict::ChangesRequested => "REQUEST_CHANGES",
        };
        let body = json!({"event": event, "body": ""}).to_string();
        self.send(self.request(
            Method::Post,
            &format!("/repos/{}/pulls/{pr_number}/reviews", repo.full_name()),
            Some(body),
        ))
        .await?;
        Ok(())
    }

    async fn merge_pull_request(
        &self,
        repo: &RepoRef,
        pr_number: u64,
        author: &str,
        reviewer: &str,
        checks: CheckStatus,
    ) -> Result<MergeOutcome, ScmError> {
        if author == reviewer {
            return Err(ScmError::Forbidden(
                "separation of duties: merger is the author".to_owned(),
            ));
        }
        if checks != CheckStatus::Green {
            return Err(ScmError::Conflict("checks are not green".to_owned()));
        }
        let body = json!({"merge_method": "squash"}).to_string();
        let response = self
            .send(self.request(
                Method::Put,
                &format!("/repos/{}/pulls/{pr_number}/merge", repo.full_name()),
                Some(body),
            ))
            .await?;
        let value: Value = serde_json::from_str(&response.body)
            .map_err(|error| ScmError::Validation(error.to_string()))?;
        Ok(MergeOutcome {
            merge_sha: value["sha"]
                .as_str()
                .ok_or_else(|| ScmError::Validation("missing merge sha".to_owned()))?
                .to_owned(),
        })
    }

    async fn comment(&self, repo: &RepoRef, number: u64, body: &str) -> Result<(), ScmError> {
        let payload = json!({"body": body}).to_string();
        self.send(self.request(
            Method::Post,
            &format!("/repos/{}/issues/{number}/comments", repo.full_name()),
            Some(payload),
        ))
        .await?;
        Ok(())
    }
}

/// Short-lived sandbox token.
#[derive(Debug, Clone)]
pub struct RepoToken {
    /// Token value.
    pub token: Secret<String>,
    /// Expiry.
    pub expires_at: DateTime<Utc>,
    /// Repository.
    pub repo: RepoRef,
}

/// In-memory token broker for sandbox runs.
#[derive(Debug, Default)]
pub struct TokenBroker {
    tokens: Mutex<BTreeMap<String, RepoToken>>,
}

impl TokenBroker {
    /// Issue a short-lived token for a run.
    pub fn issue(&self, run_id: &str, repo: RepoRef, now: DateTime<Utc>, ttl_s: i64) -> RepoToken {
        let token = RepoToken {
            token: Secret::new(format!("ailab-sandbox-{}", uuid::Uuid::new_v4())),
            expires_at: now + chrono::Duration::seconds(ttl_s),
            repo,
        };
        if let Ok(mut tokens) = self.tokens.lock() {
            tokens.insert(run_id.to_owned(), token.clone());
        }
        token
    }

    /// Revoke a token.
    pub fn revoke(&self, run_id: &str) {
        if let Ok(mut tokens) = self.tokens.lock() {
            tokens.remove(run_id);
        }
    }

    /// Read a token if it is still valid.
    #[must_use]
    pub fn get(&self, run_id: &str, now: DateTime<Utc>) -> Option<RepoToken> {
        self.tokens.lock().ok().and_then(|tokens| {
            tokens
                .get(run_id)
                .filter(|token| token.expires_at > now)
                .cloned()
        })
    }
}

/// Parse `Retry-After` and `X-RateLimit-*` headers.
#[must_use]
pub fn parse_rate_limit(headers: &BTreeMap<String, String>) -> Option<u64> {
    headers
        .get("retry-after")
        .and_then(|value| value.parse::<u64>().ok())
        .map(|seconds| seconds * 1_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FakeTransport {
        responses: Mutex<VecDeque<Result<HttpResponse, ScmError>>>,
        requests: Mutex<Vec<HttpRequest>>,
        calls: AtomicUsize,
    }

    #[async_trait]
    impl HttpTransport for FakeTransport {
        async fn send(&self, request: HttpRequest) -> Result<HttpResponse, ScmError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.requests.lock().expect("requests").push(request);
            self.responses
                .lock()
                .expect("responses")
                .pop_front()
                .unwrap_or_else(|| Err(ScmError::Transport("no response".to_owned())))
        }
    }

    fn client(responses: Vec<HttpResponse>) -> (GitHubClient, Arc<FakeTransport>) {
        let transport = Arc::new(FakeTransport {
            responses: Mutex::new(
                responses
                    .into_iter()
                    .map(Ok)
                    .collect::<VecDeque<Result<HttpResponse, ScmError>>>(),
            ),
            requests: Mutex::new(Vec::new()),
            calls: AtomicUsize::new(0),
        });
        let client = GitHubClient::new(
            transport.clone(),
            Secret::new("pat".to_owned()),
            "https://api.github.test",
        );
        (client, transport)
    }

    fn response(status: u16, body: &str) -> HttpResponse {
        HttpResponse {
            status,
            body: body.to_owned(),
            headers: BTreeMap::new(),
        }
    }

    fn repo() -> RepoRef {
        RepoRef::parse("example/ai-lab").expect("repo")
    }

    #[test]
    fn repository_and_branch_validation() {
        assert!(RepoRef::parse("example/ai-lab").is_ok());
        assert!(RepoRef::parse("..").is_err());
        assert!(BranchName::for_card("AIL-1", "fix").is_ok());
        assert!(BranchName::parse("../evil").is_err());
        assert!(FileChange::new("../secret", "x").is_err());
    }

    #[test]
    fn pr_conventions_include_card_and_trace() {
        let conventions = PrConventions {
            card_key: "AIL-1".to_owned(),
            card_title: "Add board".to_owned(),
            summary: vec!["One".to_owned()],
            changes: vec!["Added the board".to_owned()],
            acceptance: vec!["Works".to_owned()],
            verification: vec!["cargo test".to_owned()],
            risks: vec!["None".to_owned()],
            run: "run-1".to_owned(),
            agent: "agent-1".to_owned(),
            model: "deepseek-flash".to_owned(),
            attempt: 1,
        };
        assert_eq!(conventions.title(), "[AIL-1] Add board");
        let body = conventions.body();
        for section in [
            "## Summary",
            "## Changes",
            "## Verification",
            "## Risks / rollback",
        ] {
            assert!(body.contains(section), "missing {section}");
        }
        for trailer in [
            "AI-Lab-Run: run-1",
            "AI-Lab-Card: AIL-1",
            "AI-Lab-Agent: agent-1",
        ] {
            assert!(body.contains(trailer), "missing {trailer}");
        }
    }

    #[test]
    fn commit_requires_agent_trailers() {
        let (client, _) = client(vec![response(200, r#"{"commit":{"sha":"abc123"}}"#)]);
        let repo = repo();
        let branch = BranchName::for_card("AIL-1", "fix").expect("branch");
        let files = vec![FileChange::new("src/lib.rs", "content").expect("file")];
        let missing = futures::executor::block_on(client.commit_files(
            &repo,
            &branch,
            "feat(test): add thing",
            &files,
        ));
        assert!(matches!(missing, Err(ScmError::Validation(_))));
        let with_trailers = futures::executor::block_on(client.commit_files(
            &repo,
            &branch,
            "feat(test): add thing\n\nAI-Lab-Run: run-1\nAI-Lab-Card: AIL-1\nAI-Lab-Agent: agent-1",
            &files,
        ))
        .expect("commit");
        assert_eq!(with_trailers, "abc123");
    }

    fn hex_encode(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        output
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn webhook_hmac_replay_and_age() {
        let secret = "webhook-secret";
        let verifier = WebhookVerifier::new(Secret::new(secret.to_owned()), 300);
        let body = br#"{"action":"opened"}"#;
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac");
        mac.update(body);
        let signature = format!("sha256={}", hex_encode(&mac.finalize().into_bytes()));
        let now = Utc::now();
        let headers = WebhookHeaders {
            delivery_id: "delivery-1".to_owned(),
            event: "pull_request".to_owned(),
            signature,
            received_at: now,
        };
        assert!(verifier.verify(&headers, body, now).is_ok());
        assert_eq!(verifier.verify(&headers, body, now), Err(ScmError::Replay));
        let mut bad = headers.clone();
        bad.delivery_id = "delivery-2".to_owned();
        bad.signature = "sha256=00".to_owned();
        assert_eq!(
            verifier.verify(&bad, body, now),
            Err(ScmError::InvalidSignature)
        );
        let mut old = headers.clone();
        old.delivery_id = "delivery-3".to_owned();
        old.received_at = now - chrono::Duration::seconds(301);
        assert_eq!(verifier.verify(&old, body, now), Err(ScmError::Replay));
    }

    #[test]
    fn checks_map_to_green_red_and_pending() {
        let (client, _) = client(vec![
            response(
                200,
                r#"{"check_runs":[{"conclusion":"success"},{"conclusion":"failure"}]}"#,
            ),
            response(200, r#"{"check_runs":[{"conclusion":"success"}]}"#),
            response(200, r#"{"check_runs":[{"conclusion":null}]}"#),
        ]);
        let repo = repo();
        let red = futures::executor::block_on(client.get_checks(&repo, "sha")).expect("checks");
        assert_eq!(red.status, CheckStatus::Red);
        let green = futures::executor::block_on(client.get_checks(&repo, "sha")).expect("checks");
        assert_eq!(green.status, CheckStatus::Green);
        let pending = futures::executor::block_on(client.get_checks(&repo, "sha")).expect("checks");
        assert_eq!(pending.status, CheckStatus::Pending);
    }

    #[test]
    fn merge_requires_separation_and_green_checks() {
        let (client, _) = client(vec![response(200, r#"{"sha":"merge-sha"}"#)]);
        let repo = repo();
        assert_eq!(
            futures::executor::block_on(client.merge_pull_request(
                &repo,
                1,
                "alice",
                "alice",
                CheckStatus::Green
            )),
            Err(ScmError::Forbidden(
                "separation of duties: merger is the author".to_owned()
            ))
        );
        assert_eq!(
            futures::executor::block_on(client.merge_pull_request(
                &repo,
                1,
                "alice",
                "bob",
                CheckStatus::Pending
            )),
            Err(ScmError::Conflict("checks are not green".to_owned()))
        );
        let outcome = futures::executor::block_on(client.merge_pull_request(
            &repo,
            1,
            "alice",
            "bob",
            CheckStatus::Green,
        ))
        .expect("merge");
        assert_eq!(outcome.merge_sha, "merge-sha");
    }

    #[test]
    fn open_pr_sends_conventions() {
        let (client, transport) = client(vec![response(
            201,
            r#"{"number":7,"html_url":"https://github.test/pr/7","head":{"ref":"ailab/AIL-1-fix"},"base":{"ref":"main"}}"#,
        )]);
        let repo = repo();
        let request = PullRequestRequest {
            title: "[AIL-1] Add board".to_owned(),
            body: "body".to_owned(),
            head: BranchName::for_card("AIL-1", "fix").expect("branch"),
            base: "main".to_owned(),
            author: "alice".to_owned(),
            reviewer: "bob".to_owned(),
        };
        let pr =
            futures::executor::block_on(client.open_pull_request(&repo, &request)).expect("pr");
        assert_eq!(pr.number, 7);
        let sent = transport.requests.lock().expect("requests");
        assert_eq!(sent[0].method, Method::Post);
        assert!(sent[0].url.ends_with("/repos/example/ai-lab/pulls"));
        assert!(
            sent[0]
                .body
                .as_deref()
                .unwrap_or_default()
                .contains("[AIL-1] Add board")
        );
        drop(sent);
    }

    #[test]
    fn token_broker_issues_and_revokes() {
        let broker = TokenBroker::default();
        let now = Utc::now();
        let token = broker.issue("run-1", repo(), now, 300);
        assert!(token.expires_at > now);
        assert!(broker.get("run-1", now).is_some());
        broker.revoke("run-1");
        assert!(broker.get("run-1", now).is_none());
    }

    #[test]
    fn token_broker_tokens_are_unpredictable() {
        let broker = TokenBroker::default();
        let now = Utc::now();
        let first = broker.issue("run-1", repo(), now, 300);
        let second = broker.issue("run-1", repo(), now, 300);
        assert_ne!(first.token.expose(), second.token.expose());
        assert!(!first.token.expose().contains("run-1"));
        assert!(
            broker
                .get("run-1", now + chrono::Duration::seconds(301))
                .is_none()
        );
    }

    #[test]
    fn credential_verification_reports_missing_scopes() {
        let (client, _) = client(vec![
            HttpResponse {
                status: 200,
                body: "{}".to_owned(),
                headers: BTreeMap::from([("x-oauth-scopes".to_owned(), "repo".to_owned())]),
            },
            HttpResponse {
                status: 200,
                body: "{}".to_owned(),
                headers: BTreeMap::from([("x-oauth-scopes".to_owned(), "read:user".to_owned())]),
            },
            response(403, "forbidden"),
        ]);
        let repo = repo();
        let active = futures::executor::block_on(client.verify_credential(&repo)).expect("active");
        assert_eq!(active.state, CredentialState::Active);
        let scoped = futures::executor::block_on(client.verify_credential(&repo)).expect("scoped");
        assert_eq!(scoped.state, CredentialState::Expired);
        let expired =
            futures::executor::block_on(client.verify_credential(&repo)).expect("expired");
        assert_eq!(expired.state, CredentialState::Expired);
    }
}
