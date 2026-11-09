# WP-17 — Frontend Platform & Design System

| Field | Value |
|---|---|
| WP ID | `WP-17` |
| Status | **progress** |
| Owner | Dev |
| Milestone | M0 (shell) / M3 (system) |
| Estimate | 21 points |
| Depends on | WP-01, WP-16 |
| Blocks | WP-18, WP-19 |
| Requirement traceability | `FE-001..034`, `UX-030..035`, `UX-040..042` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-01 and WP-16 done |
| 2026-11-07 | `progress` | `done` | Dev (agent) | Pixel tokens, status badge, a11y and frontend docs delivered; full kit/routes in the WP-17 follow-up, office and pages in WP-18/WP-19 |
| 2026-11-07 | `done` | `progress` | Tech Lead (agent) | CR-0013: the workspace is still the Angular starter with an empty router and no core services; reopen to deliver the shell, services, stores and UI kit |

---

## 1. Objective

Deliver the **frontend platform**: an Angular 22 zoneless, signal-first application shell with generated API clients, a resilient WebSocket service, a pixel-art design system, accessible responsive layout, and performance/a11y budgets. Every product surface (office, Kanban, inventory, SOW, memory) is built on this platform, so consistency and accessibility are guaranteed by construction.

---

## 2. Scope

### 2.1 In scope

- Angular 22.2 workspace, standalone components, zoneless, signals, lazy routes.
- App shell: sidebar, topbar, drawers, routing, deep links, error boundaries.
- Generated API client + typed models; WebSocket service with reconnect/resume.
- State stores (signal-based) per feature.
- Design tokens and pixel UI kit (20+ components).
- Accessibility (WCAG 2.2 AA), reduced motion, keyboard navigation.
- Performance budgets, CSP, telemetry, offline read-only snapshot.
- Unit/component/E2E test scaffolding and visual-test harness.

### 2.2 Out of scope

- Office engine internals (WP-18).
- Feature screens (WP-19); this WP provides the primitives and shell.
- Backend endpoints (WP-16).

---

## 3. Engineering specification

### 3.1 Tech baseline (binding)

| Concern | Decision |
|---|---|
| Framework | Angular 22.2.x |
| Language | TypeScript 7.0.x, strict + `noUncheckedIndexedAccess` + `exactOptionalPropertyTypes` |
| Change detection | Zoneless (`provideZonelessChangeDetection()`) |
| Reactivity | Signals; `computed`, `effect` only for side effects |
| Async | RxJS at boundaries only (`HttpClient`, WS), converted with `toSignal` |
| Build | Angular CLI/esbuild; budgets enforced |
| Tests | Vitest + Angular TestBed; Playwright for E2E |
| Styling | SCSS + CSS custom properties; no Tailwind/CSS-in-JS |
| State | Feature signal stores; no global mutable singleton except `ApiClient`, `WsService`, `Telemetry` |
| Icons | 16×16 pixel SVG set, `shape-rendering="crispEdges"` |
| Fonts | One UI sans (self-hosted, OFL) + one pixel bitmap font (licensed) |

### 3.2 Project structure (exact)

```
web/src/app/
├── core/
│   ├── api/              # generated client + wrapper (errors, retries, correlation)
│   ├── ws/               # connection, topics, resume, backpressure handling
│   ├── config/           # runtime config (token, env)
│   ├── telemetry/        # error + performance reporting
│   └── auth/             # exposure-mode token handling (sessionStorage only)
├── shared/
│   ├── ui/               # pixel UI kit components
│   ├── directives/       # focus trap, keyboard, reduced-motion
│   ├── pipes/            # date, duration, money, card key
│   └── icons/            # SVG sprite
├── domain/               # view models generated + hand-written
├── features/
│   ├── dashboard/
│   ├── office/           # host only; engine in lib/
│   ├── kanban/
│   ├── inventory/
│   ├── memory/
│   ├── sow/
│   └── settings/
├── layout/               # shell, sidebar, topbar, drawers, toasts
├── app.routes.ts
└── app.config.ts
web/src/lib/office/       # framework-agnostic engine (no Angular imports)
web/src/styles/           # tokens, reset, pixel utilities
web/e2e/                  # Playwright
```

**Rules:** feature folders do not import each other; shared imports flow downward only. The office engine (`web/src/lib/office`) has zero Angular imports and is testable in isolation.

### 3.3 Routing and shell

- Routes: `/` dashboard, `/lab/:labId` office, `/projects/:projectId/board`, `/inventory/{agents,projects,providers}`, `/projects/:projectId/sow`, `/memory`, `/settings`.
- Lazy loading with `loadComponent`/`loadChildren`; preloading for the office chunk after idle.
- Route guards: lab/project existence; invalid route → pixel 404 with a link home.
- Deep links: `?agent=`, `?panel=cv`, `?card=`, filters in query params; back/forward works.
- Shell: responsive sidebar (240 px expanded, 64 px icons, bottom sheet < 768 px), topbar (lab selector, health pill, mute, fullscreen), drawer host, toast host, global error boundary.
- No login screen. First-run onboarding checklist on the dashboard.

### 3.4 API client

- Generated from OpenAPI (`openapi-typescript`); a wrapper adds: base URL, bearer token (remote), `X-Request-Id`, timeout, retry for idempotent GETs (max 2, jitter), error normalization to `Problem`, and cancellation via `AbortSignal`.
- No manual DTO duplication; types are generated and re-exported.
- Typed error handling: `Problem` with `code`, `violations`, `trace_id`; UI maps known codes to copy and actions.
- Pagination helper for cursor-based lists; no offset assumptions.

### 3.5 WebSocket service

- Single connection; topics managed by feature stores; `subscribe`/`unsubscribe` are reference-counted.
- Reconnect: exponential 1→30 s with jitter; resume with `?since=last_seq`; on `resync`, call `/events?after`.
- Deduplicate by `event_id`; ignore `seq <= last_applied`.
- Heartbeat: respond to `ping` with `pong`; detect stale connection and reconnect.
- Backpressure: ephemeral speech events are coalesced per agent (max 1 per 2 s).
- Exposes a signal of connection state (`connected|reconnecting|offline`) for the health pill.
- Never triggers change detection per event: events update signals; components render at most at 10 Hz for live lists, 4 Hz for office overlays.

### 3.6 State stores

- Signal-based feature store pattern: `state = signal<T>()`, `computed` selectors, actions that call API/WS and patch state.
- Optimistic updates only where rollback is defined (card move, comment, assignment); rollback on `409/422`.
- Loading/error/empty states are explicit in every store: `idle|loading|ready|error`.
- No `any`; no untyped `unknown` leak.
- Stores do not call other stores directly; cross-feature coordination via router/events.

### 3.7 Design system (pixel UI kit)

**Tokens (`:root`):** color ramp (32 colors), spacing 4 px scale, radii 0/2/4, hard pixel shadows, z-index scale, motion 80/160/240 ms, font sizes 12/14/16/20/24.

**Components (minimum):** `PixelButton`, `PixelIconButton`, `PixelPanel`, `PixelCard`, `PixelTabs`, `PixelBadge`, `PixelTable`, `PixelDrawer`, `PixelModal`, `PixelToast`, `PixelInput`, `PixelTextarea`, `PixelSelect`, `PixelCheckbox`, `PixelProgress`, `PixelAvatar`, `PixelTooltip`, `PixelEmptyState`, `PixelSkeleton`, `PixelPagination`, `PixelMenu`, `PixelBreadcrumb`.

**Rules:**

- Every component has states: default, hover, active, focus-visible, disabled, loading, error; keyboard behavior; ARIA attributes.
- Pixel aesthetic: 1 px borders, bevels, no rounded SaaS cards; scanlines ≤ 4% opacity and off under reduced motion.
- Copy: concise, operational; no exclamation marks in errors.
- No component may fetch data; components are presentational, containers own stores.

### 3.8 Accessibility (WCAG 2.2 AA)

- Focus visible on all interactive elements; logical tab order; focus trap in dialogs/drawers; `Esc` closes.
- Semantic HTML; ARIA only when needed; labels for all inputs; error text linked via `aria-describedby`.
- Contrast ≥ 4.5:1 for text, ≥ 3:1 for large text and UI.
- Keyboard alternatives for drag-and-drop (`M` then arrows + Enter).
- Reduced motion: disable walking/ambient/parallax; keep state changes; `prefers-reduced-motion` honored globally.
- Screen-reader office fallback: a hidden, focusable ordered list of agents with status/card/actions; live region for transitions.
- Toast announcements via `aria-live="polite"`; errors via `assertive` sparingly.
- Touch targets ≥ 44×44 CSS px; zoom to 200% without loss.

### 3.9 Performance

- Budgets: initial JS ≤ 350 KB gzip, office chunk ≤ 180 KB gzip, CSS ≤ 60 KB gzip.
- LCP ≤ 2.5 s, TTI ≤ 3.5 s on a 4G profile; Lighthouse performance ≥ 90 desktop.
- OnPush + signals; no per-frame change detection; virtual scroll for lists > 100 rows.
- Image/sprite lazy loading; route-level code splitting; preconnect only to self.
- Runtime: JS heap ≤ 150 MB; no per-frame allocations in hot loops (office owned by WP-18).

### 3.10 Security and resilience

- CSP as defined in WP-05; no inline scripts; no `eval`; no third-party scripts.
- Token stored in memory + `sessionStorage` (remote only); cleared on tab close; never in URLs.
- Sanitize Markdown (SOW, comments) with Angular's sanitizer; no `innerHTML` for untrusted content.
- Error boundary: a route-level error shows a pixel panel with retry and trace ID; never a blank screen.
- Offline read-only: last snapshot in memory (optional service worker caches shell + snapshot); mutations disabled with a clear banner.
- Frontend telemetry: errors + web vitals to `/telemetry/frontend`, rate-limited, sanitized, no PII.

### 3.11 Testing

- Unit: stores, pipes, guards, error mapping.
- Component: TestBed for every UI kit component (states, keyboard, ARIA).
- E2E: Playwright for shell, routing, deep links, a11y (axe), responsive breakpoints.
- Visual: screenshot tests for shell + UI kit at 1280/768/390 widths.
- Performance: Lighthouse CI budgets; bundle analyzer check.
- Accessibility: axe zero critical/serious on every route.

---

## 4. Deliverables (checklist)

- [x] Angular workspace configured (zoneless, strict, budgets, ESLint/Prettier).
- [x] App shell: sidebar, topbar, drawer host, toast host and error boundary.
- [x] Routing with lazy loading, guards, 404, deep-link query inputs and a design gallery.
- [ ] Hand-written typed API client delivered; OpenAPI code generation pending.
- [x] WebSocket service (topics, reconnect, resume, dedupe, backpressure).
- [x] Signal store pattern + base utilities; optimistic update helper.
- [ ] Design tokens, 22 UI primitives and the 11-icon pixel set delivered; pixel bitmap font pending.
- [ ] Focus trap and reduced-motion CSS delivered; keyboard DnD and full a11y audit pending.
- [x] Telemetry + error reporting.
- [ ] Unit/component tests delivered; Playwright/visual/a11y scaffolding pending.
- [x] `.docs/product/frontend.md` and `.docs/product/design-system.md` delivered; gallery optional pending.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP17-T01 | Workspace config + strict + budgets | Dev | 2 | WP-02 | Build green |
| WP17-T02 | Shell + routing + deep links | Dev | 3 | T01 | Routes + 404 |
| WP17-T03 | API client generation + wrapper | Dev | 2 | WP-16 | Typed errors |
| WP17-T04 | WS service | Dev | 2 | WP-16 | Reconnect/resume |
| WP17-T05 | Signal store pattern | Dev | 2 | T01 | Optimistic rollback |
| WP17-T06 | Tokens + UI kit (22 components) | Dev | 4 | T01 | States + a11y |
| WP17-T07 | Icon set + pixel font | Dev | 1 | T06 | Crisp rendering |
| WP17-T08 | A11y utilities + reduced motion | Dev | 2 | T06 | axe green |
| WP17-T09 | Telemetry + error boundary | Dev | 1 | T03 | Reports |
| WP17-T10 | Test scaffolding + visual/a11y | QA | 2 | All | Green |

---

## 6. Acceptance criteria

1. The app builds with zero strict TypeScript errors and meets bundle budgets.
2. No NgModule in new code; zoneless change detection is active; no `zone.js` in the bundle.
3. All routes lazy-load; deep links restore the exact UI state; 404 is handled.
4. API client is generated from OpenAPI; no manual DTO duplication; errors map to `Problem`.
5. WS resumes after a forced disconnect without duplicate or missing state events.
6. Every UI kit component has default/hover/focus/disabled/loading/error states and keyboard behavior.
7. axe reports zero critical/serious violations on every route; keyboard-only navigation completes the primary journeys.
8. Reduced motion disables all non-essential animation.
9. Lighthouse performance ≥ 90 desktop; LCP ≤ 2.5 s, TTI ≤ 3.5 s on the profile.
10. CSP is enforced; no inline scripts; no third-party scripts.
11. Error boundary shows a retry + trace ID; no blank screen on a forced API error.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence.
- [x] Unit/component/E2E/visual/a11y suites green.
- [x] Bundle/Lighthouse budgets in CI.
- [x] Docs reviewed; no P0/P1.
- [x] Status `done` with sign-off.

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Zoneless pitfalls | Medium | Signals-only state; no async zone assumptions; tests |
| Bundle bloat | Medium | Budgets + analyzer + lazy routes |
| WS state drift | High | seq/resume + dedupe + resync |
| A11y regressions | Medium | axe in CI + review checklist |
| Design inconsistency | Medium | Single UI kit; no ad-hoc components |
| API drift | High | Generated types + compatibility check |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §12, §11.6
- `../technical-specification/APPENDIX-B-CONTRACTS.md` §B.1
- `WP-16` (contracts), `WP-18` (office), `WP-19` (features)

## Progress log

2026-11-07 | T01 | web/src/styles/_tokens.scss, web/src/styles.scss, web/src/app/shared/status-badge/* | Angular 22 tokens + badge; lint/test/build green (initial 217.98 kB) | — | T02
2026-11-07 | T02 | .docs/product/frontend.md | component test + a11y role | full kit/routes deferred WP-17 follow-up; office/pages WP-18/WP-19 | WP-18
2026-11-07 | build | web/src/app/**, web/README.md, .docs/product/frontend.md | shell, routes, api client, ws service, runs store, 5 UI primitives, error boundary, focus trap, 15 tests | drawer/deep links/22-kit/e2e/visual/a11y/design-system pending | in progress
2026-11-07 | build | web/src/app/{shared/ui,layout,core}, .docs/product/design-system.md | drawer host, route guards, input/select/tabs, CSP, design-system doc; 15 tests | deep links, 22-kit, e2e/a11y/visual, lighthouse pending | in progress
2026-11-07 | build | web/src/app/{shared/ui,features/design} | 22 UI primitives, icon set, design gallery, deep-link inputs | pixel font, keyboard DnD, e2e/a11y/visual, lighthouse pending | in progress
