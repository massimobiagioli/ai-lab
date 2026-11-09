# WP-24 — Frontend & QA Hardening Follow-up Review Plan

| Field | Value |
|---|---|
| WP | `WP-24` |
| Contract | `.docs/sow/WP-24-FRONTEND-HARDENING.md` |
| Review branches | `review/p11-wp24-e2e`, `review/p12-wp24-hardening`, `review/p13-wp24-coverage` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_progress` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks and references present and consistent.
- [x] CR-0016 creates WP-24; the history and progress log record the start.

## 2. Specification conformance

- [x] Playwright installed and configured (`web/playwright.config.ts`, `.tools/run-e2e.sh`).
- [x] Axe a11y on `/`, `/settings`, `/memory`, `/design` — zero critical/serious.
- [x] Primary route smoke journeys and the dashboard/onboarding.
- [x] k6 load scenario (`infra/load/board.js`, `.tools/run-load.sh`), worker-kill chaos script and the malformed-input fuzz test — P12; live execution green after the stack reset (see evidence).
- [ ] Coverage thresholds and CI wiring — P13.
- [ ] Remaining six dashboards, cost/budget alerts, post-incident template, injected-integrity test, synthetic alert test and runbook tabletop — WP24-T07.

## 3. Deliverables

- [x] Playwright E2E (6 tests) and the axe suite.
- [x] `.tools/run-e2e.sh` wrapper + docs.
- [x] Load/chaos scripts, the fuzz test and the live runs — P12.
- [ ] Coverage/CI — P13.
- [ ] WP24-T07 observability/tabletop residuals.

## 4. Acceptance criteria

1. [x] Playwright J1–J4 smoke/primary routes pass — 6/6.
2. [x] axe zero critical/serious on the tested routes.
3. [x] Live load/soak: 1 VU, 1.66 req/s, 0% failed, p95 2.6 ms; excess 96 req/s correctly returns 429.
4. [x] Chaos: worker killed and restarted healthy; malformed-input fuzz test green.
5. [ ] Coverage thresholds — P13.
6. [ ] Weighted CI wiring — P13.

## 5. Implementation and tests

- [x] `web/e2e/smoke.spec.ts` runs in the Playwright image; 6/6 green.
- [x] No real external API in tests.
- [ ] Coverage — P13.

## 6. Security and quality gates

- [x] `harness-lint`, `sow-lint`, `check-doc-links`, `make check`, `npm lint/test/build`.
- [x] `npm run e2e` via `.tools/run-e2e.sh` — 6/6.
- [ ] Full gate run at P13.

## 7. Documentation, ADRs and indexes

- [x] `web/README.md` updated with the e2e command.
- [x] CR-0016 and this review plan indexed in `AGENTS.md`.
- [x] `.docs/sow/SOW.md` and the WP status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED` at P13.
- [ ] QA evidence recorded.
- [ ] Every blocking defect fixed.

## 9. Deferred items and residual risks

- [x] Live load/chaos — executed after `docker compose --profile core down -v`, rebuild, migrate and demo seed.
- [ ] Coverage thresholds/CI wiring — owner: QA, card: `WP-24` T06, due 2026-12-31.

## 10. Decisions

- [x] CR-0016 creates WP-24.
- [x] Playwright runs in the official image because the pinned Node image lacks the browser dependencies.
2026-11-09 | P12 | infra/load/board.js, .tools/{run-load,chaos-worker-kill}.sh, tools.rs | k6 + chaos + fuzz test; live run blocked by Compose migrate auth (k6 health 0%, migrate password failure) | operator/WP-03 | P13
2026-11-09 | P12 | infra/compose, infra/load, .tools | migrate/seed/reset OK; chaos recovered; k6 0% failed p95 2.6 ms; excess 429 | compose runner token/dev fakes fixed | P13
