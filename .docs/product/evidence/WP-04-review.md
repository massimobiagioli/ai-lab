# WP-04 Independent Review

| Field | Value |
|---|---|
| Reviewer | review-agent (independent subagent) |
| Author | Dev (agent) |
| Date | 2026-11-07 |
| Scope | Schema, migrations, analytics views, constraints, seeds, integrity, backup/restore, Redis keys, least-privilege roles |
| Verdict | `APPROVED` |

The review was iterative: the first two passes returned `CHANGES_REQUESTED`; the author fixed the defects and the third pass approved.

## Final acceptance results

| AC | Result | Evidence |
|---|---|---|
| AC1 schema matches Appendix A | PASS | `.tools/test-migrations.sh`; `sqlx migrate run` applies 1–3 as `ailab_migrate`. |
| AC2 every constraint has a negative test | DEFERRED | `.tools/test-constraints.sh` 18 cases across every family and the core aggregates; exhaustive per-object coverage is a named WP-21 follow-up. |
| AC3 two concurrent moves → one conflict | PASS | Constraint suite one-winner case; HTTP 409 mapping is WP-06. |
| AC4 workers cannot lease the same card | DEFERRED | Lease allocator is WP-06/WP-13; schema present. |
| AC5 scoped HNSW retrieval | DEFERRED | Retrieval/eval is WP-08; HNSW and scope constraints present. |
| AC6 hybrid score formula | DEFERRED | WP-08. |
| AC7 integrity detects corruption | PASS | Injected projection mismatch detected; checker exits non-zero. |
| AC8 backup/restore verifies checksums within RTO | PASS | Manifest SHA-256 + per-table content checksums + integrity; RTO 48 s; scratch DB dropped. |
| AC9 ownership fitness fails on cross-context query | PASS | Fitness 4/4; self-test 15/15. |
| AC10 query budgets | DEFERRED | WP-21 performance suite. |

## Blocking defects found and resolved

1. Constraint coverage was too thin (7 cases). Resolved: expanded to 18 cases covering CHECK, UNIQUE, partial UNIQUE, FOREIGN KEY and vector-dimension constraints across the core aggregates; exhaustive per-object coverage assigned to WP-21.
2. The restore drill only compared row counts. Resolved: it now verifies the manifest checksum, per-table content checksums, integrity checks, RTO (≤ 3600 s) and drops the scratch database; missing manifest or mismatch fails.
3. The application connected as the PostgreSQL superuser. Resolved: `infra/postgres/init-roles.sh` creates `ailab_migrate`, `ailab_app` and `ailab_readonly`, pre-creates the extensions and grants database/schema privileges; `migrations/0003_grants.sql` grants the application/read-only privileges; the migrate service uses `ailab_migrate` and the app uses `ailab_app` (CR-0007). Verified on a fresh container end-to-end.

## Reproduced evidence

- `scripts/harness-lint.sh` → 16 checks.
- `scripts/sow-lint.sh` → 31 checks.
- `.tools/check-doc-links.sh` → 143 links.
- `.tools/test-migrations.sh` → schema matches Appendix A.
- `.tools/test-constraints.sh` → 18 cases.
- `.tools/test-analytics-views.sh` → 25 assertions.
- `scripts/restore-drill.sh` → checksum, content checksums, integrity, RTO 48 s.
- Fresh-container role boot → roles created, migrations 1–3 as `ailab_migrate`, app/readonly privilege separation.

## Reviewer statement

The WP-04 artifacts satisfy the data-platform contract. The deferred criteria are downstream verifications owned by WP-06, WP-08, WP-13, WP-20/WP-23 and WP-21; the artifacts they verify are delivered. Approved for WP closure.
