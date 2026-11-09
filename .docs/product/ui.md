# AI-Lab Product UI

| Field | Value |
|---|---|
| Document | `.docs/product/ui.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/frontend.md`, `.docs/product/design-system.md`, `web/src/app/features/` |

The product UI is built on the WP-17 platform (Angular 22, signals, pixel design system). Every screen exposes loading, empty, error and ready states; mutations are optimistic with rollback and destructive actions are confirmed.

## 1. Routes

| Route | Surface | States |
|---|---|---|
| `/` | Dashboard: run KPIs, onboarding, next actions | loading/error/ready |
| `/runs` | Run list, cancel, detail steps | loading/error/empty/ready |
| `/projects/:projectId/board` | Kanban board (6 columns, DnD, keyboard) | loading/error/empty/ready |
| `/projects/:projectId/sow` | SOW sections, versions, delivery graph, materialize | loading/error/empty/ready |
| `/projects/:projectId/plan` | Plan/DAG tree, dependencies, dry-run materialize | loading/error/empty/ready |
| `/inventory/agents` | Agents list, detail and Agent Inspector | loading/error/empty/ready |
| `/inventory/projects` | Projects list, detail and create wizard | loading/error/empty/ready |
| `/inventory/providers` | Providers list, write-only key wizard | loading/error/empty/ready |
| `/memory` | Scoped search, notes, citations, pin/delete | loading/error/empty/ready |
| `/settings` | Lab, autonomy, budget, scheduler, office, danger zone | loading/error/ready |
| `/lab/:labId` | Virtual office host | ready |
| `/design` | Design-system gallery | ready |

## 2. Interaction rules

- **Keyboard:** board cards support `M`, ArrowLeft/Right, Enter and Escape; tabs/menus use roving tabindex; dialogs/drawers close on Escape; the skip link and landmarks are always present.
- **Optimism:** card moves and agent actions apply immediately and roll back on 409/422; the violation message is shown.
- **Destructive actions:** archive/delete require a typed confirmation; the result is recorded in the demo audit log until the real API lands.
- **Secrets:** provider keys are write-only; only the last four characters are displayed.
- **Filters:** board filters are kept in the component state; deep links use `?agent`, `?panel`, `?card`.
- **Reduced motion:** all non-essential animation is disabled by the global rule.
- **Demo data:** while the edge endpoints are 501 stubs, a `Demo data` badge is shown and the source is `AILAB_DEMO_DATA`.

## 3. State model

Feature stores expose `idle | loading | ready | error`, a `Problem` error and a `demo` flag. Screens render a spinner, an error card with retry, an empty state or the data; the dashboard shows a loading/error branch instead of zeroes.

## 4. Accessibility

Each route provides landmarks, headings, visible focus, `aria-live` status and no color-only meaning. Automated axe/keyboard/visual suites are owned by WP-21.
