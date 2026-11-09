---
name: BDD Step Engineering
description: Use when implementing or changing AI-Lab Gherkin step definitions, fixtures, selectors, the API client, the Playwright-BDD world, determinism or redaction in the E2E suite.
---

# BDD Step Engineering

## When to use

- Implement steps for new or changed `.feature` scenarios.
- Fix undefined, ambiguous or pending steps.
- Add or change fixtures, the API client, selectors, the world or redaction.
- Make a scenario deterministic and isolated.
- Convert a `@manual` scenario to an automated one.

Use `bdd-scenario-design` before changing the feature wording. Use `bdd-coverage-traceability` to prove every step is matched.

## Preconditions

- Read `.docs/e2e/BDD.md`, the generated features and the existing `web/e2e-bdd/steps/`.
- Read `web/e2e-bdd/playwright.config.ts`, `support/fixtures.ts`, `support/world.ts`, `support/api-client.ts`, `support/selectors.ts` and `support/redact.ts`.
- Know the API contracts in Appendix B and the UI selectors in the Angular app.
- Know `playwright-bdd` and Cucumber expression syntax.
- Confirm `AILAB_DEV_FAKES=1` and the deterministic seed are available.

## Workflow

1. Read the scenario and classify each step as generic or domain-specific.
2. Implement a parameterised generic step when the behaviour repeats, for example `I call {method} {path}`, `the response status is {int}` or `the response field {string} is {string}`.
3. Implement a domain step for Kanban guards, SOW lifecycle, plan materialization, run orchestration, memory scope and office state when a generic step would hide the rule.
4. Use the typed `ApiClient` for REST calls and the Playwright fixture world for `api`, `page`, `lab`, `project`, `agent` and `card`.
5. Use `support/selectors.ts` and stable `data-testid` values. Never select by brittle text when a stable id exists.
6. Create isolated fixtures per scenario with ids derived from the `E2E-XXX` id; clean up where safe.
7. Use bounded polling helpers, never sleeps. Keep timeouts in `support/env.ts`.
8. Redact tokens, keys and cookies in failure messages, traces, screenshots and reports.
9. Run the focused scenario through `make bdd` with the documented tag/filter variables, then run the full `make bdd`.
10. Run `.tools/check-bdd-coverage.mjs` and confirm zero unmatched and zero unused steps.
11. For section 22 (Amiga-style UI certification), implement the visual steps: screenshot diff at 0.1 percent with documented masks, canvas pixel sampling for palette and anti-aliasing, integer-scale and buffer checks, scanline opacity, frame-hash determinism, FPS and draw-call probes, axe, keyboard, screen reader and reduced-motion emulation.
12. For section 23 (Inventory and Lab management UI), implement the management steps: token and computed-style checks, hierarchy and spacing assertions, wizard and form flows, inline validation, tables/filters/pagination, responsive breakpoints, touch and keyboard parity, screen-reader announcements, axe and performance sampling.
13. Attach the screenshot, frame hash, FPS sample, axe result, performance sample and logs for every certification scenario to `results.json` by `E2E-XXX`. Never widen a threshold, edit a baseline or mask a real regression to make a visual step pass.
14. For narrative scenarios without a curated step, use `.tools/gen-bdd-steps.mjs --write` to regenerate `steps/generated.steps.ts`. Do not hand-edit it; implement the semantic handler in `support/generated-step-handler.ts` and replace curated gaps with real steps over time.

## Checklist

- [ ] Each step is reusable and typed.
- [ ] No ambiguous step matches.
- [ ] No one-off duplication of an existing step.
- [ ] `steps/generated.steps.ts` is regenerated, not hand-edited.
- [ ] Fixtures are isolated, uniquely named and cleaned up.
- [ ] Selectors use `data-testid` where available.
- [ ] No wall-clock sleeps.
- [ ] Failure messages include the `E2E-XXX` id and the failing step.
- [ ] Secrets are redacted in all artifacts.
- [ ] Focused run and full run both use `make bdd`.
- [ ] Coverage and drift checks pass.
- [ ] No real DeepSeek or GitHub API is called.
- [ ] Section-22 visual steps assert palette, integer scaling, anti-aliasing, scanline, frame hash, FPS and accessibility with attached evidence.
- [ ] Section-23 management steps assert tokens, hierarchy, wizards, validation, responsive layout, a11y, performance and visual baselines with attached evidence.
- [ ] No baseline, threshold or mask is changed to hide a regression.

## Anti-patterns

- Duplicating a step body instead of parameterising it.
- Using brittle CSS paths or visible copy where a stable test id exists.
- Sharing mutable state across scenarios.
- Sleeping for a fixed duration instead of polling a condition.
- Catching an error to make a scenario pass.
- Writing tokens or full prompts into traces or reports.
- Invoking Playwright or `bddgen` directly instead of `make bdd`.
- Asserting internal state instead of the API response, event or rendered behaviour.
- Widening a visual threshold, editing a golden baseline or adding a mask to hide a regression.
- Sampling a canvas after a CSS transform instead of the logical 384x216 buffer.
- Measuring FPS with wall-clock sleeps or a single frame instead of a bounded probe.

## References

- `../../../.docs/e2e/BDD.md`
- `../../../.docs/technical-specification/APPENDIX-B-CONTRACTS.md`
- `../../../.docs/product/frontend.md`
- `../../../.docs/product/testing.md`
- `../bdd-e2e/SKILL.md`

## Supporting files

- `../../../.prompts/bdd-suite-test-create.md`
