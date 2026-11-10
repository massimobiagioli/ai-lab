---
name: BDD E2E Suite
description: Use when creating, changing, running or debugging the Gherkin BDD end-to-end suite, its feature generator, step definitions, preflight gate or the make bdd entry point.
---

# BDD E2E Suite

## When to use

- Create or update the BDD E2E suite from `.docs/e2e/BDD.md`.
- Change feature generation, step definitions, fixtures, selectors, redaction or reports.
- Change the `make bdd` entry point, the make-only guard or the `.docs/e2e/req.md` preflight gate.
- Diagnose coverage drift, duplicate or missing `E2E-XXX` ids, unmatched steps or a preflight that blocks execution.
- Add a manual-scenario mapping or a live-scenario opt-in.

Do not use this skill for the plain Playwright specs in `web/e2e`, for product code, or for load, soak and chaos suites. Use `quality-gates` for the final gate and `tdd-angular` for the product UI.

## Preconditions

- Read `.docs/e2e/BDD.md` and confirm exactly 480 unique ids `E2E-001` … `E2E-480`.
- Read `.docs/e2e/suites.json` and confirm the six ordered suites assign every id exactly once.
- Read `.docs/e2e/req.md` and confirm it is the minimal manual setup checklist. The only expected manual entry is the DeepSeek API key for live runs, loaded with `make bdd-set-api-key`; automated runs use the fake harness.
- Read `.docs/product/testing.md` for commands, fixtures, seed and flaky policy.
- Read `AGENTS.md` for the tool registry, documentation index and quality gates.
- Read the ADR that records the BDD library, the make-only entry point and the preflight gate.
- Confirm the prepared stack is reachable; do not start or reset it from the suite.
- Confirm `AILAB_DEV_FAKES=1` is available and no real DeepSeek or GitHub API is called by default.

## Workflow

1. Verify the source contract:
   - `scripts/run-node.sh .tools/gen-bdd-features.mjs --check` reports no drift.
   - `scripts/run-node.sh .tools/gen-bdd-steps.mjs --check` reports no drift.
   - `scripts/run-node.sh .tools/check-bdd-coverage.mjs` reports 480 unique ids, zero missing and zero duplicate.
2. Generate features and narrative steps when `BDD.md` changed:
   - `scripts/run-node.sh .tools/gen-bdd-features.mjs --write`
   - `scripts/run-node.sh .tools/gen-bdd-steps.mjs --write`
   - Keep one feature file per `## N.` area, preserve order, wording and tags, and add `@E2E-XXX` to each scenario.
   - Strip the checklist marker from the generated scenario title, never from `BDD.md`.
3. Implement or update steps:
   - Reuse parameterised steps before adding new ones.
   - Use `support/selectors.ts` and `data-testid`; never rely on brittle text when a stable id exists.
   - Use the typed `support/api-client.ts` for REST calls and the Playwright fixture world for `api`, `page`, `lab`, `project`, `agent` and `card`.
   - Use polling with a bounded timeout, never wall-clock sleeps.
   - Redact secrets and tokens in every failure message, trace and report.
4. Maintain the preflight gate:
   - `scripts/bdd-preflight.mjs` reads `.docs/e2e/req.md` and exits `2` when any `- [ ]` item is missing.
   - `- [x]` is satisfied; `N/A` or `n/a` on the same line is accepted; a missing or unreadable file exits `2`.
   - There are no stop-condition or sign-off sections in the minimal checklist; do not invent them.
   - The error message names the file, the missing item and its section and line, and states that the suite was not started.
5. Maintain the make-only entry point:
   - `Makefile` exposes `bdd`, `bdd-list` and `bdd-set-api-key`; only `bdd` runs the suite, and only `bdd-set-api-key` loads the live key.
   - `make bdd SUITE=<id>` runs one suite, `make bdd UP_TO=<id>` runs the ordered prefix and `make bdd` runs all six.
   - `scripts/run-bdd.sh` refuses to run without the guard, runs the preflight first, then the drift check, then resolves the suite selection, then `bddgen`, then Playwright with the matching `--grep`.
   - `web/e2e-bdd/playwright.config.ts` also refuses to run without the guard.
   - Do not add a public `npm run bdd` script.
6. Load the live key only through `make bdd-set-api-key`:
   - The command reads the key from a hidden prompt, validates it and creates or rotates the DeepSeek provider.
   - It never accepts the key as an argument, writes it to disk or environment, echoes it or stores it in history.
   - Automated default runs do not need the key.
7. Report results:
   - Emit `web/e2e-bdd/reports/results.json` with `id`, title, area, priority, status, duration, error and artifact paths.
   - Capture trace, screenshot and video on failure.
   - Write the full console output to `web/e2e-bdd/reports/make-bdd.log` (override with `AILAB_BDD_LOG`).
   - Print the totals and the failed `E2E-XXX` ids.
   - Do not edit `BDD.md` automatically; the QA reviewer updates the marker from the report and evidence.
8. Update documentation and governance:
   - Register every new tool in the `AGENTS.md` Tool registry.
   - Document `make bdd`, `make bdd-set-api-key` and the preflight in `.docs/product/testing.md`.
   - Create or update the BDD ADR and `.docs/adr/registry.md` in the same change.
   - Run `scripts/harness-lint.sh` and `.tools/check-doc-links.sh`.

## Checklist

- [ ] `.docs/e2e/BDD.md` still has 480 unique ids.
- [ ] Generated features are not stale and contain every id exactly once.
- [ ] `generated.steps.ts` is not stale and covers every narrative step.
- [ ] Every step matches a definition, or the scenario is `@manual` with an owner and a reason.
- [ ] No real DeepSeek or GitHub API is called by default.
- [ ] `make bdd` is the only supported suite entry point.
- [ ] `make bdd-set-api-key` is the only supported way to load the live DeepSeek key, and it never accepts the key as an argument or writes it to disk.
- [ ] Direct runner and direct Playwright config invocation fail with the make-only message.
- [ ] An incomplete `.docs/e2e/req.md` blocks the suite with the missing-items message and exits `2`.
- [ ] A complete `.docs/e2e/req.md` lets the suite run.
- [ ] `scripts/bdd-preflight.test.mjs`, `scripts/bdd-suites.test.mjs` and `.tools/bdd-self-test.sh` pass.
- [ ] `.docs/e2e/suites.json` assigns every id to exactly one of the six suites.
- [ ] `make bdd-list` prints six suites; `SUITE=` and `UP_TO=` select the expected counts.
- [ ] The full run `results.json` has one entry per `E2E-001` … `E2E-480`; a suite run contains the selected entries only.
- [ ] `make bdd` writes the full console output to `reports/make-bdd.log`.
- [ ] Section 22 (Amiga-style UI) attaches palette, scaling, frame-hash, FPS and visual-diff evidence.
- [ ] Section 23 (Inventory and Lab management UI) attaches token, hierarchy, responsive, a11y, performance and visual evidence.
- [ ] `@manual` and `@live` scenarios are reported as skipped with a reason.
- [ ] No secret appears in features, steps, reports, traces or logs.
- [ ] `make check`, `scripts/harness-lint.sh` and `.tools/check-doc-links.sh` still pass.

## Anti-patterns

- Hand-editing generated `.feature` files instead of regenerating them.
- Renumbering or deleting an `E2E-XXX` id to make a test pass.
- Silently skipping a scenario without a `@manual` entry, owner and reason.
- Adding a public npm, shell or direct Playwright entry point that bypasses `make bdd`.
- Starting, resetting or seeding the stack from the suite instead of using the prepared environment.
- Treating unchecked stop-condition or sign-off items as missing when the checklist does not contain them.
- Calling the real DeepSeek or GitHub API from the default suite.
- Catching a failure and marking the scenario passed.
- Writing secrets, tokens or full prompts into reports or traces.
- Editing `BDD.md` markers from the runner without human review.

## Related skills

Load the specialist skill for the current phase:

- `bdd-scenario-design` — author, refine or review scenarios and requirement mapping.
- `bdd-step-engineering` — implement steps, fixtures, selectors, API client and redaction.
- `bdd-preflight-gate` — change `req.md`, the parser, the make-only guard or the error message.
- `bdd-coverage-traceability` — prove 480-id coverage, generate features and map evidence.
- `bdd-execution-triage` — run `make bdd`, interpret results, triage failures and quarantine flakes.

## References

- `../../../.prompts/bdd-suite-test-create.md`
- `../../../.docs/e2e/BDD.md`
- `../../../.docs/e2e/req.md`
- `../../../.docs/product/testing.md`
- `../../../.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §19
- `../../../.docs/sow/WP-21-TESTING.md`

## Supporting files

- `../../../.prompts/bdd-suite-test-create.md`
