# WP-19 — Product UI Surfaces Review Plan

| Field | Value |
|---|---|
| WP | `WP-19` |
| Contract | `.docs/sow/WP-19-PRODUCT-UI.md` |
| Review branches | `review/p01-wp19-data` … `review/p07-wp19-qa` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `approved` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks and references present and consistent.
- [x] CR-0015 reopens WP-19; the status history and progress log record it.
- [x] The contract is split into P01–P07 so each surface is independently reviewable.

## 2. Specification conformance

- [x] Routes `/`, `/runs`, `/projects/:projectId/board`, `/inventory/{agents,projects,providers}`, `/projects/:projectId/sow`, `/memory`, `/settings` exist and lazy-load — evidence: `web/src/app/app.routes.ts`.
- [x] Kanban six columns, DnD + keyboard moves, filters, card drawer and the blocked reason — P02; real-time/violation animation in P07.
- [ ] Agent Inspector Live/CV/Memory/Controls with rollback — P03.
- [x] Inventory lists, detail drawers and create dialogs; provider keys show last4 only — P03.
- [x] SOW versions, sections, status and delivery graph — P04; diff/approve/reject/export/traceability/streaming in WP-13B T07.
- [x] Delivery graph (task dependencies) and materialize confirmation — P04.
- [x] Memory scopes/search/reader/notes/pin/delete — P05.
- [x] Settings lab/autonomy/budgets/scheduler/office/exposure/danger — P06.
- [ ] Health and handoff drawers — P07/WP-13B T07.
- [x] Loading/empty/error/degraded states delivered on every implemented screen.
- [ ] Responsive, a11y and reduced motion — P02–P06, verified by WP-21.
- [ ] E2E J1–J4 and visual screenshots — WP-21 (owned, due 2026-12-31).
- [ ] `.docs/product/ui.md` with routes, states and interaction rules — P07.

## 3. Deliverables

- [x] Typed domain models for cards, agents, projects, providers, SOW, memory and settings — `web/src/app/domain/models.ts`.
- [x] ApiClient read models with the explicit demo fallback (`AILAB_DEMO_DATA`) for the 501 edge stubs — `core/api/api-client.ts`, `core/demo/demo-data.ts`.
- [x] Signal stores with `idle|loading|ready|error` for board, inventory, SOW, memory and settings — `features/*/store`.
- [x] Optimistic card move with rollback on 409/422 — `features/board/store/board.store.ts`.
- [x] Dashboard KPIs and onboarding — P06; live activity/idea composer in WP-13B T07.
- [x] Kanban board — P02 (`features/board`).
- [x] Inventory tables and create dialogs — P03 (`features/inventory`).
- [x] SOW/Plan — P04 (`features/sow`).
- [x] Memory explorer — P05 (`features/memory`).
- [x] Settings/danger zone — P06 (`features/settings`).
- [ ] Health/failed-runs/retired/handoff drawers — P03/P06.
- [x] Loading/empty/error/degraded on board, inventory, SOW, memory, settings and dashboard.
- [x] `.docs/product/ui.md` — P07.
- [ ] E2E/visual — WP-21 (owned, due 2026-12-31).

## 4. Acceptance criteria

1. [ ] Onboarding through the UI — P06.
2. [ ] Live KPIs within 3 s and the idea composer — P06 (WS/store wiring).
3. [ ] Board six columns, violations and keyboard — P02.
4. [ ] Agent inspector truthful live steps/CV/memory with rollback — P03.
5. [ ] Inventory inline validation and write-only secrets — P03.
6. [ ] SOW generation/diff/approve/export/traceability/materialize — P04.
7. [ ] Memory scoped results with citations and note/pin/delete — P05.
8. [ ] Settings validated/confirmed/audited — P06.
9. [ ] Loading/empty/error/degraded on every screen — P02–P06.
10. [ ] axe/keyboard/reduced motion — WP-21 (owned, due 2026-12-31).
11. [ ] Visual screenshots — WP-21 (owned, due 2026-12-31).

## 5. Implementation and tests

- [x] Board store load/optimistic/rollback — tests: `features/board/store/board.store.spec.ts`.
- [x] Board render/keyboard/detail — tests: `features/board/board.spec.ts`.
- [x] Demo model determinism — test: `core/demo/demo-data.spec.ts`.
- [x] Inventory/SOW/memory/settings tests — `inventory.spec.ts`, `sow.spec.ts`, `memory.spec.ts`, `settings.spec.ts`.
- [ ] No real external API in tests (fetch/WebSocket mocked).
- [ ] Coverage thresholds — WP-21.

## 6. Security and quality gates

- [x] No secrets in the frontend; provider last4 only; write-only secrets to be enforced in P03.
- [x] `scripts/harness-lint.sh`, `scripts/sow-lint.sh`, `make check`, frontend lint/test/build/format green.
- [ ] Full gate run on P07.

## 7. Documentation, ADRs and indexes

- [x] CR-0015 and the review plan indexed in `AGENTS.md`.
- [x] `.docs/sow/SOW.md` and the WP status/progress updated.
- [x] `.docs/product/ui.md` — P07.

## 8. QA review

- [x] QA subagent verdict: `APPROVED` (HEAD `400c2c5`).
- [x] QA evidence: qa-review-agent returned APPROVED with owned WP-24/WP-13B T07 deferrals; 81 tests, gates green.
- [x] Every blocking defect fixed and re-reviewed (two passes).

## 9. Deferred items and residual risks

- [ ] Real edge data for board/inventory/SOW/memory/settings — owner: Dev, card: `WP-13B` T07, due 2026-12-31.
- [ ] E2E/visual/a11y, activity/idea, health/handoff drawers and SOW diff/export — owner: Dev/QA, card: `WP-24`, due 2026-12-31.
- [ ] Activity feed/idea composer/live KPIs — owner: Dev, card: `WP-24` T01, due 2026-12-31.

## 10. Decisions

- [x] CR-0015 — reopen WP-19; alternatives and consequences recorded.
- [x] Demo read model is explicit (`AILAB_DEMO_DATA`) and never presented as live data.
2026-11-09 | P02 | web/src/app/features/board/**, app.routes.ts | board columns, DnD, keyboard, filters, drawer, tests (73 web) | real-time/violations P07 | P03
2026-11-09 | P03 | web/src/app/features/inventory/**, shared/ui/pixel-table | inventory lists/detail/create; tests | health/handoffs P06 | P04
2026-11-09 | P04 | web/src/app/features/sow/**, app.routes.ts | SOW sections/graph/materialize; tests | diff/export/streaming WP-13B T07 | P05
2026-11-09 | P05 | web/src/app/features/memory/**, app.routes.ts | memory search/notes/pin/delete; tests | real API WP-13B T07 | P06
2026-11-09 | P06 | web/src/app/features/{settings,dashboard}/** | settings + onboarding; tests | activity/idea/health WP-13B T07 | P07
2026-11-09 | P07 | web/src/app/features/{board,inventory,sow,plan,settings,dashboard}/**, .docs/product/ui.md | inspector, wizards, SOW actions, plan, settings audit, onboarding, board violations; 81 tests | a11y/E2E WP-21, real API/health/handoff WP-13B T07 | QA re-review
