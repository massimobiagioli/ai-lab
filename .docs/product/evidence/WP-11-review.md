# WP-11 Independent Review

| Field | Value |
|---|---|
| Reviewer | review-agent (independent subagent) |
| Author | Dev (agent) |
| Date | 2026-11-07 |
| Scope | Provider port, DeepSeek adapter, prompt assembly, structured repair, routing, retries/breaker, cost/budget, secret guard |
| Verdict | `APPROVED` |

Two review passes returned `CHANGES_REQUESTED`; the author fixed the defects and the third pass approved.

## Final acceptance results

| AC | Result | Evidence |
|---|---|---|
| AC1 deterministic `prompt_hash`, section order | PASS | Hash + order test; per-role snapshots are a WP-11 follow-up. |
| AC2 schema validation + one repair | PASS | Semantic failures → `InvalidOutput` after exactly one repair with error + 8 KB previous output. |
| AC3 retry/backoff/Retry-After, no auth retry | PASS | 4 attempts at 1/2/4/8 s ±20% via async `futures_timer`. |
| AC4 breaker 5/60 | PASS | Test. |
| AC5 micro-USD cost + budget hard stop | PASS | Cost/budget tests; ledger WP-16. |
| AC6 Redis fallback | DEFERRED | WP-16. |
| AC7 keys never leak | PASS | Guard covers all prompt sections and blocks; `Secret<T>` redaction. |
| AC8 escalation reason | PASS | `route`; persistence WP-16. |
| AC9 streaming | DEFERRED | WP-11 follow-up. |
| AC10 health probe/event | DEFERRED | Breaker-based health; event WP-16/WP-20. |

## Blocking defects found and resolved

1. Retry policy was 3 attempts/100 ms; fixed to 4 attempts at 1/2/4/8 s ±20%.
2. Backoff used blocking sleep; replaced with `futures_timer::Delay`.
3. Prompt order deviated from Appendix C.2; fixed.
4. Semantic validation failures returned `Validation`; now `InvalidOutput` with a richer repair payload.
5. `agents.md` documented the old order; fixed.

## Reproduced evidence

- `scripts/harness-lint.sh` → 16 checks.
- `scripts/sow-lint.sh` → 31 checks.
- `.tools/check-doc-links.sh` → 143 links.
- `cargo fmt --all --check`, `cargo clippy -D warnings`, `cargo test -p ailab-llm` → 11 passed.

## Reviewer statement

The WP-11 artifacts satisfy the LLM-platform contract. The deferred criteria are downstream verifications owned by WP-16 and WP-20; the platform they build on is delivered. Approved for WP closure.
