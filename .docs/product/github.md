# AI-Lab GitHub (SCM)

| Field | Value |
|---|---|
| Document | `.docs/product/github.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/contracts.md`, `.docs/product/security.md`, `crates/scm/src/lib.rs` |

GitHub is the only v1 SCM, behind an `ScmProvider` port. The adapter talks over an injectable HTTP transport, so tests run without network.

## 1. Port and types

- `verify_credential`, `create_branch`, `commit_files`, `open_pull_request`, `get_checks`, `submit_review`, `merge_pull_request`, `comment`.
- `RepoRef` (`owner/repo`), `BranchName` (`ailab/{card_key}-{slug}`), `FileChange`.
- Path traversal (`..`, leading `/`, `.git/`) and unsafe branch characters are rejected.
- `ScmError` maps to stable codes and HTTP statuses (`SCM_AUTH` 401, `SCM_FORBIDDEN` 403, `SCM_NOT_FOUND` 404, `SCM_RATE_LIMITED` 429, `SCM_CONFLICT`/`SCM_REPLAY` 409, `SCM_VALIDATION` 422, `SCM_TRANSPORT` 502).

## 2. Conventions

- Branch: `ailab/{card_key}-{slug}`.
- PR title: `[{CARD_KEY}] {CARD_TITLE}`.
- PR body: Card, Summary, Changes, Acceptance criteria, Verification, Risks / rollback, and the full AI-Lab trace (`AI-Lab-Run`, `AI-Lab-Card`, `AI-Lab-Agent`, `AI-Lab-Model`, `AI-Lab-Attempt`). Agent commits must carry the `AI-Lab-Run`/`AI-Lab-Card`/`AI-Lab-Agent` trailers.
- Reviewer must differ from the author; merge is squash-only and requires green checks.

## 3. Checks and merge

- `get_checks` maps `success/neutral/skipped` to passed, `failure/cancelled/timed_out/action_required` to failed, everything else to pending.
- `merge_pull_request` rejects reviewer = author and non-green checks before calling GitHub.
- Branch creation treats GitHub `422 Reference already exists` as idempotent success.

## 4. Webhooks

- `X-Hub-Signature-256` is verified with HMAC-SHA256.
- A delivery older than 5 minutes or a duplicate `X-GitHub-Delivery` is rejected with `SCM_REPLAY`.
- Invalid signatures are rejected with `SCM_INVALID_SIGNATURE`.
- Verified events carry delivery id, event, action and payload for WP-13 processing.

## 5. Sandbox credentials

`TokenBroker` issues a short-lived repo-scoped token per run and revokes it after the run. The token is a `Secret<String>` and never appears in logs or errors. Real GitHub App token exchange is deferred.

## 6. Tests

`cargo test -p ailab-scm` covers repo/branch/path validation, PR conventions, HMAC success/failure/replay/age, checks mapping, merge separation and green checks, PR request construction, and token issue/revoke.

## 7. Deferred to consumer WPs

- Wiremock suite and live `make smoke-live`: WP-12 follow-up (needs a test repo and PAT).
- Retries/backoff/circuit breaker for GitHub: WP-13.
- Webhook async processing and sync/reconciliation: WP-13.
- Problem Details mapping: WP-16.
- UI: WP-19.

## 8. References

- `.docs/sow/WP-12-SCM-GITHUB.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §10
- `.docs/technical-specification/APPENDIX-B-CONTRACTS.md`
- `crates/scm/src/lib.rs`
