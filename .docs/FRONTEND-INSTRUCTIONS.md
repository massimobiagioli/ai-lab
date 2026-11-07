# AI-Lab Frontend Instructions

| Field | Value |
|---|---|
| Document | `.docs/FRONTEND-INSTRUCTIONS.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Applies to | Every Angular application, component, store, service, office engine module and Playwright test |
| Source of truth | `.docs/technical-specification/` and `.docs/sow/` |

These rules are binding for all frontend work. They refine the technical specification and the work packages; they never override them. If a rule conflicts with the specification, the specification wins and this document is corrected through an ADR.

---

## 1. Language and toolchain

1. All code, comments, commit messages, UI copy, tests, logs and documentation are in English.
2. Angular is pinned to `22.2.x`, TypeScript to `7.0.x` and Node.js to `24.x` LTS through `package-lock.json`, `.tool-versions` and the container image.
3. TypeScript runs with `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `noImplicitOverride` and `noFallthroughCasesInSwitch`.
4. Use the Angular CLI and esbuild. Do not add a second bundler.
5. Format with Prettier and lint with ESLint. Warnings are errors in CI.
6. Use npm 11 with `package-lock.json`. Do not use yarn or pnpm in this repository.
7. No third-party runtime script, CSS framework or charting library without an ADR and a security review.

---

## 2. Architecture and structure

```text
web/src/app/
  core/       API client, WebSocket, config, auth token handling, telemetry
  shared/     pixel UI kit, directives, pipes, icons
  domain/     generated types and view models
  features/   dashboard, office host, kanban, inventory, memory, sow, settings
  layout/     shell, sidebar, topbar, drawers, toasts
web/src/lib/office/   framework-agnostic office engine
web/src/styles/       tokens, reset, pixel utilities
web/e2e/              Playwright
```

1. Feature folders must not import each other. Shared imports flow downward only.
2. The office engine under `web/src/lib/office/` MUST NOT import Angular. Angular hosts it and exchanges a snapshot and interaction events.
3. Cross-feature coordination goes through the router, the WebSocket service or a shared store owned by `core` or `shared`. Do not create circular imports.
4. One route per user surface. Use lazy loading with `loadComponent` or `loadChildren`.
5. No NgModules for new code. Use standalone components and functional providers.
6. Keep the shell thin. Screen logic lives in the feature folder, not in `AppComponent`.

---

## 3. Angular rules

1. Use standalone components, directives and pipes. Do not introduce NgModules.
2. Use zoneless change detection with `provideZonelessChangeDetection()`. Do not import `zone.js` or rely on zone-based assumptions.
3. Use signals for state and `computed` for derived values. Use `effect` only for side effects, never to derive state.
4. Use RxJS only at boundaries: `HttpClient`, WebSocket and browser events. Convert to signals with `toSignal` or an explicit store.
5. Use the current control flow (`@if`, `@for`, `@switch`). Do not add structural directives for new code.
6. Use `inject()` in field initializers or constructor injection. Prefer `providedIn: 'root'` only for true singletons.
7. Use `ChangeDetectionStrategy.OnPush` where change detection is used at all. Do not call `markForCheck` or `detectChanges` from the office loop.
8. Use typed reactive forms for forms with validation. Template-driven forms are allowed only for a single-field search input.
9. Use the router with typed routes, guards and resolvers where they prevent flicker. Deep links must restore the exact UI state.
10. Clean up subscriptions, timers, observers and animation frames in `ngOnDestroy` or with `takeUntilDestroyed`.
11. Do not use `any` outside validated parsing boundaries. Narrow `unknown` immediately with a schema or type guard.
12. Do not use `innerHTML` with untrusted content. Use Angular sanitization and a safe Markdown renderer.

---

## 4. State management

1. Each feature owns a signal store: `state = signal<T>()`, selectors with `computed`, and actions that call the API or WebSocket.
2. Model explicit states: `idle`, `loading`, `ready`, `error`. Every screen renders all four.
3. Server state is never copied into multiple stores. One canonical store per resource; other components read from it.
4. Optimistic updates are allowed only where rollback is defined. Roll back on `409` and `422` and show the server violation.
5. Do not use global mutable singletons except `ApiClient`, `WsService`, `Telemetry` and the router.
6. Do not store derived data. Compute it with `computed`.
7. Do not mirror the entire board or card object in local component state. Read from the store and pass inputs.
8. Cancel in-flight requests when the route or component is destroyed.
9. Persist only user preferences (theme, sound, filters) and the remote token in `sessionStorage`. Never persist domain state.

---

## 5. API client and WebSocket

1. Generate types from OpenAPI. Do not hand-write a duplicate DTO.
2. Wrap the generated client with a single `ApiClient` that adds the base URL, bearer token in remote mode, `X-Request-Id`, timeout, cancellation and error normalization to `Problem`.
3. Retry only idempotent GET requests, at most twice, with jitter. Never retry a mutation automatically.
4. Map `Problem.code` to user copy and actions. Show `trace_id` in error panels.
5. Use cursor pagination. Never assume offset or total count.
6. Use one WebSocket connection per client. Reference-count subscriptions per topic.
7. On reconnect, resume from `last_seq`; on `resync`, call the event backfill endpoint and then resubscribe.
8. Deduplicate events by `event_id` and ignore `seq` less than or equal to the last applied sequence.
9. Never trigger change detection per WebSocket event. Update signals and render live lists at most 10 times per second; office overlays at most 4 times per second.
10. Coalesce ephemeral speech events per agent to at most one every two seconds.
11. Handle connection states explicitly: `connected`, `reconnecting`, `offline`. Show a health pill and disable mutations while offline.

---

## 6. Design system and pixel UI

1. Use only the design tokens in `web/src/styles/`: color ramp, spacing scale, radii, hard shadows, z-index and motion durations.
2. Use the shared pixel UI kit. Do not create a one-off button, panel, table or modal.
3. Every component defines default, hover, active, focus-visible, disabled, loading and error states.
4. Keep the pixel aesthetic: 1 px borders, bevels, hard shadows and no rounded SaaS cards. Scanlines are at most 4 percent opacity and are disabled under reduced motion.
5. Use the 16×16 pixel icon set with `shape-rendering="crispEdges"`. Do not mix icon styles.
6. Do not use emoji in the office or in operational UI.
7. Copy is concise and operational. Errors explain the next action and never use exclamation marks.
8. Components are presentational. They receive inputs and emit outputs; they do not fetch data.
9. Do not introduce a CSS framework or a second design language.

---

## 7. Virtual office engine

1. Render at an internal resolution of 384×216. Scale by an integer factor with nearest-neighbor. Never use fractional scaling or smoothing.
2. Use a fixed 60 Hz simulation step with an accumulator and render on `requestAnimationFrame`. Pause simulation in a background tab and reconcile from the latest snapshot on resume.
3. Keep the engine framework-agnostic. It receives a plain snapshot and emits interaction events; it must not import Angular.
4. Use A* on the tile grid with 8 directions, no corner cutting, an octile heuristic and a bounded expansion count.
5. Drive agent animation exclusively from backend events. Never invent work that is not happening.
6. Map states exactly: idle, walking, typing, reading, thinking, blocked, celebrating and offline.
7. Cache static layers offscreen. Batch draws by atlas. Target at most 150 draw calls per frame.
8. Use object pools for particles and bubbles. Do not allocate in the hot loop.
9. Speech bubbles are at most 120 characters, three lines and six seconds; clicking one opens the inspector.
10. Every interactive object is clickable and keyboard reachable: agent, desk, whiteboard, server rack, bookshelf, filing cabinet, meeting table, door, coffee machine, plants and pets.
11. Provide `data-testid` hooks for agents and objects.
12. Honor `prefers-reduced-motion`: no walking, ambient motion or parallax; keep state changes and text.
13. Provide a screen-reader fallback list of agents with name, role, status, current card and actions.
14. Every asset must have a redistributable license recorded in the office documentation. Never copy reference-product assets.

---

## 8. Accessibility

1. Target WCAG 2.2 AA for every DOM surface.
2. Every interactive element is reachable by keyboard in a logical order. Dialogs and drawers trap focus and close on `Escape`.
3. Never remove focus outlines. Provide a visible `:focus-visible` state.
4. Label every input. Link errors with `aria-describedby`. Use `aria-live` for toasts and status changes.
5. Text contrast is at least 4.5:1; large text and UI boundaries at least 3:1.
6. Provide a keyboard alternative for drag-and-drop.
7. Touch targets are at least 44×44 CSS pixels.
8. Support zoom to 200 percent without loss of content or function.
9. Do not convey status by color alone; pair it with an icon and text.
10. Run axe on every route and fix every critical or serious violation.

---

## 9. Performance

1. Meet the budgets: initial JS at most 350 KB gzip, office chunk at most 180 KB gzip, CSS at most 60 KB gzip.
2. Meet the experience budgets: LCP at most 2.5 seconds, TTI at most 3.5 seconds, Lighthouse performance at least 90 desktop.
3. Use `OnPush` and signals. Do not run change detection per frame or per WebSocket event.
4. Lazy-load routes. Preload the office chunk after idle. Do not preload every feature.
5. Virtualize lists over 100 rows.
6. Avoid layout shift on live updates. Reserve space for badges, counters and activity rows.
7. Cancel stale requests and ignore out-of-order responses.
8. Keep the JS heap at most 150 MB. Release large buffers and image atlases when leaving the office.
9. Measure with Lighthouse CI and the office frame-rate probe. Record the result in the work package.

---

## 10. UX states and copy

1. Every screen defines loading, empty, error, degraded and ready states.
2. Loading uses skeletons or the pixel progress bar. Do not use a fake progress indicator.
3. Empty states include a short explanation and one primary action.
4. Error states show a safe message, a retry action and the `trace_id`.
5. Degraded states use a banner when a dependency is unavailable and disable the affected actions.
6. Toasts: success and info dismiss after four seconds; errors persist until dismissed. At most three are stacked.
7. Destructive actions require typed confirmation or an explicit checkbox. Never a single unlabeled click.
8. Dates use `Intl.DateTimeFormat` in the operator timezone. The API returns UTC.
9. Money is rendered from micro-USD without floating-point arithmetic.
10. Never show a blank screen. The error boundary renders a pixel panel with retry and trace ID.

---

## 11. Security

1. Enforce the Content Security Policy: `default-src 'self'`, no inline scripts, no `eval` and no third-party scripts.
2. Store the remote bearer token in memory and `sessionStorage` only. Never in `localStorage`, a URL or a log.
3. Sanitize Markdown and never bind untrusted HTML.
4. Validate and narrow all external data, including WebSocket payloads and API errors.
5. Do not render secrets. Provider and SCM keys are write-only and only the last four characters are displayed.
6. Do not add a frontend dependency without checking the audit and license.
7. Report frontend errors with a correlation ID, route and build hash. Never include PII, prompts or tokens.

---

## 12. Testing

1. Follow Red, Green, Refactor. No production behavior without a failing test first.
2. Use Vitest with TestBed for stores, services, pipes and components.
3. Use Playwright for journeys J1 to J6: idea to backlog, idea to SOW and plan, execution, office, recovery and access.
4. Use axe for accessibility and Playwright screenshots for visual regression at 1280, 768 and 390 widths.
5. Tests are deterministic: fixed seed, fixed clock, no `sleep` and no real external API.
6. Cover loading, empty, error and degraded states.
7. Cover keyboard paths for drag-and-drop, drawers and forms.
8. Maintain frontend line coverage of at least 80 percent and critical stores and components at least 90 percent.
9. Quarantine a flaky test within 24 hours and fix or delete it within 5 days.
10. Never update a screenshot to hide an unintended regression.

---

## 13. Observability

1. Report frontend errors to the telemetry endpoint with route, build hash, correlation ID and a sanitized stack.
2. Report Web Vitals and office frame-rate samples. Aggregate them; do not send per-frame data.
3. Never include PII, prompts, tokens or repository content in telemetry.
4. Surface connection state, last event sequence and sync status in the health drawer.
5. Log to the console only in development. Production logging goes through telemetry.

---

## 14. Documentation and decisions

1. Every exported component, service, store and pipe has TSDoc.
2. Every route has a documented purpose, loading state, empty state, error state and data sources.
3. Every architectural or irreversible decision creates an ADR and a registry row.
4. Update the design-system documentation when tokens or shared components change.
5. Update `AGENTS.md` indexes when a document, skill, command or ADR changes.
6. Record commands, tests and evidence in the work package progress line.

---

## 15. Definition of Done for frontend work

- [ ] Failing test observed before production code.
- [ ] Component uses the pixel UI kit and design tokens only.
- [ ] Loading, empty, error and degraded states implemented.
- [ ] Keyboard path and focus management verified.
- [ ] axe reports zero critical or serious violations.
- [ ] Reduced motion honored.
- [ ] Playwright journey passes.
- [ ] Visual regression passes within 0.1 percent.
- [ ] Bundle and Lighthouse budgets pass.
- [ ] No per-frame change detection from WebSocket or office events.
- [ ] API types generated; no duplicate DTO.
- [ ] Security checks pass; no secret in the bundle or storage.
- [ ] Coverage thresholds met.
- [ ] Docs and ADR updated where required.
- [ ] Evidence linked from the work package.

---

## 16. Forbidden patterns

- NgModules, `zone.js`, or relying on zone-based change detection.
- Per-frame or per-event Angular change detection.
- Hand-written API DTOs instead of generated types.
- Direct `HttpClient` calls from components; use the store and `ApiClient`.
- `any` outside a validated parsing boundary.
- `innerHTML` with untrusted content or unsanitized Markdown.
- Storing tokens in `localStorage`, a URL or a log.
- Ad-hoc components or styles outside the design system.
- Fake progress, untruthful animations or state that contradicts the server.
- Mouse-only interactions, missing focus states or color-only status.
- Updating screenshots to hide a regression.
- Adding a third-party script, CSS framework or charting library without an ADR.
- Copying assets from the reference product.

---

## 17. References

- `./technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md`
- `./technical-specification/APPENDIX-B-CONTRACTS.md`
- `./technical-specification/APPENDIX-C-AGENTS-KANBAN.md`
- `./technical-specification/APPENDIX-D-DELIVERY.md`
- `./sow/SOW.md` and the target `WP-XX-*.md`
- `./sow/WP-17-FRONTEND-PLATFORM.md`
- `./sow/WP-18-VIRTUAL-OFFICE.md`
- `./sow/WP-19-PRODUCT-UI.md`
- `./CONSTITUTION.md`, `./PRINCIPLES.md`, `./STACK.md`
- `./adr/registry.md`
- `../AGENTS.md`
