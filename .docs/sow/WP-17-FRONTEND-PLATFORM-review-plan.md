# WP-17 — Frontend Platform & Design System Review Plan

| Field | Value |
|---|---|
| WP | `WP-17` |
| Contract | `.docs/sow/WP-17-FRONTEND-PLATFORM.md` |
| Review branch | `feat/wp-17-frontend-platform` (stacked on the WP-13B re-baseline; opened as PR #3) |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_review` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks and references are present and consistent.
- [x] Every cited requirement and specification section exists and matches the WP (FE-001..034, UX-030..035, UX-040..042).
- [x] CR-0013 reopens WP-17; status history and progress log record the reopen.

## 2. Specification conformance

- [x] Zoneless Angular 22, standalone components, signals, strict TypeScript — evidence: `web/src/app/app.config.ts`, `web/tsconfig.json`, `npm run build`.
- [x] All routes lazy-load via dynamic `import()`; 404 handled — evidence: `web/src/app/app.routes.ts`, build chunk output.
- [x] Route guards for lab/project slugs — evidence: `web/src/app/core/guards/param.guard.ts`.
- [x] Deep-link query inputs (`agent`, `panel`, `card`) — evidence: `features/placeholder/placeholder.ts`.
- [x] Typed API client with `Problem`, retries, correlation, timeout and cancel — evidence: `core/api/api-client.ts`, `api-client.spec.ts`.
- [x] WebSocket service with topics, `?since` resume, resync backfill (`/events?after&limit`), dedupe, heartbeat and reconnect — evidence: `core/ws/ws.service.ts`, `ws.service.spec.ts`.
- [x] Signal stores with explicit `idle|loading|ready|error`, base utilities and an optimistic helper with rollback — evidence: `core/state/store-utils.ts`, `runs.store.ts`, `store-utils.spec.ts`.
- [x] Pixel tokens and the 27 primitives, including the mandated PixelPanel/PixelTable/PixelIconButton/PixelTextarea/PixelCheckbox — evidence: `shared/ui/`, `/design` gallery.
- [x] Pixel bitmap font (in-repo 3×5 glyph renderer) — evidence: `shared/ui/pixel-text/`, `pixel-text.spec.ts`.
- [x] Accessibility: skip link, landmarks, focus-visible, focus trap, Escape-close dialogs/drawers, keyboard reorder, reduced motion, `aria-live` toast, error boundary — evidence: `layout/shell/`, `shared/directives/`, `dialog.spec.ts`.
- [x] Telemetry and global error handler — evidence: `core/telemetry/`, `core/errors/`.
- [x] CSP meta and no inline scripts — evidence: `web/src/index.html`.
- [ ] Playwright, axe, visual snapshots and Lighthouse evidence — deferred to WP-21 (owned, see §9).
- [ ] Offline read-only snapshot — deferred to WP-20/WP-21 (owned, see §9).

## 3. Deliverables

- [x] Angular workspace (zoneless, strict, ESLint/Prettier).
- [x] App shell: sidebar, topbar, drawer host, toast host, error boundary.
- [x] Routing with lazy loading, guards, 404 and deep-link inputs.
- [ ] Generated API client: the typed client is hand-written; OpenAPI generation is deferred to WP-13B T09.
- [x] WebSocket service with resync backfill.
- [x] Signal store pattern with base utilities and the optimistic helper.
- [x] Tokens, 27 primitives (including the mandated set), 11-icon set and the in-repo pixel bitmap font.
- [x] Accessibility utilities (focus trap, keyboard DnD, reduced motion) plus the shell a11y baseline.
- [x] Telemetry and error reporting.
- [ ] Playwright/visual/a11y scaffolding — deferred to WP-21.
- [x] `.docs/product/frontend.md` and `.docs/product/design-system.md`; the optional gallery is delivered at `/design`.

## 4. Acceptance criteria

1. [x] Builds with zero strict TypeScript errors; budgets pass — evidence: `npm run build`.
2. [x] No NgModule in new code; zoneless active; no `zone.js` — evidence: `provideZonelessChangeDetection`, `package.json`.
3. [x] All routes lazy-load and 404 is handled; deep-link inputs exist. Full deep-link UI restoration follows WP-19.
4. [ ] API client generated from OpenAPI — deferred to WP-13B T09; the hand-written client maps errors to `Problem`.
5. [x] WS resumes with `?since`, backfills on `resync`, dedupes by `event_id`/`seq` and reconnects — evidence: `ws.service.ts`, `ws.service.spec.ts`.
6. [x] Primitives expose disabled/loading/error where applicable, dialogs/drawers close on Escape, tabs/menu implement WAI-ARIA keyboard navigation and error text is linked via `aria-describedby` — evidence: `ui-kit.spec.ts`, `dialog.spec.ts`, component templates.
7. [ ] axe zero critical/serious — deferred to WP-21.
8. [x] Reduced motion disables non-essential animation — evidence: `styles.scss`, spinner/skeleton CSS.
9. [ ] Lighthouse ≥ 90 — deferred to WP-21.
10. [x] CSP enforced; no inline scripts — evidence: `index.html`.
11. [x] Error boundary shows retry + trace id — evidence: `layout/shell/shell.ts`, `core/errors/`.

## 5. Implementation and tests

- [x] Shell/sidebar/topbar/drawer — tests: `shell.spec.ts`.
- [x] API client — tests: `api-client.spec.ts`.
- [x] Runs store — tests: `runs.store.spec.ts`.
- [x] Pixel text and keyboard DnD — tests: `pixel-text.spec.ts`, `keyboard-dnd.spec.ts`.
- [x] No real external API in tests (fetch/WebSocket are stubbed).
- [x] Every primitive has a TestBed creation test and the interactive ones cover states/keyboard — test: `ui-kit.spec.ts`.
- [x] WS resync, optimistic rollback, Escape-close and pixel font — tests: `ws.service.spec.ts`, `store-utils.spec.ts`, `dialog.spec.ts`, `pixel-text.spec.ts`.
- [ ] Coverage thresholds — reported with the WP-21 coverage run.

## 6. Security and quality gates

- [x] No secrets in the frontend; token is session-scoped; CSP set.
- [x] `scripts/harness-lint.sh` green.
- [x] `scripts/sow-lint.sh` green.
- [x] `make check` green (on the branch).
- [x] `npm run lint`, `npm test`, `npm run build`, `npm run format:check`.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/frontend.md` updated.
- [x] `.docs/product/design-system.md` created and indexed.
- [x] `AGENTS.md` documentation index updated for the new doc (and this review plan).
- [x] `.docs/sow/SOW.md` and the WP status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED`.
- [ ] QA evidence: recorded below after the review.
- [ ] Every blocking defect fixed and re-reviewed.

## 9. Deferred items and residual risks

- [ ] Playwright, axe, visual snapshots and Lighthouse evidence — owner: QA, card: `WP-21`, due: 2026-12-31.
- [ ] Offline read-only snapshot — owner: Dev, card: `WP-20`, due: 2026-12-31.
- [ ] Generated API client from OpenAPI — owner: Dev, card: `WP-13B` T09, due: 2026-12-31.
- [ ] Full deep-link UI restoration — owner: Dev, card: `WP-19`, due: 2026-12-31.

## 10. Decisions

- [x] CR-0013 — reopen WP-17; alternatives and consequences recorded.
