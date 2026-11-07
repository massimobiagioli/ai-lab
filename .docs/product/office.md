# AI-Lab Virtual Office

| Field | Value |
|---|---|
| Document | `.docs/product/office.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/frontend.md`, `web/src/app/office/` |

The office is the emotional core of AI-Lab. It is a 16-bit pixel office driven exclusively by backend state: agents never show activity that is not happening.

## 1. Rendering model

- Internal resolution 384×216, integer scaling, nearest-neighbour.
- Canvas 2D (optional WebGL2 backend later).
- Target ≥ 58 FPS desktop, ≥ 30 FPS low-end.
- Office engine decoupled from Angular (ADR-0021).

## 2. Pathfinding

`findPath` implements deterministic 4-directional A* with a stable tie-break. It returns the path including start and goal, or `null` when the goal is blocked or unreachable. Tests cover straight paths, wall detours, unreachable goals and identical endpoints.

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
- `web/src/app/office/`
