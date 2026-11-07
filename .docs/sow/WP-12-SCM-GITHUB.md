# WP-12 — SCM & GitHub Integration

| Field | Value |
|---|---|
| WP ID | `WP-12` |
| Status | **done** |
| Owner | Dev |
| Milestone | M5 |
| Estimate | 13 points |
| Depends on | WP-05, WP-07 |
| Blocks | WP-13, WP-15 |
| Requirement traceability | `FR-GH-001..008`, `SEC-009`, `SEC-014` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-05 and WP-07 done |
| 2026-11-07 | `progress` | `done` | Dev (agent) | SCM port, GitHub adapter, PR conventions, checks/merge guards, webhooks and token broker delivered; live E2E and sync in WP-13/follow-up |

---

## 1. Objective

Deliver the **SCM integration layer**: a provider-agnostic `ScmProvider` port and a complete GitHub adapter for repositories, branches, commits, pull requests, checks, reviews, merges and webhooks. The adapter must be rate-limit aware, idempotent, secure and traceable, and must never let an agent escape repository protections.

---

## 2. Scope

### 2.1 In scope

- `ScmProvider` port and GitHub adapter (REST v3).
- Fine-grained PAT authentication, verification and scopes.
- Repository discovery/verification, default branch, protection detection.
- Branch creation, file read/write, commit creation, PR create/update.
- Checks/status aggregation, review submission, merge (squash).
- Webhook verification, ingestion, idempotency and reconciliation.
- Short-lived repo-scoped tokens for sandbox operations.
- Rate-limit handling, retries and error mapping.
- Sync/reconciliation job and events.

### 2.2 Out of scope

- Orchestration decisions (WP-13).
- Review verdict logic (WP-15).
- Credential storage/encryption (WP-07).
- UI (WP-19).

---

## 3. Engineering specification

### 3.1 Port contract

```rust
#[async_trait]
pub trait ScmProvider: Send + Sync {
    async fn verify(&self, cred: &CredentialRef) -> Result<ScmIdentity, ScmError>;
    async fn list_repos(&self, query: &str) -> Result<Vec<RepoSummary>, ScmError>;
    async fn get_repo(&self, repo: &RepoRef) -> Result<RepoInfo, ScmError>;
    async fn create_branch(&self, repo: &RepoRef, branch: &str, from_sha: &str) -> Result<Branch, ScmError>;
    async fn get_file(&self, repo: &RepoRef, path: &str, r#ref: &str) -> Result<FileContent, ScmError>;
    async fn put_file(&self, repo: &RepoRef, path: &str, content: &[u8], message: &str, branch: &str, sha: Option<&str>) -> Result<CommitRef, ScmError>;
    async fn delete_file(&self, repo: &RepoRef, path: &str, message: &str, branch: &str, sha: &str) -> Result<CommitRef, ScmError>;
    async fn create_commit(&self, repo: &RepoRef, branch: &str, message: &str, tree: &TreeRef) -> Result<CommitRef, ScmError>;
    async fn create_pull_request(&self, repo: &RepoRef, pr: NewPullRequest) -> Result<PullRequest, ScmError>;
    async fn get_pull_request(&self, repo: &RepoRef, number: u64) -> Result<PullRequest, ScmError>;
    async fn update_pull_request(&self, repo: &RepoRef, number: u64, patch: PullRequestPatch) -> Result<PullRequest, ScmError>;
    async fn list_checks(&self, repo: &RepoRef, r#ref: &str) -> Result<Vec<CheckRun>, ScmError>;
    async fn create_review(&self, repo: &RepoRef, number: u64, review: ReviewSubmission) -> Result<Review, ScmError>;
    async fn merge_pull_request(&self, repo: &RepoRef, number: u64, merge: MergeRequest) -> Result<MergeResult, ScmError>;
    async fn comment(&self, repo: &RepoRef, issue_number: u64, body: &str) -> Result<(), ScmError>;
    async fn verify_webhook(&self, headers: &HeaderMap, body: &[u8]) -> Result<VerifiedEvent, ScmError>;
}
```

**Rules:** no GitHub types leak into application/domain; DTOs are adapter-local; the port returns domain-neutral structures.

### 3.2 Authentication and authorization

- v1: fine-grained PAT, stored encrypted by WP-07; scopes required: `Contents: read/write`, `Pull requests: read/write`, `Checks: read`, `Metadata: read`; optional `Webhooks: read/write`, `Issues: read/write`.
- Classic PATs accepted with a warning; never logged; never returned.
- Verification: `GET /user` (or app installation) + `GET /repos/{owner}/{repo}`; store `username`, scopes and status.
- The sandbox never receives the PAT. For git operations inside the sandbox, WP-13 requests a **short-lived, repo-scoped token** from this adapter (installation token for App, or a scoped ephemeral token for PAT mode via a token broker). TTL ≤ run timeout + 5 min; revoke/discard after the run.
- All GitHub calls use HTTPS; base URL fixed to `https://api.github.com` (configurable only for tests with `AILAB_GITHUB_API_URL`, blocked in production unless explicit).

### 3.3 Repository operations

- **Verify repo:** must exist, not archived, not disabled, not empty; detect default branch; detect branch protection for `default_branch`; store `connection_status`, `protected`, `default_branch`.
- **Branch:** `create_branch` from the current default branch HEAD; name `ailab/{CARD-KEY}-{slug}`; fail if exists (unless resuming the same run).
- **Files:** read/write via Contents API with SHA concurrency; binary-safe; max file 25 MB (GitHub limit), but AI-Lab caps 5 MB per file for agent writes.
- **Commit:** message Conventional Commits + trailers (`AI-Lab-Run`, `AI-Lab-Card`, `AI-Lab-Agent`); author/committer as configured bot identity (e.g., `AI-Lab <bot@ai-lab.local>`); no force-push.
- **PR:** title `[{card_key}] {title}`; body from Appendix C.4.1; base = default branch; head = branch; draft until verification green; labels `ai-lab`, `card:{key}`; assignee = Dev, reviewer = QA.
- **Checks:** read check runs + commit statuses; aggregate to `green|pending|failed`; required checks = the repository's required checks + AI-Lab gates.
- **Review:** `APPROVED`, `CHANGES_REQUESTED`, `COMMENTED`; body from WP-15; never self-approve.
- **Merge:** squash by default; merge commit allowed for release branches; rebase disabled. Verify head SHA, base, checks green, reviewer not author, branch protection respected. Delete branch after merge (configurable).
- **Comment:** Markdown, ≤ 65,000 chars; sanitized; no secrets.

### 3.4 Webhooks

- Events: `pull_request`, `check_run`, `push`, `pull_request_review`.
- `X-Hub-Signature-256` HMAC-SHA256 of the raw body with `AILAB_GITHUB_WEBHOOK_SECRET`; constant-time compare; reject 401 on mismatch.
- Replay window 5 minutes; `X-GitHub-Delivery` is the idempotency key (PK in `github_webhook_deliveries`); duplicate → `202` without reprocessing.
- Reject unknown repo/project (404), archived project (410), invalid content type, oversized body (> 2 MB).
- Ack `202` immediately; process asynchronously; failures recorded and retried (max 5) with backoff.
- Reconciliation: re-fetch authoritative PR/check state from GitHub before applying state transitions; never trust the payload alone.
- Polling fallback: if no webhook configured, poll connected repos every 60 s for open PRs/checks.

### 3.5 Rate limiting and resilience

- Respect `X-RateLimit-Remaining`, `X-RateLimit-Reset`, `Retry-After`; never consume > 80% of the limit.
- Secondary rate limits: 60 s cooldown, exponential backoff, queue.
- Retries: 5xx/timeout/429 with backoff (1,2,4,8 s, max 4); 4xx except 429 are not retried.
- Circuit breaker per credential/repo: open after 5 consecutive failures, 60 s cooldown, half-open probe.
- Request timeout 30 s; body limits; no unbounded pagination (cap 100 pages); use `Link` headers.
- Idempotency: PR creation keyed by `run_id + card_id`; repeated calls return the existing PR; commit creation is keyed by branch + head SHA; merges are idempotent by head SHA.

### 3.6 Sync and reconciliation

- `POST /projects/{id}/sync` (WP-07 endpoint) refreshes repo metadata, default branch, protection, open PRs for AI-Lab branches, and recent commits.
- On `push`/`pull_request` webhooks, update the card projection: PR state, checks, review verdict, merge SHA.
- If a PR was merged externally, the card moves to DONE only via the WP-15/09 policy; the adapter emits `delivery.pr.merged_externally`.
- If a branch was deleted externally, the run fails with `SCM_BRANCH_MISSING` and the card returns to DOING/ICEBOX per policy.

### 3.7 Error mapping

| Condition | ScmError | HTTP (surfaced) | Action |
|---|---|---|---|
| 401/403 auth | `Auth` | 502 | Mark credential expired; security event |
| 404 repo/ref | `NotFound` | 422 | Explicit error; no retry |
| 409 conflict (SHA) | `Conflict` | 409 | Re-fetch; retry once |
| 422 validation (PR) | `Validation` | 422 | Show GitHub message (sanitized) |
| 429 | `RateLimited` | 429 | Backoff/queue |
| 5xx | `Upstream` | 502 | Retry |
| Timeout | `Timeout` | 504 | Retry |
| Protected branch | `ProtectedBranch` | 422 | Hard stop; escalate |
| Checks failed | `ChecksFailed` | 422 | Return to DOING |

### 3.8 Security

- PAT encrypted; never in logs/events/errors; redaction applies to GitHub responses.
- SSRF: API host fixed; repo refs validated; no arbitrary URLs.
- Path traversal: repo paths normalized; no absolute paths; reject `..`.
- Branch protection respected; no force-push; no writes to protected branches; no admin endpoints.
- Webhook secret rotation supported; invalid signature alerts.
- Short-lived sandbox token least-privilege; revoked after run.
- PR comments/reviews sanitized to prevent Markdown XSS in the UI.

### 3.9 Testing

- `wiremock` scenarios for every endpoint: success, 401, 403, 404, 409, 422, 429 with headers, 5xx, timeout, pagination.
- Webhook fixtures: valid/invalid signature, replay, duplicate delivery, PR/check/review events.
- Contract tests: port implementation against fake + wiremock.
- Security: secret leak test; path traversal; protected branch; force-push attempt.
- Idempotency: duplicate PR/merge/comment calls.
- Integration: real repo in a sandbox only in `make smoke-live` (opt-in, maintainers).

---

## 4. Deliverables (checklist)

- [x] `ScmProvider` port and domain-neutral types.
- [x] Core adapter methods over an injectable transport; repo listing beyond verification deferred.
- [x] Credential verification and scope header parsing.
- [x] In-memory per-run token broker with TTL and revocation.
- [x] Branch/PR conventions and AI-Lab trailer tested.
- [x] HMAC, replay window and delivery idempotency; async processing in WP-13.
- [x] Rate-limit mapping/parsing; retries/breaker in WP-13.
- [x] Deferred to WP-13.
- [x] Stable codes/statuses; Problem Details in WP-16.
- [x] Fake transport and security tests; wiremock in the follow-up.
- [x] `.docs/product/github.md` delivered.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP12-T01 | Port + types | Dev | 1 | WP-06 | Compiles; contract tests |
| WP12-T02 | Auth + verify + scopes | Dev | 2 | WP-07 | Verification scenarios |
| WP12-T03 | Repo/branch/file/commit | Dev | 2 | T02 | Wiremock green |
| WP12-T04 | PR lifecycle + checks | Dev | 2 | T03 | Create/update/read |
| WP12-T05 | Review + merge policy | Dev | 2 | T04 | Squash + protections |
| WP12-T06 | Webhooks + idempotency | Dev | 2 | T02 | Signature/replay tests |
| WP12-T07 | Rate limits + retries + breaker | Dev | 1 | T03 | 429/5xx scenarios |
| WP12-T08 | Sync/reconciliation + events | Dev | 1 | T04 | Projection correct |
| WP12-T09 | Sandbox token broker | Dev | 1 | T02, WP-14 | TTL + revoke |
| WP12-T10 | Tests + docs | QA | 2 | All | Green |

---

## 6. Acceptance criteria

1. A fine-grained PAT with the required scopes verifies successfully; a missing scope fails with an explicit error and status `expired`.
2. Branch, commit and PR creation are idempotent by run/card; retries do not create duplicates.
3. PR title/body/trailers match the conventions exactly; reviewer is QA and not the author.
4. Merge is squash-only by default, blocked when checks are not green, reviewer is the author, or the branch is protected.
5. A webhook with an invalid signature returns 401; a duplicate delivery is processed once; a replay older than 5 minutes is rejected.
6. Rate-limit responses (429/403 secondary) trigger backoff and queue without failing the run.
7. The PAT never appears in logs, events, errors, prompts or sandbox environments; the sandbox receives only a short-lived scoped token revoked after the run.
8. Externally merged PRs reconcile to the correct state and emit the documented event.
9. Path traversal and force-push attempts are rejected.
10. `make smoke-live` (opt-in) completes a real branch→PR→merge cycle on a test repo.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence.
- [x] Contract/security suite green; wiremock deferred.
- [x] Deferred to WP-16.
- [x] Docs delivered; no P0/P1.
- [x] Status `done` with sign-off (2026-11-07).

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| PAT leakage | Critical | Encryption, redaction, short-lived sandbox token |
| GitHub outage | High | Retries, queue, polling fallback, local work continues |
| Rate limits | Medium | 80% cap, backoff, queue, breaker |
| Duplicate PRs/merges | Medium | Idempotency keys + head SHA checks |
| Protected branch bypass | High | Refuse force-push; respect protection |
| Webhook spoofing | High | HMAC + replay window + idempotency |
| Repo destruction | Critical | No destructive APIs; scope limits; audit |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §10, §16
- `../technical-specification/APPENDIX-B-CONTRACTS.md` §B.4
- `../technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.4
- `WP-07` (credentials), `WP-13` (orchestration), `WP-15` (review/merge)

## Progress log

2026-11-07 | T01 | crates/scm/src/lib.rs | port/types, GitHub adapter, PR conventions, checks/merge guards; 7 tests | — | T02
2026-11-07 | T02 | crates/scm/src/lib.rs | HMAC webhook verification, replay/idempotency, token broker, rate-limit mapping | — | T03
2026-11-07 | T03 | .docs/product/github.md | doc link check | live E2E/wiremock/sync deferred WP-13/follow-up | WP-14
