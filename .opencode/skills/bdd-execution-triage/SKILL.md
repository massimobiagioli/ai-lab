---
name: BDD Execution & Triage
description: Use when running the AI-Lab BDD suite through make bdd, interpreting results.json, triaging failures, quarantining flaky scenarios or collecting release evidence per E2E id.
---

# BDD Execution & Triage

## When to use

- Execute or re-run the BDD suite.
- Interpret `results.json`, HTML or JUnit output.
- Triage a failure as a product defect, a test defect, an environment issue or a flake.
- Quarantine a flaky scenario.
- Collect evidence for a release or an acceptance report.

Use `bdd-preflight-gate` when the suite refuses to start and `bdd-step-engineering` when a step is undefined or wrong.

## Preconditions

- `.docs/e2e/req.md` is complete.
- The prepared stack is reachable and uses the fake LLM and SCM harness.
- For live runs, the DeepSeek key is loaded with `make bdd-set-api-key`.
- The only supported suite entry point is `make bdd`.
- `web/e2e-bdd/reports/` is writable.
- The tester has access to the issue tracker and the checklist.

## Workflow

1. Complete every manual item in `.docs/e2e/req.md`. Do not edit the parser to bypass a missing item. For live runs, load the key with `make bdd-set-api-key`; never through `.env`, a file or a command-line argument.
2. Run `make bdd`. Do not invoke `scripts/run-bdd.sh`, `npx bddgen` or `npx playwright` directly.
3. If the preflight blocks, fix the checklist and re-run. The message lists the missing item and its line.
4. If the guard blocks, use `make bdd`. If the environment is unreachable, prepare it manually; the suite never starts or resets it.
5. Read `web/e2e-bdd/reports/results.json` and `web/e2e-bdd/reports/make-bdd.log`, then group failures by area and by step.
6. Reproduce a failure with the documented `make bdd` tag or filter variables. Do not add a direct runner.
7. Classify the failure:
   - product defect: link an `E2E-XXX` id, a trace, a screenshot, the commit and the seed revision;
   - test defect: open a card and fix the step or fixture;
   - environment: record the missing preparation and re-run;
   - flake: quarantine with `@quarantine`, an owner and a card within 24 hours, and fix or delete within 5 days;
   - `@manual` or `@live`: verify the reason and the evidence, never report it as passed.
8. Create a defect card with severity, the failing `E2E-XXX` ids, the artifact paths and the reproduction command.
9. Update the `.docs/e2e/BDD.md` marker manually from the evidence: `[x]` passed, `[!]` failed, `[-]` blocked or quarantined.
10. Attach the evidence to the acceptance report or the owning work package.
11. Re-run `make bdd` after a fix and confirm the full suite is green or documented.

## Checklist

- [ ] `.docs/e2e/req.md` is complete.
- [ ] The run used `make bdd`.
- [ ] For live runs, the key was loaded with `make bdd-set-api-key` and never through `.env`, a file or an argument.
- [ ] `results.json` contains one entry per `E2E-XXX` id.
- [ ] `reports/make-bdd.log` contains the full console output of the run.
- [ ] Every failure is classified and linked to a defect or a fix.
- [ ] Flaky scenarios are quarantined with an owner and a card.
- [ ] `@manual` and `@live` are reported as skipped with a reason.
- [ ] Checklist markers are updated with evidence.
- [ ] Evidence is attached to the acceptance report or work package.
- [ ] No secret appears in artifacts, traces or logs.
- [ ] The rerun is green or the residual gap is documented.

## Anti-patterns

- Invoking the runner or Playwright directly.
- Retrying until a flaky test passes.
- Marking a scenario passed without evidence.
- Changing a test to hide a product defect.
- Starting, resetting or seeding the stack from the suite.
- Ignoring `@manual`, `@live` or skipped scenarios.
- Publishing tokens, prompts or personal data in traces.
- Editing checklist markers without a green run and evidence.

## References

- `../../../.docs/e2e/BDD.md`
- `../../../.docs/e2e/req.md`
- `../../../.docs/product/testing.md`
- `../../../.docs/product/acceptance-report.md`
- `../../../.prompts/bdd-suite-test-create.md` §9–§10, §13
- `../bdd-e2e/SKILL.md`

## Supporting files

- `../../../.prompts/bdd-suite-test-create.md`
