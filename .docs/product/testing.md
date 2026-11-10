# AI-Lab Testing Guide

| Field | Value |
|---|---|
| Version | 1.0.0 |
| Status | Review (P14) |
| Related | `.docs/product/quality-gates.md`, `.docs/sow/WP-21-TESTING.md` |

## Commands

All application commands run in Docker (ADR-0027). Use the Make targets or the `scripts/run-{rust,web,node,python}.sh` wrappers; host toolchains are not supported.

| Layer | Command |
|---|---|
| Backend unit and integration | `make test` or `scripts/run-rust.sh cargo test --workspace` |
| Backend lint and format | `make fmt-check` and `make lint` |
| Web unit | `make frontend-test` |
| Web lint and build | `make frontend-lint` and `make frontend-build` |
| E2E and accessibility | `make e2e` (official Playwright image, axe on `/`, `/settings`, `/memory`, `/design`) |
| BDD E2E suite | `make bdd` (preflight-gated, make-only) |
| BDD live key | `make bdd-set-api-key` (hidden prompt, creates or rotates the DeepSeek provider) |
| Load and chaos | `.tools/run-load.sh`, `.tools/chaos-worker-kill.sh` |
| Coverage | `.tools/check-coverage.sh` (backend 80, domain 90, web 80) |
| Migrations, constraints and views | `.tools/test-migrations.sh`, `.tools/test-constraints.sh`, `.tools/test-analytics-views.sh` |
| Supply chain | `make supply-chain-test`, `gitleaks detect --source . --no-git` |

## Fixtures and seed

- `scripts/seed-demo.sh` writes the deterministic demo dataset (lab, four agents, project, SOW, plan, 12 cards and events). The E2E, load and chaos runs assume it.
- `crates/testkit` holds the in-memory fakes and repository fixtures used by the backend suites.
- The API supports `AILAB_DEV_FAKES=1` for the in-memory LLM/SCM harness; the Compose core stack sets it for the demo.

## BDD E2E suite

The BDD suite is generated from `.docs/e2e/BDD.md` and lives in `web/e2e-bdd/`.

- **Only entry point:** `make bdd`. The Makefile sets `AILAB_BDD_VIA_MAKE=1`; `scripts/run-bdd.sh` and `web/e2e-bdd/playwright.config.ts` refuse direct invocation with a clear message.
- **Suites:** the 480 scenarios are split into `core`, `kanban`, `delivery`, `platform`, `office` and `management`. Run one with `make bdd SUITE=core`, run the ordered prefix with `make bdd UP_TO=delivery`, run all with `make bdd`, and list the counts with `make bdd-list`. Each scenario carries a `@suite-<id>` tag; the runner passes the matching Playwright `--grep`.
- **Preflight:** `scripts/bdd-preflight.mjs` reads `.docs/e2e/req.md` and exits `2` when a manual item is still `[ ]`, before feature generation or any test starts. The checklist is manual-only; the current entry is the DeepSeek API key for live runs.
- **Live key:** `make bdd-set-api-key` reads the key from a hidden prompt, validates it, and creates or rotates the DeepSeek provider. The key is never an argument, a file, an environment variable, a log or a report.
- **Features:** `scripts/run-node.sh .tools/gen-bdd-features.mjs --write` regenerates one feature per BDD area; `--check` fails on drift. Never hand-edit the generated `.feature` files.
- **Steps:** curated reusable steps live in `web/e2e-bdd/steps/`; `scripts/run-node.sh .tools/gen-bdd-steps.mjs --write` regenerates `generated.steps.ts` for narrative scenarios that lack a curated step. The semantic dispatcher in `support/generated-step-handler.ts` fails loudly with `UnsupportedStepError` rather than silently passing an unimplemented assertion.
- **Coverage:** `scripts/run-node.sh .tools/check-bdd-coverage.mjs` enforces the 480 ids, duplicate/missing detection and zero pending steps. `--report-only` prints the pending inventory without failing.
- **Fakes:** automated scenarios use `AILAB_DEV_FAKES=1` and never call the real DeepSeek or GitHub APIs. `@live` scenarios run only with `AILAB_BDD_LIVE=1`.
- **Output:** `make bdd` writes the full console output to `web/e2e-bdd/reports/make-bdd.log` (override with `AILAB_BDD_LOG`). `web/e2e-bdd/reports/results.json` maps every result to its `E2E-XXX` id; HTML, JUnit, traces, screenshots and videos are written under `reports/`.
- **Policy:** a flaky scenario is tagged `@quarantine`, assigned an owner and a card, and is never silently retried.

## Flaky policy

- A test that fails intermittently is quarantined with an owner and a card; it is never silently retried in CI.
- Time-dependent tests use the `Clock` port; no wall-clock sleeps in domain tests.
- Playwright uses one worker locally and retries only on CI with `--retries=1`; retries never hide a deterministic failure.

## Coverage status

The 2026-11-09 review measured backend 77.56 %, domain 88.99 % and web 76.06 % line coverage against the 80/90/80 thresholds. The gap and its owners are recorded in `.docs/product/qa/WP-24-FRONTEND-HARDENING-qa-report.md`; the gate is wired in `.tools/check-coverage.sh`.
