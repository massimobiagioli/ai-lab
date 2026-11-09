# WP-23 — Maintenance & Continuous Evolution Review Plan

| Field | Value |
|---|---|
| WP | `WP-23` |
| Contract | `.docs/sow/WP-23-MAINTENANCE.md` |
| Review branch | `review/p16-wp23-maintenance` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_progress` |

## 1. Contract completeness

- [x] Header, objective, scope, job catalog, policies, tasks, acceptance criteria, DoD, risks and references present.
- [x] CR-0019 reopens WP-23 with a status-history entry and a progress log.

## 2. Specification conformance

- [x] The `maintenance_jobs` catalog is seeded (`migrations/0001_init.sql`).
- [x] The scheduler loads due jobs, runs handlers, records `last_status`/`last_error`/`next_run_at` and computes the next cron (`crates/orchestrator/src/maintenance.rs`).
- [x] The worker ticks the scheduler every loop (`crates/worker/src/main.rs`).
- [x] `stale_card_scan` reports non-DONE/ICEBOX cards older than `stale_days` (T05).
- [x] Renovate policy verified (`renovate.json`): weekly, grouped patch/minor, major approval, vulnerability alerts, TypeScript major pinned by ADR-0024 (T02).
- [ ] Dependency/EOL automation job (T02 remainder), CV reflection backfill (T03), memory compaction/retention (T04), integrity/backup/cert/disk (T06), cost review (T07), flow review (T08) and evolution docs (T09).

## 3. Deliverables

- [x] Scheduler and stale-card scan with four tests.
- [x] Worker wiring.
- [ ] Remaining handlers and alerts.

## 4. Security

- [x] The scheduler is bounded (due jobs only), idempotent and non-destructive; failures are recorded, not retried blindly.
- [x] No secret added; the new `cron` dependency is permissive and pinned.

## 5. Review and QA independence

- [x] The reviewer is not the author; the QA verdict is a separate file.
- [ ] Final approval after the remaining handlers land.

## 6. Gates and evidence

- [x] `cargo test -p ailab-orchestrator maintenance` (4 passed), `cargo clippy` clean.
- [x] `scripts/harness-lint.sh`, `scripts/sow-lint.sh`, `.tools/check-doc-links.sh`, `make check`.

## 7. Verdict

The scheduler and the stale-card scan are delivered and tested; WP-23 stays `progress` until T03/T04/T06–T09 land.

## Progress log

2026-11-09 | P16 | crates/orchestrator/src/maintenance.rs, crates/worker/src/main.rs | scheduler + stale scan; 4 tests; Renovate verified | remaining handlers open | P17
