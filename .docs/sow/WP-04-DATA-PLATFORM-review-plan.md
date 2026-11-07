# WP-04 — Data Platform, Schema & Persistence Review Plan

| Field | Value |
|---|---|
| WP | `WP-04` |
| Contract | `.docs/sow/WP-04-DATA-PLATFORM.md` |
| Review branch | `review/wp-04-data-platform` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_review` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present and consistent.
- [x] Requirement traceability cited in §9 maps to spec §14/§17 and Appendix A/B.

## 2. Specification conformance

- [x] Appendix A canonical DDL — evidence: `.tools/test-migrations.sh` schema diff.
- [x] Specification §14 (data), §17 (operations) — evidence: `.docs/product/database.md`, migrations.
- [x] Appendix B §B.1.3 — evidence: `.docs/product/database.md` identifiers and pagination.

## 3. Deliverables

- [x] `migrations/0001_init.sql` + `0002_analytics_views.sql` — evidence: `.tools/test-migrations.sh` OK.
- [x] SQLx offline metadata path assigned to WP-06 — evidence: documented handoff; no premature queries.
- [x] Repository implementations assigned to WP-06 — evidence: documented handoff.
- [x] Pool/timeout contract defined — evidence: WP-04 §3.1.
- [x] Optimistic concurrency at DB level — evidence: `.tools/test-constraints.sh` (one winner).
- [x] Redis key builder + TTL constants — evidence: `crates/infrastructure/src/redis_keys.rs`.
- [x] HNSW index/vector/scope constraints — evidence: `0001_init.sql`; retrieval eval WP-08.
- [x] Idempotent seeds — evidence: `scripts/seed-demo.sh` (12 cards, 36 events; re-runnable).
- [x] Integrity SQL + live checker — evidence: `scripts/data-integrity.sql`, `scripts/integrity-check.sh` OK.
- [x] Backup/restore scripts + manifest — evidence: `scripts/backup.sh`; `scripts/restore-drill.sh` OK.
- [x] Retention rules documented — evidence: `.docs/product/database.md`; jobs WP-20/WP-23.
- [x] `.docs/product/database.md` ERD/dictionary/index rationale — evidence: file exists.
- [x] Data-ownership fitness test — evidence: `.tools/check-architecture-fitness.sh`.
- [x] Constraint, concurrency and migration suites — evidence: 8/8, schema diff green.

## 4. Acceptance criteria

- [x] AC1 `make migrate` on empty DB matches Appendix A — evidence: `.tools/test-migrations.sh`.
- [x] AC2 every CHECK/UNIQUE/FK has a negative test — PARTIAL: `.tools/test-constraints.sh` now has 18 negatives covering every constraint family and the core aggregates (key_prefix, description length, memory scope, domain_events seq, duplicate keys, FK, vector dimension); exhaustive per-constraint coverage (~145 objects) is a follow-up owned by WP-21.
- [x] AC3 two concurrent moves → one success, one `VERSION_CONFLICT` — evidence: constraint suite optimistic-concurrency case.
- [x] AC4 two workers cannot lease the same card — DEFERRED to WP-06/WP-13 (lease allocator); schema `SKIP LOCKED` constraints in place.
- [x] AC5 HNSW scoped retrieval / no cross-scope leakage — DEFERRED to WP-08 (retrieval function and eval).
- [x] AC6 hybrid score formula within 1e-6 — DEFERRED to WP-08.
- [x] AC7 integrity job detects corruption and exits non-zero — evidence: `scripts/integrity-check.sh`; mismatch fixture in WP progress log.
- [x] AC8 backup/restore drill verifies rows/checksums — evidence: `scripts/restore-drill.sh` OK.
- [x] AC9 data-ownership fitness fails on cross-context query — evidence: architecture fitness self-test 15/15.
- [x] AC10 query budgets — DEFERRED to WP-21 (performance suite).

## 5. Implementation and tests

- [x] Schema equivalence — `.tools/test-migrations.sh`.
- [x] Constraint negatives — `.tools/test-constraints.sh`.
- [x] Analytics view values — `.tools/test-analytics-views.sh` 25 assertions.
- [x] Seeds idempotent, integrity clean — ran against the dev database.
- [x] No real external API in tests.
- [x] Coverage thresholds met (schema/SQL WP; suites green).

## 6. Security and quality gates

- [x] No secret in migrations or seeds; role/DSN via environment.
- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `.tools/test-migrations.sh`, `.tools/test-constraints.sh`, `.tools/test-analytics-views.sh` green; backup/restore drill OK.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/database.md` complete.
- [x] `.docs/DATA-MODEL.md` consistent (checked).
- [x] `.docs/adr/registry.md` consistent (ADR-0003).
- [x] `AGENTS.md` indexes updated for this review plan.
- [x] `.docs/sow/SOW.md` and WP-04 status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED`.
- [ ] QA evidence: reproduced gates (harness 16, sow-lint 31, doc links 143, schema diff, constraints 18/18, analytics 25/25, integrity, backup/restore).
- [x] First QA pass returned `CHANGES_REQUESTED` for three blocking defects:
  - B1 constraint coverage: expanded from 7 to 18 negatives across every family; exhaustive per-object coverage still open (owner WP-21).
  - B2 restore drill: rewritten to verify the manifest SHA-256, per-table content checksums, integrity checks, RTO and to drop the scratch database.
  - B3 least privilege: `infra/postgres/init-roles.sh` creates `ailab_migrate`, `ailab_app` and `ailab_readonly`; the migrate service uses `ailab_migrate`, the app uses `ailab_app` (CR-0007).

## 9. Deferred items and residual risks

- [x] AC4 lease allocator — owner: Dev, WP-06/WP-13.
- [x] AC5/AC6 retrieval and hybrid score — owner: Dev, WP-08.
- [x] AC10 query budgets — owner: QA, WP-21.
- [x] Retention jobs — owner: Dev, WP-20/WP-23.
- [ ] B1 exhaustive per-constraint negative coverage — owner: QA, WP-21; 18 family/core cases delivered.
- [ ] No unexplained deferral remains.

## 10. Decisions

- [x] ADR-0003 PostgreSQL as the sole system of record — recorded.
- [x] CR-0007 adds the least-privilege database role passwords to the environment contract.
