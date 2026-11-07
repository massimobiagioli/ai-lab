# WP-09 Independent Review

| Field | Value |
|---|---|
| Reviewer | review-agent (independent subagent) |
| Author | Dev (agent) |
| Date | 2026-11-07 |
| Scope | Card aggregate, 36-case transition matrix, DoR/DoD codes, WIP, separation of duties, reopen, events |
| Verdict | `APPROVED` |

One review pass returned `CHANGES_REQUESTED`; the author fixed the DoR estimate predicate and the re-review approved.

## Final acceptance results

| AC | Result | Evidence |
|---|---|---|
| AC1 36 transitions, denied → INVALID_TRANSITION | PASS | Matrix test 12 allowed / 24 denied. |
| AC2 exact DoR/DoD codes and fields | PASS | 8 DoR + 9 DoD evaluators and per-code tests; two DoR codes deferred to WP-10/WP-16. |
| AC3 100 concurrent cards | DEFERRED | WP-16; WP-07 allocator delivered. |
| AC4 concurrent transition conflict | DEFERRED | Optimistic concurrency primitive; HTTP mapping WP-16. |
| AC5 READY prerequisites incl. estimate set | PASS | Estimate 13 rejected with `DOR_ESTIMATE_INVALID`. |
| AC6 DONE prerequisites | PASS | `dod_violations`. |
| AC7 separation of duties | PASS | `SEPARATION_OF_DUTIES`; merge policy in WP-15. |
| AC8 WIP limits | PASS | DOING and REVIEW tests emit `WIP_LIMIT_EXCEEDED`. |
| AC9 one immutable event per transition | PASS | `kanban.card.created|moved`; SQL projection WP-16. |
| AC10 performance | DEFERRED | WP-21. |

## Blocking defect found and resolved

1. Estimate 13 bypassed DoR because the guard only checked presence. Resolved: the evaluator enforces `{1,2,3,5,8}` and a regression test covers estimate 13.

## Reproduced evidence

- `scripts/harness-lint.sh` → 16 checks.
- `scripts/sow-lint.sh` → 31 checks.
- `.tools/check-doc-links.sh` → 143 links.
- `cargo fmt --all --check`, `cargo clippy -D warnings` → clean.
- `cargo test -p ailab-domain` → 55 passed.
- Matrix 12/24; per-code DoR/DoD; WIP DOING/REVIEW; reopen parent link.

## Reviewer statement

The WP-09 artifacts satisfy the Kanban contract. The deferred criteria are downstream verifications owned by WP-16 and WP-21; the domain engine they build on is delivered. Approved for WP closure.
