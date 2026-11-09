# WP-18 — Virtual Office Engine Review Plan

| Field | Value |
|---|---|
| WP | `WP-18` |
| Contract | `.docs/sow/WP-18-VIRTUAL-OFFICE.md` |
| Review branch | `feat/wp-18-virtual-office` (PR #5, based on WP-17) |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `approved` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering spec, deliverables, tasks, acceptance criteria, DoD, risks and references present and consistent.
- [x] CR-0014 reopens WP-18; status history and progress log record the reopen.

## 2. Specification conformance

- [x] Framework-agnostic engine under `web/src/lib/office/` (ADR-0021) — evidence: `layout.ts`, `pathfinding.ts`, `agent-state.ts`, `engine.ts`, `sprites.ts`.
- [x] Deterministic 8-directional octile A* (10/14, octile heuristic, no corner cutting, binary heap, 4,096 cap) — evidence: `pathfinding.ts`, `pathfinding.spec.ts` (6 tests).
- [x] 24×13 grid at 384×216 internal resolution with 16 px tiles and integer 1–6× scaling — evidence: `layout.ts`, `office.ts` (`updateScale`).
- [x] Sprite atlas generated in-repo under CC0-1.0 with an 8×8 pattern per animation — evidence: `sprites.ts`, `sprites.spec.ts`.
- [x] Tile map, desks, collision, occupancy, object footprints and deterministic y-sort (y then id) — evidence: `layout.ts`, `engine.ts` (occupancy), `office.ts`.
- [x] Truthful state → animation mapping driven by WS `lab.agent.status_changed` (`payload.to`); agents start idle, never invented work — evidence: `agent-state.ts`, `engine.ts`, `office.ts` (effect), `office.ws.spec.ts`.
- [x] Interactions: agent list, click-to-select, click-to-walk, move-to-desk, path overlay and the seven object drawers (whiteboard/rack/bookshelf/cabinet/coffee/plant/door) — evidence: `office.ts`, `office.spec.ts`.
- [x] Deep link `?agent=<id>&panel=cv` restores agent focus and the CV panel; furniture `?panel=<id>` opens the object drawer — evidence: `office.ts`, `office.deep-link.spec.ts`.
- [x] Responsive layout, reduced-motion snapping and an accessible agent list — evidence: `office.ts`, `styles`.
- [x] Fixed 1/60 s timestep with a 3-step accumulator, stable frame hash and the FPS counter — evidence: `office.ts`, `engine.ts` (`frameHash`), `engine.spec.ts`.
- [ ] Ambient life, sound controls and the high-contrast theme — deferred to WP-21 (named, owned, due 2026-12-31).
- [ ] Speech bubbles — deferred to WP-19 (named, owned, due 2026-12-31).
- [ ] High-contrast theme and 32-colour palette — deferred to WP-21 (owned, due 2026-12-31).

## 3. Deliverables

- [x] `web/src/lib/office` engine (layout, A*, simulation, sprites).
- [x] Canvas host at 384×216 with pixelated rendering.
- [x] Colour tokens/icons/pixel font (from WP-17) and a generated sprite atlas with a CC0 manifest.
- [x] Tile map, desks, occupancy, collision and y-sort.
- [x] A* pathfinder + walkability.
- [x] State→animation mapping.
- [ ] Speech bubbles — deferred to WP-19 (named, owned, due 2026-12-31).
- [x] Interactions, click-to-walk and `?agent` deep link; camera focus is not required on a single-screen 24×13 office.
- [ ] Ambient life and sound controls — deferred to WP-21 (owned, due 2026-12-31).
- [x] Responsive behaviour, reduced motion and screen-reader fallback.
- [x] FPS counter; heap telemetry and benchmark evidence deferred to WP-21.
- [ ] Visual/FPS/a11y suites — deferred to WP-21 (owned, due 2026-12-31).
- [x] `.docs/product/office.md` updated for the host increment.

## 4. Acceptance criteria

1. [ ] 60 FPS on the reference machine — deferred to the WP-21 performance run.
2. [x] Agent animation is driven only by backend state — evidence: `agent-state.ts`, `engine.ts`, `office.ts` WS effect; initial states are idle.
3. [x] Octile A* takes diagonal shortcuts, does not cut blocked corners and returns null when unreachable — evidence: `pathfinding.spec.ts`.
4. [x] Click-to-walk and the agent list both move/select agents — evidence: `office.spec.ts`, `engine.spec.ts`.
5. [x] Reduced motion snaps to the target — evidence: `engine.spec.ts`.
6. [x] No third-party asset is loaded; the atlas is generated in-repo with a CC0 manifest and a build-failing `lint-assets` check — evidence: `sprites.ts`, `sprites.spec.ts`, `web/scripts/lint-assets.mjs`, `npm run lint`.
7. [ ] Screen-reader and keyboard journey evidence — deferred to WP-21 axe run; the agent list is the accessible control.
8. [ ] Visual regression — deferred to WP-21.

## 5. Implementation and tests

- [x] A*/engine determinism — tests: `pathfinding.spec.ts`, `engine.spec.ts`.
- [x] Sprite manifest/licence — test: `sprites.spec.ts`.
- [x] Office host selection — test: `office.spec.ts`.
- [x] No real external API in tests.
- [ ] Coverage/visual/FPS thresholds — WP-21.

## 6. Security and quality gates

- [x] No secrets; no third-party assets; generated sprites only.
- [x] `scripts/harness-lint.sh` green.
- [x] `scripts/sow-lint.sh` green.
- [x] `make check` green.
- [x] `npm run lint/test/build/format:check`.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/office.md` updated (engine, resolution, sprites, deep link, FPS).
- [x] ADR-0021 respected (engine has no Angular imports).
- [x] `AGENTS.md` documentation index updated for this review plan.
- [x] `.docs/sow/SOW.md` and the WP status/progress updated.

## 8. QA review

- [x] QA subagent verdict: `APPROVED` (HEAD `a0f91db`).
- [x] QA evidence: qa-review-agent returned APPROVED with all AC/deliverables verified or owned deferrals; 66 tests, gates green.
- [x] Every blocking defect fixed and re-reviewed (three passes).

## 9. Deferred items and residual risks

- [ ] Speech bubbles — owner: Dev, card: `WP-19`, due: 2026-12-31.
- [ ] Ambient life, sound, high-contrast theme — owner: Dev, card: `WP-21`, due: 2026-12-31.
- [ ] Visual/FPS/a11y suites and the 60 FPS benchmark — owner: QA, card: `WP-21`, due: 2026-12-31.

## 10. Decisions

- [x] CR-0014 — reopen WP-18; alternatives and consequences recorded.
2026-11-07 | fix | web/src/{lib/office,app/features/office}, web/scripts/lint-assets.mjs | octile A*; integer scaling; fixed timestep+hash; occupancy; WS-driven states; objects/drawers; deep-link panel; asset linter; 64 web tests | ambient/sound/visual/a11y/60fps deferred | QA re-review
2026-11-07 | fix | web/src/app/features/office/office.ts, office.{ws,deep-link}.spec.ts | WS payload.to; ?panel=cv focus/CV; deep-link tests; 66 web tests | ambient/sound/visual/a11y/60fps deferred | QA re-review
