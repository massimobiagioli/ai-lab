---
name: Angular TDD
description: Use for any Angular frontend change in components, stores, services, routes, the office host or Playwright tests, following Red-Green-Refactor with accessibility and performance gates.
---

# Angular TDD

## When to use

- The task changes `web/` code, routes, stores, UI kit components or the office host.
- A bug needs a failing component or E2E test.
- A new screen, interaction or visual state is required.

Do not use this skill for Rust-only or documentation-only tasks.

## Preconditions

- Node.js 24 LTS and npm 11 are active.
- `web/package-lock.json` is installed: `npm --prefix web ci`.
- The API contract is generated: `npm --prefix web run generate:api` or the checked-in generated client is current.
- The dev server can run: `npm --prefix web start`.

## Workflow

1. Read the acceptance criteria, the route, the store and the relevant design tokens.
2. Write a failing test:
   - Unit/store: Vitest with TestBed.
   - Component: TestBed with DOM assertions and keyboard interactions.
   - E2E: Playwright for the journey.
3. Run the focused test and confirm the expected failure:
   - `npm --prefix web test -- --run <pattern>`
   - `npx --prefix web playwright test <spec>`
4. Implement the minimum component, store or service change.
5. Run the focused test, then the suite:
   - `npm --prefix web test -- --run`
6. Refactor with tests green.
7. Run gates:
   - `npm --prefix web run lint`
   - `npm --prefix web run build`
   - `npx --prefix web playwright test`
   - `npx --prefix web axe <route>` or the project's axe Playwright check
8. Verify the visual state with the deterministic seed and update screenshots only when the change is intentional.
9. Check budgets: initial JS gzip, office chunk gzip, CSS gzip, LCP and TTI.
10. Record exact commands and results in the WP progress line.

## Checklist

- [ ] Failing test observed before production code.
- [ ] Test asserts user-visible behavior and accessibility.
- [ ] Component uses the pixel UI kit and design tokens.
- [ ] Signals and zoneless rules respected; no per-frame change detection.
- [ ] Loading, empty, error and degraded states covered.
- [ ] Keyboard path tested; focus management correct.
- [ ] Reduced motion honored.
- [ ] axe reports zero critical or serious violations.
- [ ] Playwright journey passes.
- [ ] Bundle and Lighthouse budgets pass.
- [ ] Screenshots updated only for intentional visual changes.
- [ ] Evidence recorded.

## Anti-patterns

- Testing implementation details instead of rendered behavior.
- Using `any` outside validated parsing boundaries.
- Introducing NgModules or zone.js assumptions.
- Adding ad-hoc styles outside the design tokens.
- Mouse-only interactions without a keyboard path.
- Updating screenshots to hide an unintended regression.
- Running the office hot loop through Angular change detection.

## References

- `../../../.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §11, §12, §19
- `../../../.docs/sow/WP-17-FRONTEND-PLATFORM.md`
- `../../../.docs/sow/WP-18-VIRTUAL-OFFICE.md`
- `../../../.docs/sow/WP-19-PRODUCT-UI.md`
- `../../../.docs/PRINCIPLES.md` §6

## Supporting files

- None.
