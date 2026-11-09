# AI-Lab BDD E2E suite

This directory contains the Gherkin BDD suite generated from `.docs/e2e/BDD.md`.

**The only supported entry point is `make bdd`.** The Playwright config refuses to run without the make-only guard, and `scripts/run-bdd.sh` refuses direct invocation. `make bdd-set-api-key` is the only supported way to load a live DeepSeek key; it never accepts the key as an argument or writes it to disk.

The suite runs against the prepared local stack with `AILAB_DEV_FAKES=1`. It does not start, reset or seed the stack. Read the preflight checklist in `.docs/e2e/req.md` first.

`make bdd` writes the full console output to `reports/make-bdd.log` (override with `AILAB_BDD_LOG`) and the structured evidence to `reports/results.json`, `reports/html/` and `reports/junit.xml`. The generated `steps/generated.steps.ts` covers the narrative scenarios that do not yet have a curated step; regenerate it with `.tools/gen-bdd-steps.mjs --write`.
