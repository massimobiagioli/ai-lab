# ADR-0026 — BDD E2E suite with Playwright-BDD and a make-only gate

| Field | Value |
|---|---|
| Status | Accepted |
| Date | 2026-11-09 |
| Deciders | QA, Tech Lead, Frontend |
| Supersedes | — |
| Superseded by | — |
| Related | ADR-0008, ADR-0021, ADR-0024 |

## Context

The project already has a plain Playwright suite in `web/e2e`. The end-to-end
contract is now expressed as 480 Gherkin scenarios in `.docs/e2e/BDD.md`, and
the manual setup contract is in `.docs/e2e/req.md`. The suite must be
deterministic, must not call real DeepSeek or GitHub APIs by default, must
produce evidence mapped to `E2E-XXX` ids, and must not run accidentally against
an unprepared environment. The repository has Node 24 and Playwright already,
but no JVM toolchain.

## Decision

1. Use **`playwright-bdd` 9.x on top of `@playwright/test` 1.65** as the BDD
   runtime. It reuses the existing Node/TypeScript toolchain and the Playwright
   browser stack, and it generates Playwright tests from Gherkin features.
2. Put the suite in `web/e2e-bdd/` with its own config, features, steps,
   support, baselines and reports. Keep `web/e2e` untouched.
3. Generate the feature files from `.docs/e2e/BDD.md` with
   `.tools/gen-bdd-features.mjs`. The generated files are deterministic,
   idempotent and never hand-edited.
4. Expose exactly two make targets: `make bdd` runs the suite and
   `make bdd-set-api-key` loads the live DeepSeek key. No public npm, shell or
   direct Playwright entry point is supported.
5. Guard the suite in three places: the Makefile, `scripts/run-bdd.sh` and
   `web/e2e-bdd/playwright.config.ts`. Direct invocation fails with a clear
   make-only message.
6. Gate execution on `.docs/e2e/req.md` with `scripts/bdd-preflight.mjs`. The
   checklist is minimal and manual-only: the current entry is the DeepSeek API
   key for live runs. An unchecked item exits `2` before feature generation or
   any test starts.
7. Load the live DeepSeek key only through `scripts/bdd-set-api-key.sh`: hidden
   prompt, validation, provider create or rotate through the provider API,
   never an argument, a file, an environment variable, a log or a report.
8. Run automated scenarios with `AILAB_DEV_FAKES=1`; tag real-credential
   scenarios `@live` and skip them unless `AILAB_BDD_LIVE=1`.
9. Emit `web/e2e-bdd/reports/results.json` with one entry per `E2E-XXX` id,
   plus HTML, JUnit, traces, screenshots and videos on failure.

## Consequences

- One test runtime and one browser stack; no JVM or Karate toolchain.
- The generated features stay in sync with the BDD checklist through the
  generator and coverage drift checks.
- The suite cannot be launched outside `make bdd`, and it cannot start on an
  unprepared environment.
- The preflight is deliberately narrow: only manual operations belong in
  `.docs/e2e/req.md`; anything a script can provision is not requested there.
- Pending step definitions are reported by the coverage report and fail loudly
  at execution; they are never silently skipped or marked passed.
- Visual certification (sections 22 and 23) uses golden screenshots, canvas
  probes, frame hashes, FPS samples and axe, all mapped to `E2E-XXX` evidence.

## Alternatives considered

- **Karate**: rejected because it needs a JVM toolchain that is not in the
  pinned stack, and it would duplicate the Playwright browser stack.
- **@cucumber/cucumber without Playwright**: rejected because it would add a
  second runner and a second browser/API integration.
- **A public `npm run bdd` script**: rejected because the suite must be
  launchable only through `make bdd`.
- **Storing the DeepSeek key in `.env`**: rejected because the provider API
  stores it encrypted with AES-256-GCM and the key must never be a file or an
  environment variable.

## Compliance

- `scripts/run-bdd.sh` enforces the guard, the preflight, the drift and coverage
  checks, the stack readiness check, `bddgen` and Playwright, in that order.
- `web/e2e-bdd/playwright.config.ts` throws without
  `AILAB_BDD_VIA_MAKE=1`.
- `.tools/gen-bdd-features.mjs --check` and
  `.tools/check-bdd-coverage.mjs` enforce feature drift and the 480-id contract.
- `.docs/product/testing.md` documents `make bdd` and
  `make bdd-set-api-key`. The `AGENTS.md` Tool registry lists the new tools.
