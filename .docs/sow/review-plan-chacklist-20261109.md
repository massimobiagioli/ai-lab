# Review Plan Checklist — 2026-11-09

| Field | Value |
|---|---|
| Purpose | Strict, ordered execution plan to finish the full work-package review defined by `.prompts/review-all-prompt.md`. |
| Baseline | `main` at `f8cbc98`; WP-17 and WP-18 merged; WP-13B created; CI suspended by PR #4 (revert at P25). |
| Ordering | Execute P00 → P26 in order. One branch per point, merged into `main` before starting the next. |
| Evidence | A point is complete only with a green local gate run, the merge SHA and, for WP points, a QA `APPROVED` verdict. |
| Owner | Chief Software Engineer (agent) |

## 0. Merge protocol (applies to every point)

1. `git switch main && git pull --ff-only`.
2. `git switch -c review/<point-id>-<slug>`.
3. Implement the point; update its WP review plan, status, progress log and indexes.
4. Run the point gates and record the exact output in the review plan.
5. Commit with a Conventional Commit message and the trailers:
   `AI-Lab-Run: n/a`, `AI-Lab-Card: WP-XX`, `AI-Lab-Agent: opencode-dev-agent`.
6. `git push -u origin review/<point-id>-<slug>`; open a PR.
7. For WP points, spawn the independent QA subagent and require `APPROVED`; fix and re-review until approved.
8. Merge with `gh pr merge <n> --admin --merge` (CI is suspended; admin merge is authorised by PR #4 and only after the local gates are green).
9. `git switch main && git pull --ff-only`; confirm the merge SHA; append it to the WP progress log; delete the local/remote branch.
10. Never start the next point with a red gate or an open P0/P1.

**Local gates for every point**

```bash
scripts/harness-lint.sh
scripts/sow-lint.sh
make check
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
scripts/run-web.sh npm run lint
scripts/run-web.sh npm test
scripts/run-web.sh npm run build
scripts/run-web.sh npm run format:check
.tools/check-doc-links.sh
.tools/check-crate-dependencies.sh
.tools/check-architecture-fitness.sh
.tools/check-openapi-drift.py
```

Point-specific gates are listed on each point. No real external APIs in tests.

---

## P00 — Canonical checklist and governance cleanup

- **Branch:** `review/p00-review-checklist`
- **Scope:** add this file; index it in `AGENTS.md`; close the duplicate PR #6 and delete its branch; confirm no stray checklist file.
- **Acceptance:** file exists at `.docs/sow/review-plan-chacklist-20261109.md`; `harness-lint`, `sow-lint`, `check-doc-links` green; PR #6 closed; merge SHA recorded.
- **Merge:** docs-only, no QA subagent required.

## P01 — Reopen WP-19, review plan and data layer

- **Branch:** `review/p01-wp19-data`
- **Scope:** CR-0015 reopens WP-19; write `WP-19-PRODUCT-UI-review-plan.md`; add typed models and signal stores for board/cards, inventory (agents/projects/providers), SOW, memory and settings, using the edge API where available and an explicit demo read model where the edge is still a 501 stub (marked with `AILAB_DEMO_DATA` and a WP-13B T07 follow-up); wire the dashboard to the new stores.
- **Acceptance:** CR + review plan; typed stores with `idle|loading|ready|error`; no `any`; tests for each store; `make check` green.
- **Depends:** WP-17/WP-18 (done).

## P02 — WP-19 Kanban board

- **Branch:** `review/p02-wp19-kanban`
- **Scope:** `/projects/:projectId/board` with TODO/READY/DOING/REVIEW/DONE/ICEBOX columns, card detail, filters (`?card`, `?assignee`), drag-and-drop and keyboard move (`M`, arrows, Enter), optimistic move with rollback on 409/422.
- **Acceptance:** board renders from the store; move and rollback tests; keyboard path test; no fake API; review-plan items checked; gates green.
- **Depends:** P01.

## P03 — WP-19 inventory

- **Branch:** `review/p03-wp19-inventory`
- **Scope:** `/inventory/agents`, `/inventory/projects`, `/inventory/providers`: lists, detail drawers, status/health, empty/loading/error states.
- **Acceptance:** component tests per surface; routes lazy; accessibility checks; gates green.
- **Depends:** P01.

## P04 — WP-19 SOW and delivery graph

- **Branch:** `review/p04-wp19-sow`
- **Scope:** `/projects/:projectId/sow`: SOW sections, WBS/task graph, materialize action with confirmation; read-only when the API read model is unavailable.
- **Acceptance:** view + materialize tests; optimistic/disabled states; gates green.
- **Depends:** P01.

## P05 — WP-19 memory

- **Branch:** `review/p05-wp19-memory`
- **Scope:** `/memory`: scoped search, document/notes list, citation preview, empty/loading/error states.
- **Acceptance:** search store tests; keyboard/a11y; gates green.
- **Depends:** P01.

## P06 — WP-19 settings and onboarding

- **Branch:** `review/p06-wp19-settings`
- **Scope:** `/settings`: lab/autonomy/budget settings, provider/SCM status, first-run onboarding checklist on the dashboard; token exposure mode.
- **Acceptance:** settings forms with validation; onboarding tests; gates green.
- **Depends:** P01.

## P07 — WP-19 QA and closure

- **Branch:** `review/p07-wp19-qa`
- **Scope:** complete the WP-19 review plan, run the independent QA subagent, fix findings, record `APPROVED`, update WP/SOW status and deferrals (real-data assertions to WP-13B T07, visual suites to WP-21).
- **Acceptance:** QA `APPROVED`; all local gates green; WP-19 `done` with merge SHA.
- **Depends:** P02–P06.

## P08 — Reopen WP-20 and deliver metrics/logs

- **Branch:** `review/p08-wp20-metrics`
- **Scope:** CR-0016 reopens WP-20; review plan; structured logs, `/metrics` counters/histograms, log fields and redaction.
- **Acceptance:** metrics endpoint evidence; log redaction tests; review-plan items checked; gates green.
- **Depends:** P07.

## P09 — WP-20 traces, SLOs, dashboards and alerts

- **Branch:** `review/p09-wp20-tracing`
- **Scope:** OTel trace wiring (consumes WP-13B T03 when available; otherwise a documented local exporter), SLO definitions, dashboards and alert rules.
- **Acceptance:** trace/SLO/dashboard/alert artifacts; tests or config validation; gates green.
- **Depends:** P08.

## P10 — WP-20 backup/DR/runbooks and QA

- **Branch:** `review/p10-wp20-dr`
- **Scope:** backup/restore drill evidence, DR runbook, incident runbooks; independent QA; close WP-20.
- **Acceptance:** restore-drill output; runbooks indexed; QA `APPROVED`; WP-20 `done`.
- **Depends:** P09.

## P11 — WP-21 E2E and accessibility tooling

- **Branch:** `review/p11-wp21-e2e`
- **Scope:** reopen WP-21 (CR-0017); review plan; install/configure Playwright and axe; E2E journeys J1–J5; a11y checks on every route.
- **Acceptance:** Playwright suite runs; axe zero critical/serious; CI-ready config; gates green.
- **Depends:** P07 (routes) and P10.

## P12 — WP-21 load, chaos, security and fuzz

- **Branch:** `review/p12-wp21-hardening`
- **Scope:** k6 load scenarios, chaos (worker/runner kill), security and fuzz tests; no real external APIs.
- **Acceptance:** load/chaos runs recorded; security/fuzz suites; gates green.
- **Depends:** P11.

## P13 — WP-21 coverage and CI wiring

- **Branch:** `review/p13-wp21-coverage`
- **Scope:** coverage reports and thresholds (backend ≥80%, domain ≥90%, frontend ≥80%, new code ≥80%), wire all suites into CI; independent QA; close WP-21.
- **Acceptance:** coverage reports; threshold failures fail the gate; QA `APPROVED`; WP-21 `done`.
- **Depends:** P12.

## P14 — Reopen WP-22 and deliver docs/demo/acceptance

- **Branch:** `review/p14-wp22-docs`
- **Scope:** CR-0018 reopens WP-22; review plan; docs manifest, acceptance report, demo script, handover pack.
- **Acceptance:** docs indexed; demo script passes; gates green.
- **Depends:** P13.

## P15 — WP-22 SBOM, signing, release and QA

- **Branch:** `review/p15-wp22-release`
- **Scope:** syft SBOMs, cosign signing/verification, release 1.0 changelog, rollback procedure; independent QA; close WP-22.
- **Acceptance:** SBOM/signature evidence; QA `APPROVED`; WP-22 `done`.
- **Depends:** P14.

## P16 — Reopen WP-23 and deliver maintenance jobs

- **Branch:** `review/p16-wp23-maintenance`
- **Scope:** CR-0019 reopens WP-23; review plan; dependency updates/Renovate, CV reflection, memory hygiene, stale-card/health jobs; independent QA; close WP-23.
- **Acceptance:** job schedules/tests; QA `APPROVED`; WP-23 `done`.
- **Depends:** P15.

## P17 — WP-13B review plan and T01 resume

- **Branch:** `review/p17-wp13b-resume`
- **Scope:** write `WP-13B-FOLLOW-UP-review-plan.md`; deliver T01 open/branch resume with head-SHA handling.
- **Acceptance:** resume tests; review-plan items checked; gates green.
- **Depends:** P16 (WP-13B is last by decision).

## P18 — WP-13B T02 real context provider

- **Branch:** `review/p18-wp13b-context`
- **Scope:** repo map, memory top-8 with citations, files, conventions and prior feedback through a `ContextProvider` port; deterministic local provider plus the WP-08 integration.
- **Acceptance:** context tests; citations rendered; no placeholders; gates green.
- **Depends:** P17.

## P19 — WP-13B T03 OTel and T10 readiness

- **Branch:** `review/p19-wp13b-telemetry`
- **Scope:** OTel spans for LLM/SCM/sandbox and queue/lease/run metrics; `/health/ready` component checks (DB, migrations, Redis, runner) with `NOT_READY`.
- **Acceptance:** spans/metrics/readiness tests; gates green.
- **Depends:** P18.

## P20 — WP-13B T04 DoD and T05 real adapters

- **Branch:** `review/p20-wp13b-adapters`
- **Scope:** Kanban-service DONE transition with DoD; real DeepSeek/GitHub adapters behind the HTTP transport; fakes only under `AILAB_DEV_FAKES`.
- **Acceptance:** DoD/adapter tests; gates green.
- **Depends:** P19.

## P21 — WP-13B T06 Redis limiter and fan-out

- **Branch:** `review/p21-wp13b-redis`
- **Scope:** Redis token bucket (fail closed) and pub/sub fan-out; integration test gated by `AILAB_REDIS_URL`.
- **Acceptance:** limiter/fan-out tests; fail-closed evidence; gates green.
- **Depends:** P20.

## P22 — WP-13B T07 full REST surface and schemathesis

- **Branch:** `review/p22-wp13b-rest`
- **Scope:** implement the remaining Appendix B operations (replace the 501 stubs); run schemathesis with zero failures; re-run the WP-19 real-data assertions.
- **Acceptance:** schemathesis zero failures; contract drift gate; WP-19 integration re-checked; gates green.
- **Depends:** P21.

## P23 — WP-13B T08 WebSocket buffer and T09 generated OpenAPI

- **Branch:** `review/p23-wp13b-openapi`
- **Scope:** per-connection 256 buffer and ephemeral-first drop; generated OpenAPI (utoipa or an equivalent generator) with `openapi-diff` breaking-change checks and the full DTO schema set.
- **Acceptance:** WS backpressure tests; generated OpenAPI + breaking-change gate; gates green.
- **Depends:** P22.

## P24 — WP-13B T11 tests/docs and QA

- **Branch:** `review/p24-wp13b-qa`
- **Scope:** complete tests/docs, independent QA, close WP-13B.
- **Acceptance:** QA `APPROVED`; WP-13B `done`; all residual WP-13/WP-16 items closed or explicitly accepted.
- **Depends:** P23.

## P25 — Re-enable CI

- **Branch:** `review/p25-ci-reenable`
- **Scope:** restore the automatic triggers in `ci.yml`, `images.yml`, `nightly.yml`, `release.yml` (revert PR #4); push and confirm all required checks run on `main`.
- **Acceptance:** 16 required checks registered; a green run on `main`; no workflow parse errors.
- **Depends:** P24.

## P26 — Final review report and closure

- **Branch:** `review/p26-final-report`
- **Scope:** publish the final report: WP, review plan, defects fixed, tests, QA verdict, merge SHA; final summary table; delete review branches; confirm `SOW.md` all `done`.
- **Acceptance:** report indexed; `sow-lint`/`harness-lint` green; CI green; no P0/P1; no open blocker.
- **Depends:** P25.

---

## Current risks to keep visible during execution

| Risk | Mitigation |
|---|---|
| Edge REST endpoints for WP-19 are 501 until WP-13B T07 | Use the documented demo read model (`AILAB_DEMO_DATA`) in P01–P07 and re-run the real-data assertions in P22. |
| CI is suspended until P25 | Use `--admin` merges only after the full local gate run; record the gate output in the review plan. |
| Tooling absent (Playwright, k6, axe, schemathesis, trivy, syft) | Install/configure in P11, P12, P13, P15, P22; record the commands. |
| WP-19..WP-23 are currently `done` without review | Reopen each with a CR in its first point; do not claim done without QA. |
| WP-13/WP-16 closed by CR-0012 without QA | Their residuals are owned by WP-13B P17–P24; record the residual risk in P24. |
| Stacked/feature branches may need rebases | Rebase on `main` after each merge; re-run gates before requesting QA. |
