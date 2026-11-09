# AI-Lab Virtual Office

| Field | Value |
|---|---|
| Document | `.docs/product/office.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/frontend.md`, `web/src/lib/office/`, `web/src/app/features/office/` |

The office is the emotional core of AI-Lab. It is a 16-bit pixel office driven exclusively by backend state: agents never show activity that is not happening.

## 1. Rendering model

- Internal resolution 384×216 (24×13 tiles of 16 px, 4 px top offset), integer scaling, nearest-neighbour.
- Canvas 2D (optional WebGL2 backend later).
- Target ≥ 58 FPS desktop, ≥ 30 FPS low-end.
- Office engine decoupled from Angular (ADR-0021).

## 2. Engine and host

The framework-agnostic engine lives in `web/src/lib/office/` (ADR-0021):

- `layout.ts`: deterministic 24×13 grid, walls and five desks.
- `pathfinding.ts`: deterministic 4-directional A* with a stable tie-break.
- `agent-state.ts`: truthful domain-state → animation mapping.
- `engine.ts`: agents, A* movement, per-frame interpolation and reduced-motion snapping.
- `sprites.ts`: generated 8×8 sprite atlas and CC0 manifest (no third-party assets).
- `web/scripts/lint-assets.mjs`: build-failing asset linter (licence, 8×8 frames, palette).

Interactions cover agents and seven objects (whiteboard, rack, bookshelf, cabinet, coffee, plant, door); the
objects open a drawer with a route. `?agent=<id>` and `?panel=<id>` restore the selection/panel. The
simulation uses a fixed 1/60 s step and exposes a stable frame hash for determinism.

The Angular host is `web/src/app/features/office/office.ts`: a 384×216 pixelated canvas, y-sorted
sprite agents, an accessible agent list, click-to-select/click-to-walk, a path overlay, an FPS
counter, `?agent=<id>` deep link and a detail panel. The route is `/lab/:labId` with a slug guard.

## 3. Truthful state mapping

| Domain state | Animation |
|---|---|
| idle | idle |
| working | type |
| blocked | blocked |
| paused/offline | sleep |

Walking frames are deterministic (`floor(elapsed / 200) % 4`). The UI never invents activity.

## 4. Accessibility

- Reduced-motion mode replaces walking and typing with a static pose.
- Screen-reader fallback lists agents and statuses as text.
- Status is never color-only; the badge label carries the state.

## 5. Tests

`npm --prefix web test` covers A* paths/blocked/unreachable and the state/animation mapping.

## 6. Deferred to the WP-18 follow-up

- Sprite atlas, animation pipeline and canvas render loop.
- Clickable objects, inspector, deep links and office events.
- FPS telemetry and the canvas performance suite (WP-21).

## 7. References

- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §11
- `.docs/adr/adr-0007.md`, `.docs/adr/adr-0021.md`
- `web/src/lib/office/`
