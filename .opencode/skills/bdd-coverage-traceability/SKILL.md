---
name: BDD Coverage & Traceability
description: Use when generating features from the AI-Lab BDD checklist, checking the 480 E2E ids, mapping scenarios to steps and evidence, or maintaining the coverage matrix and drift gates.
---

# BDD Coverage & Traceability

## When to use

- `.docs/e2e/BDD.md` changes or a new gap-closure scenario is added.
- Feature generation or a drift check fails.
- The coverage check reports a missing, duplicate or orphan id.
- A `@manual` or `@live` scenario must be mapped.
- Results and evidence must be traced back to `E2E-XXX` ids.

Use `bdd-scenario-design` to author scenarios and `bdd-step-engineering` to implement them.

## Preconditions

- Read `.docs/e2e/BDD.md`; the expected count is 480 unique ids `E2E-001` … `E2E-480`.
- Read `.tools/gen-bdd-features.mjs`, `.tools/gen-bdd-steps.mjs`, `.tools/check-bdd-coverage.mjs`, `web/e2e-bdd/manual-scenarios.md` and `web/e2e-bdd/reports/results.json`.
- Know the area-to-feature mapping and the generated feature layout.
- Know the fake harness and the deterministic seed.

## Workflow

1. Inventory the ids: fail on duplicates, gaps, malformed ids or a count other than 480.
2. Generate one feature file per `## N.` area with `.tools/gen-bdd-features.mjs --write`:
   - preserve area order, scenario order, wording and tags;
   - strip the checklist marker from the generated title;
   - add `@E2E-XXX` as the first scenario tag;
   - preserve the step text and resolve `And`/`But` to the previous primary keyword so patterns stay unambiguous.
3. Generate narrative steps with `.tools/gen-bdd-steps.mjs --write`.
4. Verify with `.tools/gen-bdd-features.mjs --check` and `.tools/gen-bdd-steps.mjs --check` that the generated files are not stale.
5. Verify with `.tools/check-bdd-coverage.mjs` that:
   - every id appears exactly once;
   - every scenario has at least one matching step definition or a `@manual` entry with owner, reason and review date;
   - no id is missing or duplicated.
6. Keep `web/e2e-bdd/manual-scenarios.md` current for every `@manual` scenario.
7. Keep the live map current for every `@live` scenario and the condition under which it runs.
8. After execution, map every `results.json` entry to its `E2E-XXX` id, status and artifacts.
9. Update the coverage summary, progress tracker and gap-analysis sections when the scenario set changes.
9. Never renumber or delete an id. Append new gaps with the next free id.
10. Run the coverage check in `run-bdd.sh` and in CI before the suite.

## Checklist

- [ ] Exactly 480 unique ids.
- [ ] No duplicate, missing or malformed id.
- [ ] One generated feature per area.
- [ ] Generated features are not stale.
- [ ] Every id appears in exactly one scenario.
- [ ] Every scenario has a matching step or a `@manual` entry with owner and reason.
- [ ] No unused step definitions.
- [ ] `results.json` has one entry per id.
- [ ] Every result references its evidence or artifact.
- [ ] Coverage summary, progress tracker and gap analysis are consistent.
- [ ] No id was renumbered or deleted.

## Anti-patterns

- Hand-editing generated `.feature` files.
- Renumbering or deleting an id to close a coverage gap.
- Omitting a scenario without a `@manual` entry.
- Counting scenarios that have no step definition as covered.
- Leaving stale generated features in the repository.
- Producing a report without the `E2E-XXX` id.
- Automatically editing `BDD.md` markers from the runner.
- Writing secrets or full prompts into evidence.

## References

- `../../../.docs/e2e/BDD.md`
- `../../../.docs/e2e/req.md`
- `../../../.docs/product/testing.md`
- `../../../.prompts/bdd-suite-test-create.md` §5, §6, §10
- `../bdd-e2e/SKILL.md`

## Supporting files

- `../../../.prompts/bdd-suite-test-create.md`
