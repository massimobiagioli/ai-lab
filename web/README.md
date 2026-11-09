# AI-Lab Web

Angular 22 zoneless, signal-first frontend for AI-Lab. The shell, core services and pixel UI
primitives live here; the virtual office (WP-18) and product screens (WP-19) build on top.

## Commands

Run npm through the repository wrapper so the pinned Node toolchain is used:

```bash
scripts/run-web.sh npm install
scripts/run-web.sh npm start        # dev server
scripts/run-web.sh npm run build    # production build + budgets
scripts/run-web.sh npm run lint
scripts/run-web.sh npm test
.tools/run-e2e.sh              # Playwright + axe (official Playwright image)
```

## Structure

```
src/app/
├── core/
│   ├── api/          # typed edge client (Problem, retries, request id)
│   ├── auth/         # session-scoped token
│   ├── config/       # runtime configuration
│   ├── errors/       # global error boundary state
│   ├── telemetry/    # frontend telemetry
│   └── ws/           # single WebSocket, topics, resume, dedupe
├── domain/           # shared view models
├── features/         # dashboard, runs, placeholders for WP-18/19
├── layout/           # shell, sidebar, topbar
├── shared/
│   ├── directives/   # focus trap
│   ├── status-badge/
│   └── ui/           # button, card, empty-state, spinner, page-header
├── app.config.ts
├── app.routes.ts
└── app.ts
```

## Routes

`/` dashboard · `/runs` run list · `/lab/:labId` office host · `/projects/:projectId/board` ·
`/projects/:projectId/sow` · `/inventory/{agents,projects,providers}` · `/memory` · `/settings` ·
`**` 404. All routes are lazy-loaded; the 404 page links home.

## Runtime configuration

`window.__AILAB_CONFIG__` may override `apiBaseUrl`, `wsBaseUrl` and `token`
(`src/app/core/config/runtime-config.ts`). The token is also read from `sessionStorage`
(`ailab.token`) and never from `localStorage`.

## Accessibility

Skip link, landmarks, focus-visible outlines, `aria-live` toast host, focus trap directive and a
global `prefers-reduced-motion` rule. The error boundary announces failures with the trace id and a
retry action.
