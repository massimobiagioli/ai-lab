# AI-Lab Product UI

| Field | Value |
|---|---|
| Document | `docs/product-ui.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `docs/frontend.md`, `docs/office.md`, `web/src/app/` |

The product UI is a Trello-simple Kanban with a pixel office and precise operational surfaces. Every screen has loading, empty, error and degraded states; the UI is a projection of backend truth.

## 1. Surfaces

| Surface | Purpose | Status |
|---|---|---|
| Dashboard | Flow KPIs and health | Delivered (flow summary) |
| Office | Pixel office and inspector | WP-18 slice |
| Kanban | Six-column board | WP-19 follow-up |
| Inventory | Providers, models, agents, CV, projects | WP-19 follow-up |
| SOW / Plan | Idea, SOW, WBS, materialization | WP-19 follow-up |
| Memory | Scoped search and explorer | WP-19 follow-up |
| Settings | Lab, autonomy, budgets, exposure | WP-19 follow-up |

## 2. Dashboard

`Dashboard` takes a `FlowMetrics` signal input and renders lead p50/p95, cycle p50, WIP, throughput and blocked share. The `StatusBadge` shows `Flow health` as `success` when blocked share ≤ 10% and `warning` otherwise. It reuses the pixel tokens and status component.

## 3. UX rules

- Immediate feedback; no fake progress.
- No color-only status; labels carry meaning.
- Keyboard paths and focus-visible styles.
- Responsive down to phone, read-mostly.
- Empty/error/degraded states are never blank pages.

## 4. Tests

`npm --prefix web test` covers dashboard rendering and the unhealthy-flow badge. The full Playwright a11y/visual/E2E suites are owned by WP-21.

## 5. Deferred to the WP-19 follow-up

- Full board, inventory, SOW, memory and settings pages with stores.
- Optimistic updates and rollback.
- Playwright journeys J1–J4 and screenshots.

## 6. References

- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §11.7–§11.9
- `.docs/FRONTEND-INSTRUCTIONS.md`
- `web/src/app/dashboard/`
