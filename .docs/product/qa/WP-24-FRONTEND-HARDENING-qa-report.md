# WP-24 — Frontend and QA hardening QA report

Date: 2026-11-09. Reviewer: opencode-qa. Author: opencode-dev (separation of duties).

## Verdict

**NOT APPROVED.** The delivered hardening is sound, but WP-24 cannot close: T01, T03, T06 and T07 are open, and the coverage thresholds are not met.

## Review

| Item | Result |
|---|---|
| Scope vs WP | T02/T04/T05 delivered; T01/T03 wait for the WP-13B T07 real-data endpoints; T06 misses visual/Lighthouse; T07 (load/soak, injected integrity, synthetic alert, dashboards) is partial. |
| Acceptance criteria | Coverage: backend 77.56 % (<80), domain 88.99 % (<90), web 76.06 % (<80). E2E 6/6, axe green, load 0 % failed (p95 2.6 ms), chaos worker recovery green. |
| Security | No secrets added; the Compose runner/API tokens are development placeholders; `gitleaks`/`cargo deny` green. |
| Tests | `cargo nextest`, `npm test`, Playwright 6/6, k6 and worker-kill chaos executed; evidence in the review plan. |
| Docs | `ui.md`, the review plan and the checklist are current; T01/T03/T06/T07 gaps are recorded. |
| Gates | Harness, SOW lint, doc links, `make check`, Playwright green; coverage red. |

## Blockers

- T01/T03: depend on WP-13B T07.
- T06: visual and Lighthouse runs not executed.
- T07: synthetic alert, budget alert and seven-dashboard evidence incomplete.
- Coverage: add tests in `orchestrator/postgres.rs` (1.66 %), `docker_backend.rs` (25.62 %), `domain/value.rs` (73.04 %) and the under-tested web views.

## Next

Re-review after WP-13B T07 and the coverage work; only then mark WP-24 `done`.
