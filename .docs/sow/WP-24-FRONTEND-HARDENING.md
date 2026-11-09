# WP-24 — Frontend & QA Hardening Follow-up

| Field | Value |
|---|---|
| WP ID | `WP-24` |
| Status | **ready** |
| Owner | Dev |
| Milestone | M8+ |
| Estimate | 21 points |
| Depends on | WP-19, WP-21, WP-18 |
| Blocks | — |
| Requirement traceability | `UX-010..018`, `UX-040..063`, `FE-001..013`, `UX-035` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-09 | — | `ready` | Tech Lead (agent) | CR-0016: schedule the WP-19/WP-21 residual UI, activity, drawer and test-suite work |

---

## 1. Objective

Close the product and verification residuals that WP-19 and WP-21 could not finish: the live dashboard activity/idea surfaces, the health/handoff/retired-agent drawers, the SOW diff/export/streaming actions, the full board filters/card back, the WBS DAG/timeline/critical path, and the Playwright/axe/visual/Lighthouse suites. Real edge data for these surfaces arrives with WP-13B T07.

---

## 2. Scope

### 2.1 In scope

- Dashboard: live activity feed, idea composer, the UX-040 KPI set and the office preview.
- Health, handoff, failed-run and retired-agent drawers.
- SOW diff/export/streaming and the plan WBS DAG/timeline/critical path.
- Board URL-persisted filters and the full card back (story, AC, deliverables, activity, links, run summary).
- Playwright E2E J1–J4, axe a11y, visual snapshots and Lighthouse budgets.
- Read-only real-data wiring once WP-13B T07 lands.

### 2.2 Out of scope

- The backend REST surface and schemas (WP-13B T07).
- The core platform, office engine and product surfaces delivered in WP-17–WP-19.

---

## 3. Engineering specification

### 3.1 Dashboard

The dashboard consumes the runs, lab and event stores: live activity from the WS hub, an idea composer that creates a TODO card through the edge API, the UX-040 KPI set (active agents, cards by column, spend, success rate, overdue) and the office preview.

### 3.2 Drawers

Health (provider/SCM/runner), handoffs and retired agents are read-only drawers with links to the owning surfaces. Failed runs filter `/runs?state=failed`.

### 3.3 SOW and plan

SOW diff compares versions; export downloads Markdown/JSON; streaming shows generation progress. The plan view renders the WBS DAG, timeline and critical path with the materialize dry-run.

### 3.4 Board

Filters (text, label, priority, assignee, source, overdue) are persisted in the URL; the card back shows the story, acceptance criteria, deliverables, activity, links and run summary.

### 3.5 Verification

Playwright covers J1–J4, axe covers every route, visual snapshots cover 1280/768/390 and Lighthouse enforces the budgets. Fixtures are deterministic; no real external APIs.

---

## 4. Deliverables (checklist)

- [ ] Dashboard activity feed, idea composer, UX-040 KPIs and office preview.
- [ ] Health, handoff, failed-run and retired-agent drawers.
- [ ] SOW diff/export/streaming.
- [ ] Plan WBS DAG, timeline and critical path.
- [ ] Board URL filters and full card back.
- [ ] Playwright E2E J1–J4.
- [ ] axe a11y suite on every route.
- [ ] Visual snapshots at 1280/768/390 and Lighthouse budgets.
- [ ] Real-data integration re-check after WP-13B T07.
- [ ] Load/soak, integrity injected test, synthetic alert test, budget alert, dashboards, cost, post-incident template and runbook tabletop.
- [ ] Tests and docs for every item.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP24-T01 | Dashboard activity/idea/KPIs/preview | Dev | 4 | WP-13B T07 | Live ≤3 s |
| WP24-T02 | Health/handoff/failed/retired drawers | Dev | 3 | WP-19 | Drawers verified |
| WP24-T03 | SOW diff/export/streaming | Dev | 3 | WP-13B T07 | Actions work |
| WP24-T04 | Plan DAG/timeline/critical path | Dev | 3 | WP-19 | Graph verified |
| WP24-T05 | Board URL filters and card back | Dev | 3 | WP-19 | Filters persisted |
| WP24-T06 | Playwright/axe/visual/Lighthouse | QA | 5 | WP-21 | Suites green |
| WP24-T07 | Load/soak, integrity injected test, synthetic alert test, budget alert, seven dashboards, cost dashboard, post-incident template, runbook tabletop | QA | 5 | WP-20, WP-13B T03 | Evidence green |

---

## 6. Acceptance criteria

1. The dashboard shows live activity and KPIs within 3 s of an event and the idea composer creates a card.
2. Health, handoff, failed-run and retired-agent drawers open with truthful data and links.
3. SOW diff, export and generation progress work.
4. The plan renders the WBS DAG, timeline and critical path, and the dry-run materialize previews the cards.
5. Board filters persist in the URL and the card back shows the full story, AC, deliverables, activity, links and run summary.
6. Playwright J1–J4 pass; axe reports zero critical/serious; visual snapshots match; Lighthouse budgets pass.
7. Real-data integration is re-verified after WP-13B T07.

---

## 7. WP Definition of Done

- [ ] All deliverables checked with evidence.
- [ ] WP24-T01..T06 complete; no P0/P1.
- [ ] Quality gates green.
- [ ] Docs updated; deferrals closed.
- [ ] Independent QA `APPROVED`.

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Edge endpoints still 501 | Real-data gaps | Land after WP-13B T07 and re-run the assertions. |
| Playwright/Lighthouse tooling absent | Suites cannot run | Install/configure in the first task and record the evidence. |
| Visual noise | Flaky screenshots | Deterministic fixtures and fixed viewports. |

---

## 9. References

- `.docs/sow/WP-19-PRODUCT-UI.md`, `.docs/sow/WP-21-TESTING.md`
- `.docs/product/ui.md`, `.docs/product/design-system.md`
- ADR-0008, ADR-0024
- `.docs/product/change-requests/CR-0016.md`
