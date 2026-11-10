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

- Application toolchains run in Docker (ADR-0027); use `scripts/run-web.sh` and never run `npm`/`npx` on the host.

- Docker is running; all frontend commands run in the pinned Node container (ADR-0027).
- `web/package-lock.json` is installed: `scripts/run-web.sh npm ci`.
- The API contract is generated: `scripts/run-web.sh npm run generate:api` or the checked-in generated client is current.
- Use the Docker-only `make e2e` harness; do not run a host dev server.

## Workflow

1. Read the acceptance criteria, the route, the store and the relevant design tokens.
2. Write a failing test:
   - Unit/store: Vitest with TestBed.
   - Component: TestBed with DOM assertions and keyboard interactions.
   - E2E: Playwright for the journey.
3. Run the focused test and confirm the expected failure:
   - `scripts/run-web.sh npm test -- --include <pattern>`
   - `.tools/run-e2e.sh <spec>`
4. Implement the minimum component, store or service change.
5. Run the focused test, then the suite:
   - `make frontend-test`
6. Refactor with tests green.
7. Run gates:
   - `make frontend-lint`
   - `make frontend-build`
   - `make e2e`
   - `make e2e` (the axe checks run in the Playwright container)
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
