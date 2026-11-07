# WP-12 — SCM & GitHub Integration Review Plan

| Field | Value |
|---|---|
| WP | `WP-12` |
| Contract | `.docs/sow/WP-12-SCM-GITHUB.md` |
| Review branch | `review/wp-12-scm-github` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `approved` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present and consistent.
- [x] Requirement traceability maps to spec §10/§16, Appendix B §B.4 and Appendix C §C.4.

## 2. Specification conformance

- [x] Specification §10 SCM and §16 security — evidence: `crates/scm/src/lib.rs`, `.docs/product/github.md`.
- [x] Appendix B §B.4 webhook/merge contracts — evidence: `WebhookVerifier`, `merge_pull_request`.
- [x] Appendix C §C.4 PR conventions — evidence: `PrConventions`, tests.

## 3. Deliverables

- [x] `ScmProvider` port and domain-neutral types — evidence: `lib.rs`.
- [x] Core adapter methods over an injectable transport — evidence: `GitHubClient`, `HttpTransport`, `FakeTransport`.
- [x] Credential verification and scope parsing — evidence: `verify_credential` (active/expired).
- [x] In-memory per-run token broker with TTL and revocation — evidence: `TokenBroker`, random tokens.
- [x] Branch/PR conventions and AI-Lab trailer — evidence: `PrConventions`, `ensure_commit_trailers`, tests.
- [x] HMAC, replay window, delivery idempotency — evidence: `WebhookVerifier`, constant-time `verify_slice`.
- [x] Rate-limit mapping/parsing — evidence: `parse_rate_limit`, `send`.
- [x] Stable codes/statuses — evidence: `ScmError`.
- [x] Fake transport/security tests — evidence: 9 scm tests.
- [x] `.docs/product/github.md` — evidence: file.

## 4. Acceptance criteria

- [x] AC1 PAT scopes verify; missing scope fails with `expired` — evidence: `verify_credential` returns `Expired` on 403/404 or a classic scope header without `repo`; tests.
- [x] AC2 branch/commit/PR idempotency by run/card — DEFERRED to WP-13 orchestration; the adapter treats GitHub `422 already exists` as success.
- [x] AC3 PR title/body/trailers; reviewer QA ≠ author — evidence: `PrConventions` renders Card/Summary/Changes/Acceptance/Verification/Risks and the full trace; `commit_files` enforces the Run/Card/Agent trailers; `open_pull_request` separation; QA identity WP-13.
- [x] AC4 squash-only merge, green checks, reviewer ≠ author — evidence: `merge_pull_request`; branch-protection detection WP-13.
- [x] AC5 invalid signature 401, duplicate once, replay >5 min rejected — evidence: constant-time HMAC, delivery idempotency, replay window; test.
- [x] AC6 429/403 secondary rate limits back off/queue — evidence: `RateLimited` with `Retry-After`; retry/queue WP-13.
- [x] AC7 PAT never leaks; sandbox gets a short-lived random token revoked after the run — evidence: `Secret<String>`, `TokenBroker` random UUID token, TTL/revoke tests; log wiring WP-20.
- [x] AC8 externally merged PRs reconcile — DEFERRED to WP-13.
- [x] AC9 path traversal and force-push rejected — evidence: `FileChange`, `BranchName`; no force-push API.
- [x] AC10 `make smoke-live` — DEFERRED (opt-in, needs a real repo).

## 5. Implementation and tests

- [x] Repo/branch path validation, webhook HMAC/replay/idempotency, checks mapping, merge guards, PR conventions, token broker — tests.
- [x] No network in tests (`FakeTransport`).
- [x] Coverage thresholds met (scm crate).

## 6. Security and quality gates

- [x] Constant-time HMAC compare; PAT held in `Secret`; random short-lived sandbox token.
- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p ailab-scm` green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/github.md` updated.
- [x] `.docs/adr/registry.md` consistent (ADR-0011, ADR-0013).
- [x] `AGENTS.md` indexes updated for this review plan.
- [x] `.docs/sow/SOW.md` and WP-12 status/progress updated.

## 8. QA review

- [x] QA subagent verdict: `APPROVED`.
- [x] QA evidence: reproduced gates (harness 16, sow-lint 31, doc links 143, 10 scm tests, clippy, HMAC/idempotency/token broker).
- [x] First QA pass returned `CHANGES_REQUESTED` for the incomplete PR body/trace and missing commit-trailer enforcement; fixed with the full Appendix C.4.1 body, full trace, trailer enforcement and tests.

## 9. Deferred items and residual risks

- [x] AC2/AC4/AC6/AC8 idempotency, protection, retries, reconciliation — owner: Dev, WP-13.
- [x] AC10 live smoke — owner: QA, opt-in release check.
- [x] No unexplained deferral remains.

## 10. Decisions

- [x] ADR-0011 separation of duties and ADR-0013 remote exposure recorded.
