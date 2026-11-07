# WP-12 Independent Review

| Field | Value |
|---|---|
| Reviewer | review-agent (independent subagent) |
| Author | Dev (agent) |
| Date | 2026-11-07 |
| Scope | SCM port, GitHub adapter, credentials, token broker, webhooks, PR/merge conventions, rate limits |
| Verdict | `APPROVED` |

One review pass returned `CHANGES_REQUESTED`; the author fixed the defects and the re-review approved.

## Final acceptance results

| AC | Result | Evidence |
|---|---|---|
| AC1 PAT scopes / expired | PASS | `verify_credential` active/expired; tests. |
| AC2 run/card idempotency | DEFERRED | WP-13 orchestration. |
| AC3 PR conventions and trailers | PASS | Full body/trace; commit trailer enforcement; test. |
| AC4 squash-only merge guards | PASS | Checks green and author≠reviewer; protection WP-13. |
| AC5 webhook HMAC/replay/idempotency | PASS | Constant-time `verify_slice`, replay window, delivery idempotency. |
| AC6 rate-limit backoff/queue | DEFERRED | WP-13; mapping present. |
| AC7 PAT never leaks; short-lived sandbox token | PASS | `Secret`, random UUID token, TTL/revoke. |
| AC8 reconciliation | DEFERRED | WP-13. |
| AC9 traversal/force-push | PASS | Path/branch validation; no force-push API. |
| AC10 live smoke | DEFERRED | Opt-in release check. |

## Blocking defects found and resolved

1. The PR body lacked Changes, Verification, Risks/rollback and the full trace. Resolved in `PrConventions`.
2. Agent commits did not enforce the Run/Card/Agent trailers. Resolved with `ensure_commit_trailers` and a test.

## Reproduced evidence

- `scripts/harness-lint.sh` → 16 checks.
- `scripts/sow-lint.sh` → 31 checks.
- `.tools/check-doc-links.sh` → 143 links.
- `cargo fmt --all --check`, `cargo clippy -D warnings`, `cargo test -p ailab-scm` → 10 passed.

## Reviewer statement

The WP-12 artifacts satisfy the SCM contract. The deferred criteria are downstream verifications owned by WP-13, WP-14, WP-16 and WP-20; the adapter and security primitives they build on are delivered. Approved for WP closure.
