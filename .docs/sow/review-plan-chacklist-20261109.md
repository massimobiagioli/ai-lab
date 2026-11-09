# Review Plan Checklist — 2026-11-09

| Field | Value |
|---|---|
| Purpose | Ordered execution checklist to finish the full work-package review defined by `.prompts/review-all-prompt.md`. |
| Baseline | `main` at `1ca3da8`; WP-17 and WP-18 merged; WP-13B created; CI suspended (PR #4, revert at P25). |
| Rule | Execute P00 → P26 in order. One branch per point, merged into `main` before the next. A box is checked only after its gates pass and the branch is merged (or, for P00, after the docs merge). |
| Owner | Chief Software Engineer (agent) |

## 0. Merge protocol

1. `git switch main && git pull --ff-only`.
2. `git switch -c review/<point-id>-<slug>`.
3. Implement; update the WP review plan, status, progress and indexes.
4. Run the point gates and record the output in the review plan.
5. Commit with `AI-Lab-Run`/`AI-Lab-Card`/`AI-Lab-Agent` trailers.
6. Push, open a PR; run the QA subagent for WP points; fix until `APPROVED`.
7. Merge with `gh pr merge <n> --admin --merge` (CI is suspended; only after green local gates).
8. Pull `main`, record the merge SHA, delete the branch, tick the box in the next point's branch.

**Gates for every point**

```bash
scripts/harness-lint.sh
scripts/sow-lint.sh
make check
scripts/run-web.sh npm run lint
scripts/run-web.sh npm test
scripts/run-web.sh npm run build
scripts/run-web.sh npm run format:check
.tools/check-doc-links.sh
.tools/check-crate-dependencies.sh
.tools/check-architecture-fitness.sh
.tools/check-openapi-drift.py
```

---

## 1. Ordered checklist

- [x] **P00 — Canonical checklist and governance cleanup.**
  Branch `review/p00-review-checklist`; add this file, index it in `AGENTS.md`, close the duplicate PR #6. Merged `1ca3da8`.

- [x] **P01 — Reopen WP-19, review plan and data layer.**
  Branch `review/p01-wp19-data`; CR-0015, `WP-19-PRODUCT-UI-review-plan.md`, typed models, ApiClient board/inventory/SOW/memory/settings with the explicit demo fallback, signal stores, tests. Merged `58dd820`.

- [x] **P02 — WP-19 Kanban board.**
  Branch `review/p02-wp19-kanban`; `/projects/:projectId/board`, six columns, card detail, filters, DnD + keyboard move (`M`, arrows, Enter), optimistic rollback, tests. Depends P01.

- [x] **P03 — WP-19 inventory.**
  Branch `review/p03-wp19-inventory`; `/inventory/{agents,projects,providers}` lists, detail drawers, status/health, empty/loading/error, tests. Depends P01.

- [x] **P04 — WP-19 SOW and delivery graph.**
  Branch `review/p04-wp19-sow`; `/projects/:projectId/sow` sections, task graph, materialize confirmation, tests. Depends P01.

- [x] **P05 — WP-19 memory.**
  Branch `review/p05-wp19-memory`; `/memory` scoped search, notes, citations, empty/loading/error, tests. Depends P01.

- [x] **P06 — WP-19 settings and onboarding.**
  Branch `review/p06-wp19-settings`; `/settings` lab/autonomy/budget/office, onboarding checklist on the dashboard, tests. Depends P01.

- [x] **P07 — WP-19 QA and closure.**
  Branch `review/p07-wp19-qa`; complete the review plan, QA `APPROVED`, fix findings, WP-19 `done`, deferrals to WP-13B T07 (real data) and WP-21 (visual/a11y). Depends P02–P06.

- [x] **P08 — Reopen WP-20 and deliver metrics/logs.**
  Branch `review/p08-wp20-metrics`; CR-0016, review plan, structured logs, `/metrics`, redaction tests. Depends P07.

- [x] **P09 — WP-20 traces, SLOs, dashboards, alerts.**
  Branch `review/p09-wp20-tracing`; OTel trace wiring (consumes WP-13B T03), SLOs, dashboards, alert rules. Depends P08.

- [x] **P10 — WP-20 backup/DR/runbooks and QA.**
  Branch `review/p10-wp20-dr`; restore drill, DR and incident runbooks, QA `APPROVED`, WP-20 `done`. Depends P09.

- [x] **P11 — WP-24 E2E and accessibility tooling.**
  Branch `review/p11-wp24-e2e`; Playwright + axe, journeys J1–J5, a11y per route; WP-21 is `done`, so these suites are owned by WP-24. Depends P07, P10.

- [ ] **P12 — WP-24 load, chaos, security, fuzz.** (scripts + fuzz delivered; live load/chaos blocked by Compose migrate auth — owner WP-03/operator)
  Branch `review/p12-wp24-hardening`; k6 load, chaos (worker/runner kill), security and fuzz suites; WP-24 evidence. Depends P11.

- [ ] **P13 — WP-24 coverage and CI wiring.**
  Branch `review/p13-wp24-coverage`; coverage thresholds (backend 80/domain 90/frontend 80/new 80), CI wiring, QA `APPROVED`, WP-24 `done`. Depends P12.

- [ ] **P14 — Reopen WP-22 and deliver docs/demo/acceptance.**
  Branch `review/p14-wp22-docs`; CR-0018, review plan, docs manifest, acceptance report, demo script, handover. Depends P13.

- [ ] **P15 — WP-22 SBOM, signing, release and QA.**
  Branch `review/p15-wp22-release`; syft SBOM, cosign signing/verification, release 1.0 changelog, rollback, QA `APPROVED`, WP-22 `done`. Depends P14.

- [ ] **P16 — Reopen WP-23 and deliver maintenance jobs.**
  Branch `review/p16-wp23-maintenance`; CR-0019, review plan, Renovate/CV reflection/memory hygiene/stale-card/health jobs, QA `APPROVED`, WP-23 `done`. Depends P15.

- [ ] **P17 — WP-13B review plan and T01 resume.**
  Branch `review/p17-wp13b-resume`; `WP-13B-FOLLOW-UP-review-plan.md`, open/branch resume with head-SHA handling, tests. Depends P16.

- [ ] **P18 — WP-13B T02 real context provider.**
  Branch `review/p18-wp13b-context`; repo map, memory top-8 with citations, files, conventions, feedback via a `ContextProvider` port, tests. Depends P17.

- [ ] **P19 — WP-13B T03 OTel and T10 readiness.**
  Branch `review/p19-wp13b-telemetry`; LLM/SCM/sandbox spans, queue/lease/run metrics, `/health/ready` component checks. Depends P18.

- [ ] **P20 — WP-13B T04 DoD and T05 real adapters.**
  Branch `review/p20-wp13b-adapters`; Kanban-service DONE transition with DoD, real DeepSeek/GitHub adapters, fakes dev-gated. Depends P19.

- [ ] **P21 — WP-13B T06 Redis limiter and fan-out.**
  Branch `review/p21-wp13b-redis`; Redis token bucket (fail closed) and pub/sub fan-out, integration test. Depends P20.

- [ ] **P22 — WP-13B T07 full REST surface and schemathesis.**
  Branch `review/p22-wp13b-rest`; implement the remaining Appendix B operations, schemathesis zero failures, re-run WP-19 real-data assertions. Depends P21.

- [ ] **P23 — WP-13B T08 WS buffer and T09 generated OpenAPI.**
  Branch `review/p23-wp13b-openapi`; per-connection 256 buffer and ephemeral-first drop, generated OpenAPI + `openapi-diff` + full DTO schema set. Depends P22.

- [ ] **P24 — WP-13B T11 tests/docs and QA.**
  Branch `review/p24-wp13b-qa`; complete tests/docs, QA `APPROVED`, WP-13B `done`. Depends P23.

- [ ] **P25 — Re-enable CI.**
  Branch `review/p25-ci-reenable`; restore the automatic triggers (revert PR #4), confirm 16 required checks run on `main`. Depends P24.

- [ ] **P26 — Final review report and closure.**
  Branch `review/p26-final-report`; publish the final report (WP, review plan, defects, tests, QA, merge SHA), confirm all WPs `done`, no P0/P1, CI green. Depends P25.

---

## 2. Risks to keep visible

| Risk | Mitigation |
|---|---|
| WP-19 edge endpoints are 501 until WP-13B T07 | Use the explicit demo read model and re-run real-data assertions in P22. |
| CI suspended until P25 | Merge with `--admin` only after the full local gate run; record the output. |
| Missing tooling (Playwright, k6, axe, schemathesis, trivy, syft) | Install in P11/P12/P13/P15/P22 and record the commands. |
| WP-19..WP-23 were `done` without review | Reopen with CRs in P01/P08/P11/P14/P16; no done without QA. |
| WP-13/WP-16 closed by CR-0012 without QA | WP-13B P17–P24 owns the residual; record it in P24. |
| Stacked/feature branches need rebases | Rebase on `main` after each merge and re-run gates. |

## 3. Progress log

| Date | Point | Branch | Merge SHA | Notes |
|---|---|---|---|---|
| 2026-11-09 | P00 | `review/p00-review-checklist` | `1ca3da8` | Checklist created and indexed. |
| 2026-11-09 | P01 | `review/p01-wp19-data` | `58dd820` | Data layer, demo fallback, stores, board rollback. |
| 2026-11-09 | P02 | `review/p02-wp19-kanban` | `0dcfa90` | Six-column board, DnD, keyboard, filters, drawer, tests. |
| 2026-11-09 | P03 | `review/p03-wp19-inventory` | `6f2e24e` | Inventory lists, detail drawers, create dialog, last4 keys. |
| 2026-11-09 | P04 | `review/p04-wp19-sow` | `9cd6624` | SOW sections, delivery graph, materialize. |
| 2026-11-09 | P05 | `review/p05-wp19-memory` | `bd6d940` | Memory search, notes, citations, pin/delete. |
| 2026-11-09 | P06 | `review/p06-wp19-settings` | `dc27438` | Settings, onboarding, danger zone. |
| 2026-11-09 | P07 | `review/p07-wp19-qa` | `ad4c65c` | QA APPROVED; WP-19 done. |
| 2026-11-09 | P08 | `review/p08-wp20-metrics` | `b0b90e1` | Metrics, redacted logs. |
| 2026-11-09 | P09 | `review/p09-wp20-tracing` | `d6af2b3` | Trace propagation, SLOs, dashboard, alerts. |
| 2026-11-09 | P10 | `review/p10-wp20-dr` | `5eac3c2` | QA APPROVED; WP-20 done. |
| 2026-11-09 | P11 | `review/p11-wp24-e2e` | `da9cb89` | Playwright/axe, 6/6. |
| 2026-11-09 | P12 | `review/p12-wp24-hardening` | blocked | Scripts + fuzz delivered; live load/chaos blocked by Compose migrate auth. |
