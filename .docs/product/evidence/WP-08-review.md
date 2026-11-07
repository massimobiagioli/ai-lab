# WP-08 Independent Review

| Field | Value |
|---|---|
| Reviewer | review-agent (independent subagent) |
| Author | Dev (agent) |
| Date | 2026-11-07 |
| Scope | Lab aggregate/settings, scoped memory, chunking, secret quarantine, dedupe, hybrid retrieval, compaction |
| Verdict | `APPROVED` |

One review pass returned `CHANGES_REQUESTED`; the author fixed the incomplete lab-settings validation and the re-review approved.

## Final acceptance results

| AC | Result | Evidence |
|---|---|---|
| AC1 lab requires a Tech Lead | PASS | `Lab::create` → `validate_team`. |
| AC2 settings ranges | PASS | Full §3.1 schema modelled; lower and upper bounds enforced; boundary harness 32/32; unknown keys via WP-16 JSON Schema. |
| AC3 deterministic chunking | PASS | 800/120 word split, identical boundaries. |
| AC4 secret quarantine | PASS (domain) / DEFERRED event | `MEMORY_QUARANTINED` before chunk/embed; blocked persistence + security event WP-16. |
| AC5 near-duplicate dedupe | PASS | Cosine ≥ 0.92 dedupe. |
| AC6 scope filtering | PASS | Scope filter before scoring; leakage test. |
| AC7 hybrid formula | PASS | Within 1e-9 on the fixed fixture; nDCG WP-21. |
| AC8 cold/warm cache | DEFERRED | WP-16. |
| AC9 compaction/retention | PASS | >30d compact, >180d delete, pins preserved. |
| AC10 p95/explorer | DEFERRED | WP-21/WP-16/WP-19. |
| AC11 lab delete cascade | DEFERRED | WP-16; DB `ON DELETE CASCADE` present. |

## Blocking defect found and resolved

1. Lab settings modelled only five fields and skipped the documented upper bounds and the autonomy/scheduler/office/maintenance blocks. Resolved: full schema added and every range enforced with boundary tests.

## Reproduced evidence

- `scripts/harness-lint.sh` → 16 checks.
- `scripts/sow-lint.sh` → 31 checks.
- `.tools/check-doc-links.sh` → 143 links.
- `cargo fmt --all --check`, `cargo clippy -D warnings`, `cargo test -p ailab-domain` → 52 passed.
- Settings boundary harness 32/32; memory invariant harness 22/22 (all prefixes incl. AIza, chunking, scope, dedupe, compaction).

## Reviewer statement

The WP-08 artifacts satisfy the lab and memory contract. The deferred criteria are downstream verifications owned by WP-16, WP-19 and WP-21; the domain contract they build on is delivered. Approved for WP closure.
