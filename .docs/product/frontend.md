# AI-Lab Frontend

| Field | Value |
|---|---|
| Document | `.docs/product/frontend.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/architecture.md`, `.docs/FRONTEND-INSTRUCTIONS.md`, `web/` |

The frontend is Angular 22 zoneless with signals, strict TypeScript 6.0 (ADR-0024), standalone components and the pixel design system.

## 1. Design tokens

`web/src/styles/_tokens.scss` defines the pixel palette, spacing, borders, typography and status colours as CSS custom properties. Tokens are the only source of visual values; components do not hardcode colours or spacing.

## 2. Components

- `StatusBadge` (`web/src/app/shared/status-badge/`): signal inputs `label` and `tone`, `role="status"`, uppercase pixel styling, no color-only meaning.
- UI primitives (`web/src/app/shared/ui/`): `Button`, `Card`, `EmptyState`, `Spinner`, `PageHeader`.
- `FocusTrap` (`web/src/app/shared/directives/focus-trap.ts`): traps Tab focus and restores it on destroy.

## 3. Shell and routing

`Shell` (`web/src/app/layout/shell/`) composes the sidebar, topbar, error boundary, toast host and
router outlet, with a skip link and a `main` landmark. Routes are lazy-loaded and the wildcard route
renders the 404 page. The dashboard and runs page are store-backed; the office/board/SOW/inventory/
memory/settings routes are placeholders for WP-18/WP-19.

## 4. Core services

- `ApiClient`: typed `fetch` wrapper with base URL, bearer token, `X-Request-Id`, timeout, GET
  retries (max 2, jitter), cancellation and `Problem` normalization (`core/api/`).
- `WsService`: single socket, reference-counted topics, `?since=` resume, `event_id`/`seq` dedupe,
  heartbeat, exponential reconnect and speech coalescing; exposes a connection signal.
- `Telemetry`: fire-and-forget frontend telemetry.
- `ErrorState` + `AppErrorHandler`: global boundary that records telemetry and shows a retry with
  the trace id.

## 5. State

Feature stores are signal-based with explicit `idle|loading|ready|error` states. `RunsStore` is the
reference implementation (load, cancel, counters).

## 6. Testing

Vitest + Angular TestBed. `npm test` covers the API client, the runs store, the shell, the status
badge and the office pathfinding/state engines.

## 3. Accessibility

- WCAG 2.2 AA contrast.
- `role="status"` for live status; keyboard paths; focus-visible styles.
- Reduced-motion support for animated surfaces (WP-18).
- No color-only status: the badge text carries the state.

## 4. Budgets and quality

- Initial bundle budget 350 kB error, 300 kB warning; office chunk 180 kB; component styles 60 kB.
- `npm run lint`, `npm test`, `npm run build` are required.
- The component test covers rendering, the status role and the tone class.

## 5. Deferred to consumer WPs

- Full design system kit and route shell: WP-17 follow-up.
- Canvas office host: WP-18.
- Dashboard, inventory, Kanban, SOW, memory and settings pages: WP-19.
- Playwright a11y/visual suites: WP-21.

## 6. References

- `.docs/FRONTEND-INSTRUCTIONS.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §11–§12
- `web/src/styles/_tokens.scss`, `web/src/app/shared/status-badge/`
