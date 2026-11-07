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
- [x] AC2 every CHECK/UNIQUE/FK has a negative test — evidence: `.tools/test-constraints.sh` 8 cases.
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
- [ ] QA evidence: reproduced gates (harness 16, sow-lint 31, doc links 142, schema diff, constraints 8/8, analytics 25/25, integrity, backup/restore).

## 9. Deferred items and residual risks

- [x] AC4 lease allocator — owner: Dev, WP-06/WP-13.
- [x] AC5/AC6 retrieval and hybrid score — owner: Dev, WP-08.
- [x] AC10 query budgets — owner: QA, WP-21.
- [x] Retention jobs — owner: Dev, WP-20/WP-23.
- [x] No unexplained deferral remains.

## 10. Decisions

- [x] ADR-0003 PostgreSQL as the sole system of record — recorded.
