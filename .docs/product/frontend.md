# AI-Lab Frontend

| Field | Value |
|---|---|
| Document | `.docs/product/frontend.md` |
| Version | 1.1.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/architecture.md`, `.docs/product/design-system.md`, `.docs/FRONTEND-INSTRUCTIONS.md`, `web/` |

The frontend is Angular 22 zoneless with signals, strict TypeScript 6.0.x (ADR-0024), standalone components and the pixel design system.

## 1. Design tokens

`web/src/styles/_tokens.scss` defines the pixel palette, spacing, borders, typography, shadows, motion and z-index as CSS custom properties. Tokens are the only source of visual values; components do not hardcode colours or spacing.

## 2. Components

- `StatusBadge` (`web/src/app/shared/status-badge/`): signal inputs `label` and `tone`, `role="status"`, no color-only meaning.
- 27 UI primitives (`web/src/app/shared/ui/`): Button, Card, EmptyState, Spinner, PageHeader, Drawer, Dialog, Toast, Breadcrumbs, Pagination, Skeleton, Progress, Avatar, Icon, Menu, Divider, Kbd, Tooltip, Input, Select, Tabs, PixelPanel, PixelTable, PixelIconButton, PixelTextarea, PixelCheckbox and PixelText.
- `FocusTrap` and `KeyboardDnd` (`web/src/app/shared/directives/`).

The design-system inventory, states and accessibility rules live in `.docs/product/design-system.md`; the `/design` route renders a live gallery.

## 3. Shell and routing

`Shell` (`web/src/app/layout/shell/`) composes the sidebar, topbar, drawer host, error boundary, toast host and router outlet, with a skip link and a `main` landmark. Every route is lazy-loaded; the wildcard renders the 404 page. Placeholder routes carry `agent`/`panel`/`card` query inputs for WP-18/WP-19.

## 4. Core services

- `ApiClient`: typed `fetch` wrapper with base URL, bearer token, `X-Request-Id`, timeout, GET retries (max 2, jitter), cancellation, `Problem` normalization and event backfill.
- `WsService`: single socket, reference-counted topics, `?since=` resume, `/events?after` resync backfill, `event_id`/`seq` dedupe, heartbeat, exponential reconnect and speech coalescing.
- `Telemetry` and `ErrorState`/`AppErrorHandler`: global boundary with trace id and retry.
- `withOptimistic` (`core/state/store-utils.ts`): optimistic update with rollback.

## 5. State

Feature stores are signal-based with explicit `idle|loading|ready|error` states. `RunsStore` is the reference implementation (load, optimistic cancel with rollback, counters).

## 6. Testing

Vitest + Angular TestBed: 52 tests covering the API client, WS resync, runs store, optimistic helper, shell, UI-kit primitives, status badge, pixel font, keyboard DnD and the office engine.

## 7. Accessibility

- WCAG 2.2 AA contrast; visible focus rings.
- Skip link, `main` landmark, `aria-live` toast host and `role="alert"` errors with trace id.
- Focus trap, Escape-close dialogs/drawers, roving-tabindex tabs/menus and `aria-describedby` form errors.
- Reduced-motion rules disable non-essential animation.

## 8. Budgets and quality

- Initial bundle budget 350 kB error / 300 kB warning; office chunk 180 kB; component styles 60 kB.
- `npm run lint`, `npm test`, `npm run build`, `npm run format:check` are required.
- CSP is set in `web/src/index.html`; no inline or third-party scripts.

## 9. Deferred to consumer work packages

- Office sprite atlas, ambient life, FPS telemetry and visual suites: WP-18.
- Dashboard, inventory, Kanban, SOW, memory and settings pages: WP-19.
- Playwright/axe/visual/Lighthouse suites: WP-21; offline snapshot: WP-20.
- OpenAPI code generation: WP-13B T09.

## 10. References

- `.docs/FRONTEND-INSTRUCTIONS.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §11–§12
- `.docs/product/design-system.md`
