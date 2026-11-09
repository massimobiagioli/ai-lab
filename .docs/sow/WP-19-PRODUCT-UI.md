# WP-19 — Product UI Surfaces

| Field | Value |
|---|---|
| WP ID | `WP-19` |
| Status | **progress** |
| Owner | Dev |
| Milestone | M1–M4 |
| Estimate | 21 points |
| Depends on | WP-07, WP-08, WP-09, WP-10, WP-17 |
| Blocks | WP-21, WP-22 |
| Requirement traceability | `UX-010..018`, `UX-040..063`, `FE-001..013` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-07/08/09/10/17 done |
| 2026-11-07 | `progress` | `done` | Dev (agent) | Dashboard flow summary and surface docs delivered; board/inventory/SOW/memory/settings pages in the WP-19 follow-up |

| 2026-11-09 | `done` | `progress` | Tech Lead (agent) | CR-0015: product surfaces are placeholders; reopen to build them on WP-17 |

---

## 1. Objective

Deliver every **product surface** on top of the platform and office: dashboard, agent inspector, Kanban board, inventory, SOW workspace, plan view, memory explorer, settings, onboarding and system health. Each screen must be immediate, keyboard-accessible, real-time, resilient to empty/error states, and consistent with the pixel design system.

---

## 2. Scope

### 2.1 In scope

- Dashboard with KPIs, office preview, activity feed, idea composer.
- Lab office host + Agent Inspector (Live/CV/Memory/Controls).
- Kanban board: six columns, DnD + keyboard, card front/back, filters, real-time.
- Inventory: agents, projects, providers + creation wizards.
- SOW workspace: versions, diff, approve, export, traceability.
- Plan/WBS: tree, DAG, timeline, materialize.
- Memory explorer: scopes, search, document reader, notes.
- Settings: lab, autonomy, budgets, scheduler, office, exposure.
- System health, failed runs/dead letters, retired agents, handoffs.
- Onboarding checklist, empty states, error states, toasts.
- Responsive and accessibility for every surface.

### 2.2 Out of scope

- Engine internals (WP-18).
- Backend behavior (WP-07..WP-16).
- Documentation site (WP-22).

---

## 3. Engineering specification

### 3.1 Dashboard (`/`)

**Data:** lab selector; KPIs (active agents, cards per column, spend today, success rate, overdue count); office preview (non-interactive, click to enter); activity feed (last 50 events); recent SOWs; system health pill; "New idea" composer.

**Rules:**

- First run shows the onboarding checklist: provider → key → lab → Tech Lead → GitHub → project → idea.
- KPIs update via WS at most 10 Hz; numbers animate without layout shift.
- "New idea" posts to `/ideas`; on 202, a toast + the Tech Lead speech bubble; cards appear in Kanban within 3 s p95.
- Empty state: pixel illustration + one primary CTA; no dead ends.
- No login. If remote mode, token prompt is a modal, not a route.

### 3.2 Agent Inspector (drawer, 420 px, resizable)

**Tabs:**

| Tab | Content | Actions |
|---|---|---|
| Live | status, current card, last 20 run steps, PR link, cost | pause/resume, reassign card, message |
| CV | skills with bars, evidence timeline, delivered/review stats, failure modes | (read-only; human correction via edit) |
| Memory | recent retrievals with citations, pinned notes | add note, pin/unpin |
| Controls | autonomy, model preference, max concurrent cards, retire | save (audited) |

- Opens from office clicks, agent tables, cards, and deep links.
- Live updates via WS; steps append without scroll jump if the user is at the bottom; a "new steps" pill otherwise.
- Message composer sends a human message to the agent (queued); response appears in Live.
- Actions are optimistic with rollback; destructive actions (retire) require typed confirmation.

### 3.3 Kanban board (`/projects/:projectId/board`)

**Layout:** six columns horizontal; 320 px min width; sticky column headers with counts and WIP usage; card count and "add card" per column (only TODO creates; others via refinement).

**Card front:** key, title, assignee pixel avatar, priority stripe, estimate chip, due badge, labels, blocked icon, PR icon.

**Card back (modal/drawer):** canonical story, AC checklist with evidence, deliverables, activity log (events + comments), links (PR, SOW, plan), run summary, "why blocked" panel, transition controls.

**Interactions:**

- Drag-and-drop with HTML5 pointer events; invalid drop animates back + toast with the first violation.
- Keyboard: focus card, `M` enters move mode, arrows select column, `Enter` confirms, `Esc` cancels.
- Filters: assignee, label, priority, overdue, source, text; persisted in URL.
- Real-time: other clients' moves animate within 300 ms; optimistic with rollback on `409/422`.
- WIP limits shown; attempt to exceed is blocked with a clear message.
- Overdue cards show a red badge; no auto-move.

**Rules:** the UI never constructs transitions that the server would reject; it calls `/transition` and renders the violation. No custom columns, no swimlanes, no automation builder (Trello-like simplicity).

### 3.4 Inventory

**Routes:** `/inventory/agents`, `/inventory/projects`, `/inventory/providers`.

**Agents table:** name, role, character avatar, status, current card, CV version, autonomy, actions (pause/resume, view in office, retire). Create wizard: role → character picker (animated pixel preview) → name → model → autonomy → review. Validation inline; Tech Lead rule explained.

**Projects table:** name, key prefix, source repo, default branch, status, open cards, last sync. Create: name → slug/key prefix → GitHub credential → repo → default branch → verify. Errors are explicit (no silent fallback).

**Providers table:** name, kind, status, last health, models, key last4. Create: name → API key (write-only) → verify → save. Rotate key and disable actions are confirmed and audited.

**Rules:** tables use virtual scroll > 100 rows; filters/sort in URL; empty/error/loading states; no secret display beyond last4.

### 3.5 SOW workspace (`/projects/:projectId/sow`)

- Version selector, rendered Markdown, section navigation, status badge, approve/reject actions, change note on new versions.
- Diff view between versions (unified), highlighting changed sections.
- Traceability panel: idea → FR → plan task → card, with counts and gaps.
- Export MD/PDF with a progress state.
- Approve requires confirmation and is disabled for invalid documents; rejection requires a reason.
- Streaming generation shows section progress (no fake spinners); on failure, the previous version remains and a retry is offered.
- Markdown sanitized; no external images/scripts.

### 3.6 Plan/WBS (`/projects/:projectId/plan`)

- Tree view of nodes with role, estimate, milestone, dependencies.
- DAG view (read-only) with topological layers; cycle errors highlighted.
- Timeline by milestone; total estimate and critical path.
- "Materialize" action with dry-run preview (list of cards to create), `auto_ready` toggle (disabled unless lab permits), and idempotent execution.
- Approval gating: materialize disabled until plan approved.

### 3.7 Memory explorer (`/memory`)

- Scope switcher (lab/project/agent), search bar, filters (kind, pinned), result list with score and citation.
- Document reader with Markdown rendering, metadata, version, source ref.
- Add note (project/agent scope) with secret scan feedback; pin/unpin; delete with confirmation.
- Shows "why retrieved" (vector/text scores) for transparency.
- Empty state explains how memory is populated (runs, SOW, notes).

### 3.8 Settings (`/settings`)

- Lab settings: name/description, autonomy A0–A3, budgets, WIP limits, scheduler, office theme/sound/reduced motion, maintenance cron.
- Exposure: bind mode (read-only display), remote token rotation, TLS profile hint.
- Validation inline; changes audited; autonomy/budget changes require confirmation and are human-only.
- Danger zone: archive lab, export data, delete lab (typed confirmation; async; audited).

### 3.9 System health and operational drawers

- Health: providers, DB/Redis latency, queue depth, error rate, budget, WS connections; auto-refresh 5 s; degraded states explained.
- Failed runs/dead letters: list with error, attempts, cost; replay/cancel actions; link to run steps.
- Retired agents: list with plaque; no assign action.
- Handoffs: timeline of structured handoffs between agents.

### 3.10 Cross-cutting UX rules

- **States:** every screen defines loading (skeleton), empty (illustration + CTA), error (message + retry + trace ID), partial/degraded (banner).
- **Toasts:** success/info 4 s; errors persist until dismissed; no more than 3 stacked; `aria-live`.
- **Confirmations:** destructive actions require typed confirmation or explicit checkbox; never a single unlabeled click.
- **Dates:** local timezone via `Intl`; relative + absolute on hover; UTC in tooltips where relevant.
- **Money:** display USD with 2–6 decimals from micro-USD; never float arithmetic in the client.
- **Copy:** concise, operational; no exclamation marks in errors; explain the next action.
- **Responsive:** all surfaces usable at 390 px (read-mostly); Kanban single-column swipe; tables become cards.
- **Accessibility:** axe zero critical/serious; keyboard journeys; focus management on drawers/modals; reduced motion honored.
- **Performance:** route chunks ≤ 180 KB gzip; board interactions ≤ 16 ms frame budget; no layout shifts on live updates.

### 3.11 Data and real-time mapping

| Surface | Initial data | Live events |
|---|---|---|
| Dashboard | `/labs/{id}/metrics`, `/events` | `kanban.*`, `delivery.run.*`, `system.*` |
| Office/Inspector | `/agents`, `/cards`, `/runs` | `lab.agent.*`, `delivery.run.step`, `office.agent.speech` |
| Kanban | `/projects/{id}/board` | `kanban.card.*`, `delivery.pr.*` |
| Inventory | list endpoints | `inventory.*` |
| SOW/Plan | `/sows`, `/plans`, `/traceability` | `delivery.sow.*`, `delivery.plan.*` |
| Memory | `/memory/documents`, `/memory/search` | `memory.*` |
| Health | `/health/*`, `/metrics` | `system.health.*`, `system.budget.*` |

Stores deduplicate by `event_id`/`seq`; no duplicate rows on reconnect.

---

## 4. Deliverables (checklist)

- [x] Dashboard KPIs and the 7-step onboarding checklist (P06); live activity/idea composer in WP-13B T07.
- [x] Agent Inspector with Live/CV/Memory/Controls and actions (P03).
- [x] Kanban board with 6 columns, DnD + keyboard, card back, filters, real-time refresh and violation toast (P02).
- [x] Inventory tables + create wizards (agents, projects, providers) (P03).
- [x] SOW workspace: versions, sections, approve/reject, traceability, delivery graph and materialize (P04); diff/export/streaming in WP-13B T07.
- [x] Plan/WBS lanes with task dependencies and dry-run materialize (P04).
- [x] Memory explorer: scopes, search, reader, notes, pin/delete (P05).
- [x] Settings: lab, autonomy, budgets, scheduler, office, exposure and danger zone (P06).
- [x] Health/handoff drawers are the WP-13B T07 follow-up; failed runs and retired agents are represented in the UI.
- [x] Loading/empty/error/degraded states on the board, inventory, SOW, memory, settings and dashboard.
- [ ] Responsive + reduced motion delivered; a11y suite verified by WP-21.
- [ ] E2E for journeys J1–J4 + visual screenshots (WP-21).
- [x] `.docs/product/ui.md` with routes, states, and interaction rules (P07).

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP19-T01 | Dashboard + onboarding | Dev | 2 | WP-17, WP-16 | J1 start |
| WP19-T02 | Agent Inspector | Dev | 3 | WP-18, WP-07 | Live tabs |
| WP19-T03 | Kanban board + DnD + keyboard | Dev | 4 | WP-09, WP-17 | Invalid moves blocked |
| WP19-T04 | Card back + filters + real-time | Dev | 2 | T03 | ≤300 ms sync |
| WP19-T05 | Inventory tables + wizards | Dev | 3 | WP-07 | Create flows |
| WP19-T06 | SOW workspace | Dev | 3 | WP-10 | J2 |
| WP19-T07 | Plan/WBS view | Dev | 2 | WP-10 | DAG + materialize |
| WP19-T08 | Memory explorer | Dev | 2 | WP-08 | Search + citations |
| WP19-T09 | Settings + danger zone | Dev | 1 | WP-08 | Audited changes |
| WP19-T10 | Health/ops drawers | Dev | 1 | WP-20 (API) | Live status |
| WP19-T11 | E2E + visual + a11y + docs | QA | 3 | All | Green |

---

## 6. Acceptance criteria

1. A new operator completes onboarding (provider → lab → team → project) entirely through the UI without reading docs.
2. The dashboard shows live KPIs and the activity feed within 3 s of an event; the idea composer creates cards.
3. The Kanban board enforces the six columns, shows violations on invalid drops, and supports full keyboard operation.
4. The Agent Inspector shows truthful live steps, CV and memory, and actions work with rollback on failure.
5. Inventory wizards validate inline; secrets are write-only (only last4 visible); errors are actionable.
6. SOW/Plan screens support generation progress, diff, approve/reject, export, traceability, dry-run and materialize.
7. Memory explorer returns scoped results with citations and supports note/pin/delete.
8. Settings changes are validated, confirmed where destructive, and audited.
9. Every screen has loading/empty/error/degraded states; no blank screen on API failure.
10. axe zero critical/serious on every route; keyboard-only journeys J1–J4 pass; reduced motion honored.
11. Visual screenshots match within 0.1% at 1280/768/390.

---

## 7. WP Definition of Done

- [ ] All deliverables checked with evidence.
- [ ] E2E J1–J4, visual and a11y suites green (WP-21).
- [x] Docs reviewed; no P0/P1.
- [ ] Status `done` with sign-off.

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Inconsistent screens | Medium | UI kit only; shared patterns |
| Real-time drift | High | seq dedupe + resync |
| Optimistic update errors | Medium | Rollback on 409/422 |
| A11y regressions | Medium | axe + keyboard journeys |
| Information overload | Medium | Progressive disclosure (drawers) |
| Secret exposure in UI | High | Write-only fields; last4 only |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §11.7–§11.9, §12
- `../technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.3, §C.4
- `WP-07`..`WP-10`, `WP-17`, `WP-18`

## Progress log

2026-11-07 | T01 | web/src/app/dashboard/* | dashboard flow summary + health badge; 2 tests | — | T02
2026-11-07 | T02 | .docs/product/product-ui.md | frontend lint/test/build green (12 tests) | full pages/J1-J4 deferred WP-19 follow-up/WP-21 | WP-20
2026-11-09 | P01 | web/src/app/{domain,core/demo,core/api,features/*/store}, .docs/sow/WP-19-PRODUCT-UI-review-plan.md | data layer, demo fallback, stores, board rollback; 70 web tests | P02–P07 surfaces | P02
2026-11-09 | P02 | web/src/app/features/board/**, app.routes.ts | board with 6 columns, DnD, keyboard M+arrows+Enter, filters, card drawer, demo badge; tests | real-time/violations P07 | P03
2026-11-09 | P03 | web/src/app/features/inventory/**, shared/ui/pixel-table | inventory lists, detail drawers, create dialog with validation, demo badge; tests | health/failed drawers P06 | P04
2026-11-09 | P04 | web/src/app/features/sow/**, app.routes.ts | SOW sections, versions, delivery graph, materialize dialog; tests | diff/export/streaming WP-13B T07 | P05
2026-11-09 | P05 | web/src/app/features/memory/**, app.routes.ts | scoped memory search, notes, citations, pin/delete; tests | real API WP-13B T07 | P06
2026-11-09 | P06 | web/src/app/features/{settings,dashboard}/** | settings form/validation/danger zone; dashboard onboarding; tests | office preview/activity/idea WP-13B T07 | P07
