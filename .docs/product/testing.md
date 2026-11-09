# AI-Lab Testing Guide

| Field | Value |
|---|---|
| Version | 1.0.0 |
| Status | Review (P14) |
| Related | `.docs/product/quality-gates.md`, `.docs/sow/WP-21-TESTING.md` |

## Commands

| Layer | Command |
|---|---|
| Backend unit and integration | `cargo nextest run --workspace` |
| Backend lint and format | `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings` |
| Web unit | `npm --prefix web test` |
| Web lint and build | `npm --prefix web run lint && npm --prefix web run build` |
| E2E and accessibility | `.tools/run-e2e.sh` (official Playwright image, axe on `/`, `/settings`, `/memory`, `/design`) |
| Load and chaos | `.tools/run-load.sh`, `.tools/chaos-worker-kill.sh` |
| Coverage | `.tools/check-coverage.sh` (backend 80, domain 90, web 80) |
| Migrations, constraints and views | `.tools/test-migrations.sh`, `.tools/test-constraints.sh`, `.tools/test-analytics-views.sh` |
| Supply chain | `gitleaks detect --source . --no-git`, `cargo deny check`, `npm --prefix web audit --audit-level=high` |

## Fixtures and seed

- `scripts/seed-demo.sh` writes the deterministic demo dataset (lab, four agents, project, SOW, plan, 12 cards and events). The E2E, load and chaos runs assume it.
- `crates/testkit` holds the in-memory fakes and repository fixtures used by the backend suites.
- The API supports `AILAB_DEV_FAKES=1` for the in-memory LLM/SCM harness; the Compose core stack sets it for the demo.

## Flaky policy

- A test that fails intermittently is quarantined with an owner and a card; it is never silently retried in CI.
- Time-dependent tests use the `Clock` port; no wall-clock sleeps in domain tests.
- Playwright uses one worker locally and retries only on CI with `--retries=1`; retries never hide a deterministic failure.

## Coverage status

The 2026-11-09 review measured backend 77.56 %, domain 88.99 % and web 76.06 % line coverage against the 80/90/80 thresholds. The gap and its owners are recorded in `.docs/product/qa/WP-24-FRONTEND-HARDENING-qa-report.md`; the gate is wired in `.tools/check-coverage.sh`.
