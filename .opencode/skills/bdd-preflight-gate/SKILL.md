---
name: BDD Preflight Gate
description: Use when changing the manual prerequisites checklist, the make-only guard, the preflight parser, the exit codes or the fail-fast message for the AI-Lab BDD suite.
---

# BDD Preflight Gate

## When to use

- Change `.docs/e2e/req.md`, the preflight parser or its unit tests.
- Change `make bdd`, `scripts/run-bdd.sh` or the guard in the Playwright config.
- Add or remove a manual prerequisite.
- Diagnose a suite that refuses to start.
- Adjust the missing-prerequisite message or exit code.

Use `bdd-execution-triage` after the gate passes to interpret the run.

## Preconditions

- Read `.docs/e2e/req.md`, `scripts/run-bdd.sh` and `web/e2e-bdd/playwright.config.ts`.
- Read `.prompts/bdd-suite-test-create.md` §8 and §9.
- Know what the tooling can provision automatically: stack, database, Redis, master key, runner token, seed, fakes, browser, fixtures.
- Know the exit-code contract: `0` ready, `2` blocked.

## Workflow

1. Keep `.docs/e2e/req.md` minimal: only operations a human must perform. At present the only entry is the DeepSeek API key for live runs, loaded with `make bdd-set-api-key`; the default suite uses the fake LLM harness.
2. Do not add anything the scripts, the app or CI can derive or verify: URLs, ports, versions, environment defaults, health checks, migrations, seed counts, harness availability or command syntax.
3. Preflight parser rules:
   - a missing, empty or unreadable file exits `2`;
   - every `- [ ]` line is a missing prerequisite unless the line contains `N/A` or `n/a`;
   - an item marked `(live)` is required only with `--live` (the runner passes it when `AILAB_BDD_LIVE=1`);
   - `- [x]` is satisfied;
   - checkbox-shaped text inside a fenced code block is ignored;
   - extra checked items are optional and never block;
   - no numbered sections, stop conditions or sign-off table are required or invented.
4. Error message rules: print to stderr, name the file, list every missing item with its line, state that the suite was not started, and exit `2` before any generator or test process starts.
5. Make-only and setup targets:
   - the `Makefile` exposes `bdd` and `bdd-set-api-key`;
   - `bdd` invokes `AILAB_BDD_VIA_MAKE=1 scripts/run-bdd.sh` and is the only suite entry point;
   - `bdd-set-api-key` invokes `scripts/bdd-set-api-key.sh`, which reads the key from a hidden prompt and creates or rotates the DeepSeek provider;
   - the key loader never accepts the key as an argument, writes it to disk or environment, echoes it or stores it in history;
   - `scripts/run-bdd.sh` refuses to run without the guard;
   - `web/e2e-bdd/playwright.config.ts` also refuses without the guard;
   - no public npm, shell or direct Playwright entry point exists.
6. Execution order inside `run-bdd.sh`: guard, preflight, drift check, coverage check, stack readiness, `bddgen`, Playwright.
7. The gate never starts, resets or seeds the stack. If the prepared environment is unreachable, exit `2` with a clear message.
8. Add unit tests in `scripts/bdd-preflight.test.mjs` and a fixture self-test in `.tools/bdd-self-test.sh`.
9. Update the ADR, `.docs/product/testing.md` and the `AGENTS.md` Tool registry.

## Checklist

- [ ] `.docs/e2e/req.md` contains only manual operations.
- [ ] No auto-provisioned item is requested manually.
- [ ] Parser handles checked, unchecked, `N/A`, fenced and missing-file cases.
- [ ] Exit `0` only when every checkbox is checked or `N/A`.
- [ ] Exit `2` prints the missing items with line numbers.
- [ ] The suite is not started when the gate fails.
- [ ] The guard exists in the Makefile, the runner and the Playwright config.
- [ ] `make bdd-set-api-key` loads the live key from a hidden prompt and writes it nowhere on disk.
- [ ] No public npm or direct runner entry point.
- [ ] Unit tests and the self-test pass.
- [ ] `make bdd` blocks and allows as intended.
- [ ] Documentation and ADR are updated.

## Anti-patterns

- Asking a human to set what a script can provision.
- Adding URLs, ports, versions or environment defaults to the manual checklist.
- Running the preflight after `bddgen` or after the first test.
- Printing a generic failure without the missing item and line.
- Enforcing the guard only in the Makefile.
- Silently starting, resetting or seeding the stack.
- Inventing stop-condition or sign-off sections that the checklist does not contain.

## References

- `../../../.docs/e2e/req.md`
- `../../../.docs/product/testing.md`
- `../../../.docs/product/operations.md`
- `../../../.prompts/bdd-suite-test-create.md` §8–§9
- `../bdd-e2e/SKILL.md`

## Supporting files

- `../../../.prompts/bdd-suite-test-create.md`
