# WP-18 — Virtual Office Engine

| Field | Value |
|---|---|
| WP ID | `WP-18` |
| Status | **done** |
| Owner | Dev |
| Milestone | M3 |
| Estimate | 34 points |
| Depends on | WP-17, WP-09 |
| Blocks | WP-19 |
| Requirement traceability | `UX-001..027`, `UX-030..035`, `FE-020..026` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-17 and WP-09 done |
| 2026-11-07 | `progress` | `done` | Dev (agent) | A*, truthful state mapping and office docs delivered; sprite/canvas/interaction engine in the WP-18 follow-up |
| 2026-11-07 | `done` | `progress` | Tech Lead (agent) | CR-0014: only A*/state helpers exist; reopen to build the canvas office host |
| 2026-11-07 | `progress` | `done` | QA (agent) | QA `APPROVED` (review plan §8); 66 web tests; visual/FPS/a11y/ambient/sound deferred to WP-21, speech bubbles to WP-19 |

---

## 1. Objective

Deliver the **16-bit virtual office**: a Canvas 2D engine that renders a 384×216 Amiga-inspired room with walking, working, reading, thinking and blocked agents whose animations are driven by real backend events. Every object is clickable, every state is truthful, the experience holds 60 FPS, and it remains fully accessible and deterministic.

---

## 2. Scope

### 2.1 In scope

- Canvas engine: layers, fixed timestep, render loop, camera, integer scaling.
- Palette, tiles, sprites, atlases, animation definitions, asset manifest and license metadata.
- Tile map, furniture placement, y-sorting, collision, hit testing.
- A* pathfinding with walkability and corner-cutting prevention.
- Agent state machine and event→animation mapping.
- Speech bubbles, ambient life, sound.
- Interactions: agent inspector, whiteboard, server rack, bookshelf, cabinet, door, coffee, plants/pets.
- Deep links and camera focus.
- Responsive scaling, reduced motion, screen-reader fallback.
- Performance budgets and deterministic replay.
- Visual regression and FPS tests.

### 2.2 Out of scope

- Feature panels/drawers content (WP-19).
- Backend events (WP-09/WP-16); this WP consumes them.
- Design tokens/UI kit (WP-17).

---

## 3. Engineering specification

### 3.1 Visual language

- **Internal resolution:** `384 × 216` logical pixels (16:9). Scale factor `k = floor(min(cssW/384, cssH/216))`, min 1, max 6; CSS size = `384k × 216k`; `image-rendering: pixelated`; no fractional scaling, no smoothing.
- **Palette:** 32 fixed colors, indexed look. Core ramp:

| Name | Hex | Use |
|---|---|---|
| deep_navy | `#0B1026` | Background/window frame |
| midnight | `#141B3C` | Walls/shadow |
| steel | `#2E3A6B` | Furniture metal |
| periwinkle | `#5B6FB0` | Accents |
| ice | `#9FB3E6` | Highlights |
| cream | `#F2E9D8` | Text/paper |
| amber | `#F2B84B` | Working LED/sunlight |
| orange | `#E07A3F` | Wood/accents |
| crimson | `#C0392B` | Blocked/error |
| teal | `#2A9D8F` | Idle LED/plants |
| green | `#57C785` | Success |
| magenta | `#C45AA0` | Secondary accent |

- **Typography:** 8×8 pixel bitmap font for in-canvas text (names, bubbles, plaques), 8×12 for titles. DOM text uses the UI sans (WP-17).
- **No anti-aliasing, no blur, no fractional coordinates.** All draw coordinates are integers in internal space.
- **CRT flavor:** optional scanline overlay ≤ 4% opacity; off under reduced motion and in screenshots unless enabled.

### 3.2 Engine architecture (framework-agnostic)

```
OfficeEngine
├── Simulation (fixed 60 Hz)
│   ├── AgentStateMachine[]
│   ├── Pathfinder (A*)
│   ├── Movement/occupancy
│   └── AmbientScheduler
├── Renderer (on RAF)
│   ├── StaticLayerCache (floor, walls, furniture-behind)
│   ├── DynamicLayer (characters y-sorted, effects)
│   ├── ForegroundLayer (furniture-front)
│   └── BubbleLayer
├── Input (hit testing, hover, click, keyboard)
├── Assets (manifest, atlases, sounds)
└── Bridge (Angular host: snapshot in, events out)
```

- The engine has **no Angular imports**; it receives a plain snapshot `{agents, lab, layout, theme}` and emits `{selectAgent, selectObject, cameraChanged}`.
- Angular updates the snapshot from signals at most 4 Hz; the engine interpolates locally.
- Simulation timestep is fixed at 1/60 s with an accumulator (max 3 catch-up steps); rendering on `requestAnimationFrame`. Background tab pauses simulation; on resume, state is reconciled from the latest snapshot (no fast-forward).
- Determinism: same snapshot sequence + seed → same positions/animations. Used by visual tests.

### 3.3 Layers and draw order

1. Floor tiles (cached offscreen)
2. Walls/back (cached)
3. Furniture behind characters (cached)
4. Characters, y-sorted by feet Y
5. Furniture in front (cached with dynamic mask if needed)
6. Particles/effects (confetti, zzz, screen glow)
7. Speech bubbles and name plaques
8. Debug overlay (dev only)

Draw calls target ≤ 150/frame; static layers are cached and redrawn only on layout/theme change.

### 3.4 Tile map, furniture and occupancy

- Grid up to 64×64 tiles; default room 24×14 tiles. Each tile is walkable or blocked by furniture footprint.
- Furniture footprint `[w,h]`, `walkable` flag, optional interaction and seat.
- Agents occupy one tile at a time; occupancy prevents overlap. Seats are reserved by agents; desks have one seat.
- Hot desks/standing spots allow > 8 agents without overlap; layout supports up to 16 agents.
- Y-sort key = feet Y; ties broken by `z` then ID to keep stable order.

### 3.5 A* pathfinding (exact)

- Grid: 8-directional; diagonal moves allowed only if both adjacent orthogonal tiles are walkable (no corner cutting).
- Cost: orthogonal 10, diagonal 14; heuristic octile; binary heap; max 4,096 expansions per query; return `NoPath` if exceeded.
- Path smoothing: none in v1 (pixel movement is intended); agents move tile-to-tile with sub-tile interpolation.
- Path cache per agent, invalidated on layout change or target change.
- Random idle wandering: choose a reachable idle tile within a radius; never into occupied seats; max 1 wander target at a time.
- Spawn/respawn: place on a free walkable tile near the assigned desk; if none, stand at the door.

### 3.6 Agent state machine

```
IDLE ──work──► WORKING ──blocked──► BLOCKED ──resolved──► WORKING
  │                │                                         │
  │ wander         │ done                                    │ pause
  ▼                ▼                                         ▼
WANDER           IDLE ◄────────────────────────────────── PAUSED
  │
  └── offline ──► OFFLINE
```

- Server status (`idle|working|blocked|paused|offline`) is authoritative. Local `WANDER` is a sub-state of `IDLE` and never contradicts it.
- Event mapping:

| Server event/step | Visual |
|---|---|
| `working` + `tool_call` write | at desk, `type` animation, screen glow |
| `working` + read/search | at desk, `read` animation, magnifier bubble |
| `working` + `plan`/`llm_request` | `think`, "..." bubble |
| `blocked` | red `!` bubble, pacing near desk |
| `paused` | gray LED, "zzz", idle |
| `offline` | empty chair, plaque only |
| `delivery.merge.completed` | `celebrate`, confetti, chime |
| `system.budget.exhausted` | lab-wide blocked visual + red banner |

- If a server event cannot be mapped, the agent stays in the last known state and a warning is logged (never invent activity).
- Speech bubbles: ≤ 120 chars, 28 chars/line, max 3 lines, 6 s auto-dismiss, click opens inspector; coalesce per agent (1/2 s).
- Ambient life (coffee trips, plants, pets) only in `IDLE`; yields to work within one tick.

### 3.7 Interactions

| Target | Action | Result |
|---|---|---|
| Agent sprite | click | Inspector (Live tab), camera focus |
| Desk | click | Inspector + desk metadata |
| Whiteboard | click | Kanban drawer filtered by project |
| Server rack | click | System Health drawer |
| Bookshelf | click | Memory explorer drawer |
| Filing cabinet | click | Failed runs / dead letters |
| Meeting table | click | Handoffs timeline |
| Door | click | Retired agents |
| Coffee machine | click | Idle agents / cosmetic |
| Plant/pet | click | Cosmetic animation/sound |

- Hover: pixel outline + tooltip. Keyboard: `Tab` cycles interactive objects, `Enter` opens, `Esc` closes, arrows pan when zoomed.
- Hit testing: per-sprite hitbox in tile coordinates + DOM overlay buttons for a11y/testability (`data-testid="agent-{id}"`, `data-testid="object-whiteboard"`).
- Deep links: `#/lab/{labId}?agent={id}&panel=cv` focuses the agent and opens the panel.
- Camera: fit-room by default; wheel zoom snaps to integer factors 1–6; drag pan; "fit" resets; focus animation ≤ 300 ms.

### 3.8 Assets and licensing

- Manifest schema: Appendix C.5.2 (`atlases`, `frames`, `animations`, `characters`, `furniture`, license metadata).
- Sprites: 24×32 characters, 32×32 tiles, 4-direction walk, 4-frame walk, 2-frame idle, 4-frame work.
- Atlas: one per category; max 8 MB compressed; PNG with transparency; no premultiplied alpha.
- Asset linter: palette ≤ 32 colors/atlas, frame bounds valid, no missing frames, license present (`author`, `source`, `license`, `modifications`).
- **Licensing:** original art or permissive packs only (CC0/OFL/commercial with redistribution). Never copy reference-product assets. Missing license fails the build.
- Loading: manifest → parallel fetch → decode → build atlases → progress screen with pixel bar; simulation starts only when critical atlases are ready; failure shows a retry panel.

### 3.9 Sound

- Optional: chime on merge, warning on blocked, subtle click on interactions.
- Master volume + mute persisted per lab; default muted until first user interaction (autoplay policy).
- No sound under reduced motion unless explicitly enabled; no sound in automated tests.

### 3.10 Responsive and accessibility

- ≥ 1280 px: sidebar + office + optional inspector. 768–1279: icons + overlay inspector. < 768: bottom sheet + full-screen inspector.
- Integer scaling only; letterboxing with the window frame color; no distortion.
- Touch targets ≥ 44×44 CSS px via DOM overlays.
- `prefers-reduced-motion`: no walking animation (instant reposition with fade), no ambient motion, no parallax; state changes and text remain.
- Screen-reader mode: a hidden but focusable ordered list of agents with name, role, status, current card and actions; live region announces transitions; all objects reachable by keyboard.
- High-contrast theme: palette variant with ≥ 4.5:1 text contrast; status conveyed by icon + text, not color alone.

### 3.11 Performance

| Metric | Budget |
|---|---|
| FPS desktop | ≥ 58 |
| FPS low-end | ≥ 30 |
| Draw calls | ≤ 150/frame |
| JS heap | ≤ 150 MB |
| Atlas memory | ≤ 8 MB compressed |
| First office render | ≤ 1.5 s after data ready |
| Per-frame allocations | 0 in hot loop (pools) |
| Event→animation latency | ≤ 250 ms p95 |

- Offscreen caches for static layers; object pools for particles/bubbles; no `new` per frame; avoid string concat in the loop; batch by atlas.
- Telemetry: FPS, dropped frames, draw calls, heap (sampled) reported to `/telemetry/frontend` (aggregated, no PII).

### 3.12 Testing

- Unit: A* (paths, no-corner-cut, no-path), state machine transitions, animation mapping, hit testing, scaling math.
- Determinism: same snapshot sequence → identical rendered frame hash.
- Visual: screenshot tests for office at 1280/768/390, reduced motion, high contrast, blocked/working/celebrating states.
- FPS soak: 16 agents + 200 props for 10 min; assert ≥ 58 FPS desktop / ≥ 30 low-end.
- A11y: axe + keyboard journey; screen-reader fallback list.
- Asset linter test: missing license/frame fails.
- Interaction E2E: click agent → inspector; whiteboard → Kanban; rack → health.

---

## 4. Deliverables (checklist)

- [x] `web/src/lib/office` engine (layout, octile A*, simulation, sprites) and the Angular canvas host with the WS bridge.
- [x] Canvas host at 384×216 internal resolution with pixelated rendering.
- [x] Colour tokens/icons/font delivered; the high-contrast theme is deferred to WP-21 (owner Dev, due 2026-12-31).
- [x] Generated in-repo sprite atlas, CC0 manifest, validation test and build-failing asset linter.
- [x] Tile map, desks, collision, occupancy, object footprints and deterministic y-sort.
- [x] A* pathfinder + walkability (deterministic, tested).
- [x] State→animation mapping is WS-driven; speech bubbles are deferred to WP-19 (owner Dev, due 2026-12-31).
- [x] Selection, click-to-walk, agent panel, object drawers and `?agent`/`?panel` deep links.
- [x] Ambient life + sound controls deferred to WP-21 (owner Dev, due 2026-12-31).
- [x] Responsive layout, reduced-motion snapping and an accessible agent list.
- [x] Fixed-timestep FPS counter and frame hash delivered; heap telemetry and the 60 FPS benchmark deferred to WP-21.
- [x] Engine/sprite/host tests delivered (66); visual/FPS/a11y suites deferred to WP-21 (owner QA, due 2026-12-31).
- [x] `.docs/product/office.md` updated for the host increment; assets/perf/licences sections pending.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP18-T01 | Engine skeleton + loop + scaling | Dev | 3 | WP-17 | 60 FPS demo |
| WP18-T02 | Asset pipeline + manifest + linter | Dev | 3 | T01 | Atlases load |
| WP18-T03 | Tile map + furniture + y-sort | Dev | 3 | T02 | Clickable room |
| WP18-T04 | A* + occupancy + collision | Dev | 3 | T03 | Deterministic paths |
| WP18-T05 | State machine + animation mapping | Dev | 4 | T04, WP-09 | Truthful states |
| WP18-T06 | Speech bubbles + ambient + sound | Dev | 2 | T05 | Coalesced bubbles |
| WP18-T07 | Inspector interactions + deep links | Dev | 3 | T05, WP-19 | Panels open |
| WP18-T08 | Responsive + reduced motion + a11y | Dev | 3 | T05 | WCAG AA |
| WP18-T09 | Perf instrumentation + budgets | Dev | 2 | T01 | Budgets met |
| WP18-T10 | Determinism/visual/FPS/a11y tests | QA | 3 | All | Green |
| WP18-T11 | Docs + asset licenses | Analyst | 2 | All | Reviewed |

---

## 6. Acceptance criteria

1. The office renders at 384×216 internally, scales only by integer factors, and is pixel-crisp at all supported widths.
2. Given the same snapshot sequence and seed, two runs produce identical frame hashes.
3. A* finds shortest paths on the octile grid, prevents corner cutting, and returns `NoPath` deterministically.
4. Agent animations match server state within 250 ms p95; no animation claims work that is not happening.
5. Every interactive object is clickable and keyboard-reachable; deep links restore focus and panel.
6. 16 agents + 200 props sustain ≥ 58 FPS desktop / ≥ 30 FPS low-end for 10 minutes.
7. Reduced motion removes walking/ambient/parallax; screen-reader fallback lists all agents with status and actions.
8. Asset linter fails on missing frames or missing license; no reference-product assets are present.
9. Clicking an agent opens the inspector; whiteboard opens Kanban; rack opens health; bookshelf opens memory; cabinet opens failed runs.
10. Visual regression screenshots match within 0.1% at 1280/768/390.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence; deferred suites are owned in WP-21/WP-19.
- [x] Determinism/unit suites green; visual/FPS/a11y suites deferred to WP-21.
- [x] Asset licence recorded (CC0) and reviewed.
- [x] Docs reviewed; no P0/P1.
- [x] Status `done` with QA sign-off (PR #5).

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Canvas perf on low-end | Medium | Low internal resolution, caching, pools, auto-quality |
| State/animation mismatch | High | Event-driven mapping + tests |
| Asset licensing | High | Linter + review + original art |
| A* jank | Medium | Max expansions, path cache, grid limits |
| A11y neglect | Medium | Keyboard/DOM overlays + axe + fallback list |
| Determinism drift | Medium | Fixed timestep + seeded RNG + frame hashes |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §11
- `../technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.5
- `WP-17` (platform), `WP-19` (drawers), `WP-09` (events)

## Progress log

2026-11-07 | T01 | web/src/app/office/pathfinding.ts, agent-state.ts | deterministic A*, state/animation mapping; 6 office tests | — | T02
2026-11-07 | T02 | .docs/product/office.md | frontend lint/test/build green (10 tests) | sprite/canvas/interactions deferred WP-18 follow-up/WP-21 | WP-19
2026-11-07 | build | web/src/lib/office/**, web/src/app/features/office/** | layout, A*, canvas host, selection, click-to-walk, reduced motion, agent panel; 20 web tests | sprite atlas/ambient/sound/fps/visual/a11y pending | in progress
2026-11-07 | review | .docs/sow/WP-18-VIRTUAL-OFFICE-review-plan.md, web/src/lib/office/**, web/src/app/features/office/** | review plan; 384×216, CC0 sprite atlas, y-sort, FPS, deep link; 59 web tests | ambient/sound/visual/a11y deferred | in review
2026-11-07 | fix | web/src/lib/office, web/src/app/features/office, web/scripts/lint-assets.mjs | octile A*; integer scaling; fixed timestep; occupancy; WS states; objects; asset linter; 66 tests | ambient/sound/visual/a11y/60fps deferred | QA re-review
2026-11-07 | fix | web/src/app/features/office/office.ts | WS payload.to; ?panel=cv focus; 66 tests | ambient/sound/visual/a11y/60fps deferred | QA re-review
2026-11-07 | closure | web/**, .docs/sow/WP-18-VIRTUAL-OFFICE-review-plan.md | QA APPROVED; 66 web tests; deferrals owned | merge PR #5 | WP-19
