# AI-Lab — BDD E2E Suite Creation Prompt

> **Purpose.** This prompt instructs an AI agent to build a complete, deterministic, Gherkin-based end-to-end test suite for AI-Lab. The suite MUST implement every scenario in `.docs/e2e/BDD.md`, MUST run only through `make bdd`, and MUST refuse to start when any manual prerequisite in `.docs/e2e/req.md` is missing. This prompt is a self-contained implementation contract; it does not ask the agent to change product behaviour.

---

## 0. Role and mission

You are the **QA Automation Engineer** for AI-Lab. Your mission is to deliver a BDD E2E suite that:

1. covers all 480 scenarios in `.docs/e2e/BDD.md` with stable `E2E-XXX` identifiers;
2. uses Gherkin feature files and executable step definitions;
3. is launchable **only** through `make bdd`;
4. refuses to start, with an explicit error message, when `.docs/e2e/req.md` is incomplete;
5. runs deterministically against the local stack and the fake LLM/SCM harness without real external APIs;
6. produces machine-readable and human-readable evidence that maps every result to its `E2E-XXX` id.

You do not redesign the product, rewrite the scenarios, or weaken the specification. You implement the verification layer only.

---

## 1. Mandatory inputs

Read these files before writing anything. Do not skip any.

1. `.docs/e2e/BDD.md` — the 480-scenario source of truth for scope and wording.
2. `.docs/e2e/req.md` — the manual pre-test checklist that gates execution.
3. `.docs/product/testing.md` — test layers, fixtures, seed and flaky policy.
4. `.docs/product/operations.md` — stack, profiles, health and reset rules.
5. `.docs/product/security.md` — exposure modes, secrets, sandbox and webhooks.
6. `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §15–§19 and the appendices B and C for API, event and Kanban contracts.
7. `AGENTS.md` — operating rules, tool registry, documentation index and quality gates.
8. `.opencode/skills/bdd-e2e/SKILL.md` — the project skill for the BDD suite, feature generation, steps, preflight and `make bdd`.
9. The specialist BDD skills, loaded for their phase: `.opencode/skills/bdd-scenario-design/SKILL.md`, `.opencode/skills/bdd-step-engineering/SKILL.md`, `.opencode/skills/bdd-preflight-gate/SKILL.md`, `.opencode/skills/bdd-coverage-traceability/SKILL.md` and `.opencode/skills/bdd-execution-triage/SKILL.md`.
10. `Makefile`, `web/package.json`, `web/playwright.config.ts`, `web/e2e/` and `scripts/` — existing commands and conventions.
11. `.docs/adr/registry.md` — ADR numbering and format.

If an input is missing or unreadable, record it and stop; do not invent content.

---

## 2. Hard constraints

1. **Make-only entry point.** The suite MUST be launchable only through `make bdd`. Do not add a public npm script, a standalone shell entry point, or a documented direct command. Direct invocation of the underlying runner MUST fail with a clear message. `make bdd-set-api-key` is a setup command, not a test runner, and does not change this rule.
2. **Preflight gate.** If any required manual item in `.docs/e2e/req.md` is missing, `make bdd` MUST exit non-zero, print a message that something required is missing, list the missing items with section and line, and MUST NOT start the suite, the feature generator or any test.
3. **Full coverage.** All 480 unique `E2E-001` … `E2E-480` scenarios MUST be represented. No scenario may be silently omitted. A scenario that cannot be automated MUST be tagged `@manual` and listed in a documented manual-scenario map with a reason; it still counts as covered.
4. **No real external APIs by default.** Automated runs MUST use the fake LLM/SCM harness (`AILAB_DEV_FAKES=1`). Live scenarios are opt-in with `@live` and `AILAB_BDD_LIVE=1`; they are skipped by default and reported as skipped, never as passed.
5. **Determinism.** Use the deterministic demo seed or isolated API fixtures. No wall-clock sleeps. Use the application clock and polling helpers. Tests must be repeatable from a clean seeded environment.
6. **English only.** Features, step definitions, code, reports, comments and error messages are English.
7. **No secrets.** Never write API keys, tokens, cookies or personal data into features, steps, reports, traces or logs. Redact secrets in failure output. The DeepSeek key is loaded only through the hidden-prompt `make bdd-set-api-key` command; it is never accepted as a command-line argument, written to `.env`, or committed.
8. **No product changes.** Do not modify `crates/`, `web/src/`, migrations, DDL or product behaviour. This prompt produces test code, runner scripts, tooling and documentation updates only.
9. **No destructive suite.** `make bdd` must not call `make reset`, drop volumes, force-push, delete the demo lab or run destructive SQL. Destructive scenarios use disposable fixtures created and removed through the API.
10. **No questions.** Where a decision is required, choose the option that best fits the stack, follow the existing conventions, and record a material decision as an ADR.

---

## 3. Library and architecture decision

Use **`playwright-bdd` on top of `@playwright/test` and Cucumber Gherkin** as the primary implementation. The repository already uses Playwright, Node 24 and TypeScript; this gives one runtime, one browser stack and one report format.

- Add `playwright-bdd` as a `web/` dev dependency. Do not add a JVM toolchain.
- Do not use Karate unless an accepted ADR adds a Java toolchain to the stack; Karate is not the default.
- Keep the existing `web/e2e` Playwright suite untouched.
- Put the BDD suite under `web/e2e-bdd/` with its own Playwright config.
- Use `bddgen` to generate the Playwright tests from the `.feature` files.
- Use Cucumber expressions or regular expressions with typed parameters; never duplicate the same step body for many scenarios.
- Record the library, the Make-only guard and the preflight gate in a new ADR, using the next free `adr-nnnn.md` number from `.docs/adr/registry.md`, and update the registry in the same change.

---

## 4. Output contract (exact file tree)

Create or update exactly the following. Do not add unrelated tooling.

```text
Makefile                                   # add the bdd and bdd-set-api-key targets
scripts/
├── bdd-preflight.mjs                      # parses .docs/e2e/req.md and gates execution
├── bdd-preflight.test.mjs                 # unit tests for the parser
├── bdd-set-api-key.sh                     # hidden-prompt key loader; create or rotate the provider
├── bdd-set-api-key.test.sh                # fake-transport tests for the key loader
└── run-bdd.sh                             # guarded runner: preflight -> bddgen -> playwright
.tools/
├── gen-bdd-features.mjs                   # BDD.md -> .feature generator
├── gen-bdd-steps.mjs                      # exact-match steps for narrative scenarios
├── check-bdd-coverage.mjs                 # 480-id coverage and drift check
└── bdd-self-test.sh                       # fixture-based self-test for the tools
web/
├── package.json                           # add playwright-bdd; no public bdd script
├── package-lock.json
└── e2e-bdd/
    ├── playwright.config.ts               # make-only guard + bdd config + reporters
    ├── features/                          # generated .feature files, one per BDD.md area
    │   ├── 01-health.feature
    │   ├── ...
    │   └── 21-gap-closure.feature
    ├── steps/
    │   ├── common.steps.ts
    │   ├── api.steps.ts
    │   ├── ui.steps.ts
    │   ├── kanban.steps.ts
    │   ├── delivery.steps.ts
    │   ├── security.steps.ts
    │   └── ...
    ├── support/
    │   ├── fixtures.ts
    │   ├── world.ts
    │   ├── api-client.ts
    │   ├── selectors.ts
    │   ├── env.ts
    │   ├── redact.ts
    │   ├── visual.ts                     # screenshot diff, masks and baseline registry
    │   ├── office-probe.ts               # canvas pixel sampling, scale and FPS probes
    │   └── frame-hash.ts                 # deterministic office frame hash helper
    ├── visual-baselines/                  # golden screenshots per viewport/DPR/theme/motion
    ├── manual-scenarios.md                # @manual scenarios with owner and reason
    └── reports/                           # generated: junit, html, json, traces, videos
.docs/
└── adr/
    ├── adr-nnnn-....md                    # BDD library, make-only entry and preflight gate
    └── registry.md
AGENTS.md                                  # tool registry rows for the new tools
.docs/product/testing.md                   # document make bdd, make bdd-set-api-key and the preflight gate
.docs/STACK.md                             # only if the command surface changed
```

Rules:

- Generated `.feature` files are deterministic and idempotent. Rerunning the generator on unchanged `BDD.md` produces byte-identical files.
- `reports/` is gitignored; do not commit test artifacts.
- Every new tool under `.tools/` or `scripts/` MUST be registered in the `AGENTS.md` Tool registry, with path, purpose and invocation, in the same change.
- If a tool invocation changes `make` or repository commands, update `.docs/STACK.md` in the same change.
- Update the Documentation index in `AGENTS.md` only if you create a new document under `.docs/`.

---

## 5. Phase 1 — Inventory and coverage map

Before writing tests:

1. Parse `.docs/e2e/BDD.md` and count the scenarios. The expected count is **480** unique identifiers `E2E-001` … `E2E-480`.
2. Detect duplicates, gaps and malformed ids. Fail with a precise report if the source is inconsistent.
3. Map each `## N.` area to exactly one feature file. Preserve the area order.
4. Map each scenario's priority tags (`@p0`, `@p1`, `@p2`) and traceability tags (`@fr-*`, `@ux-*`, `@sec-*`, `@ops-*`, `@ac-*`).
5. Produce `.tools/gen-bdd-features.mjs` that:
   - accepts `--check` to fail on drift without writing;
   - accepts `--write` to regenerate the feature files;
   - extracts the `Feature:` line, tags, `Scenario: [ ] E2E-XXX ...` title and the Gherkin steps;
   - strips the checklist marker from the generated scenario title, for example `Scenario: E2E-001 ...`;
   - adds `@E2E-001` as the first tag on the scenario;
   - preserves the step text exactly and resolves `And`/`But` to the previous primary keyword so duplicate patterns do not become ambiguous;
   - writes one feature file per area;
   - fails on a duplicate id, a missing id or a scenario without a `Then`.
6. Produce `.tools/check-bdd-coverage.mjs` that verifies:
   - `.docs/e2e/BDD.md` contains exactly 480 unique ids;
   - every id exists in exactly one generated `.feature` scenario;
   - every generated scenario has at least one matching step definition, or is listed in `web/e2e-bdd/manual-scenarios.md` with a reason and an owner;
   - no step definition is unused when the suite is complete;
   - the generated features are not stale (`gen-bdd-features --check`).
7. Produce `.tools/gen-bdd-steps.mjs` that generates exact-match step definitions for narrative scenarios that lack a curated step. It MUST:
   - exclude the curated step files when computing the pending inventory;
   - emit one definition per unique keyword and step text;
   - delegate to a semantic dispatcher that executes API, UI, visual and accessibility patterns and fails loudly with `UnsupportedStepError` for unimplemented assertions;
   - be deterministic and support `--check` and `--write`.
8. Add a fixture-based self-test `.tools/bdd-self-test.sh` that proves the generator and coverage checker fail on a missing id, a duplicate id, a stale feature and an unmatched step.

---

## 6. Phase 2 — Feature generation

1. The source of truth is `.docs/e2e/BDD.md`. Never hand-edit generated `.feature` files; regenerate them.
2. Keep the scenario wording, ids and tags. Do not rewrite steps to make implementation easier; implement the steps instead.
3. Each feature file starts with the area tags, `Feature:`, and then the scenarios in the same order as `BDD.md`.
4. If a scenario is genuinely not automatable, keep it in the feature file, tag it `@manual`, and list it in `manual-scenarios.md` with:
   - `E2E-XXX`, owner, reason, required human evidence, review date.
   A `@manual` scenario is reported as skipped with the reason, never as passed.
5. Use `@live` for scenarios that require a real DeepSeek or GitHub key. They are skipped unless `AILAB_BDD_LIVE=1` and the preflight permits live runs.
6. Use `@p0`, `@p1`, `@p2` to support selective runs, but `make bdd` runs the full suite by default.

---

## 7. Phase 3 — Step definitions and support

1. Implement a small, reusable step library. Prefer parameterised steps such as `I call {method} {path}`, `the response status is {int}`, `the response field {string} is {string}`, `I open {string}`, `I click {string}`, `I see {string}`.
2. Keep domain-specific steps for Kanban transitions, DoR/DoD violations, SOW lifecycle, plan materialization, run orchestration, memory scopes and office state where a generic step would hide the behaviour.
3. Use selectors from `support/selectors.ts`; use `data-testid` where available. Never select by brittle text when a stable id exists.
4. Use a typed `ApiClient` in `support/api-client.ts` for REST calls; reuse the existing API contract and error shapes.
5. Use one Playwright fixture world that exposes `api`, `page`, `lab`, `project`, `agent` and `card` handles. Create fixtures through the API, with unique names, and clean them up where the scenario allows.
6. Use deterministic IDs and names derived from the scenario id, for example `E2E-057-lab`. Never depend on execution order or shared mutable state.
7. Use polling helpers with a bounded timeout instead of sleeps. Read the timeout from `support/env.ts`.
8. Select the target URL from the environment or config, not from hardcoded feature text. Features describe intent, not hostnames or ports.
9. Redact secrets and tokens in all failure messages, traces and reports.
10. Make step definitions fail loudly with the scenario id and the failing step in the message.

### 7b. Visual certification steps (section 22)

Section 22 certifies the Amiga-style office. Implement dedicated visual steps with these rules:

1. Use Playwright screenshot assertions with a maximum changed-pixel ratio of 0.1 percent and only documented dynamic masks. Store baselines under `visual-baselines/` per viewport, DPR, theme and reduced-motion setting.
2. Use `office-probe.ts` to read canvas pixels with `getImageData` and assert:
   - the buffer is exactly 384x216 and the CSS scale is an integer;
   - every sampled colour belongs to the fixed palette;
   - sprite edges have no anti-aliasing blend;
   - scanline opacity is at most 4 percent and off under reduced motion.
3. Use `frame-hash.ts` to assert determinism for the same event sequence and seed, and to compare reconnect and background-tab catch-up.
4. Measure FPS with a bounded `requestAnimationFrame` probe and assert the desktop and low-end targets. Measure the frame budget, draw calls and heap where the scenario requires it.
5. Use axe with WCAG 2.2 AA for the office, and keyboard, screen-reader and reduced-motion emulation where the scenario requires it.
6. Capture a screenshot, frame hash, FPS sample and the relevant logs for every section-22 scenario and reference them from `results.json` by `E2E-XXX`.
7. Never make a visual step pass by widening the threshold, editing a baseline or masking a real regression. A visual change requires an intentional baseline update in a separate reviewed change.
8. The suite must be able to run the section-22 visual matrix (1x, 2x, 3x at DPR 1 and 2, reduced motion on and off) reproducibly from the deterministic seed.
9. For section 23 (Inventory and Lab management UI), implement management-certification steps:
   - assert the design tokens and computed styles, visual hierarchy, spacing rhythm and one primary action per screen;
   - drive every wizard, form, inline validation, table, filter, sort and pagination state;
   - exercise desktop, tablet and phone breakpoints, orientation change, touch targets and 200 percent text zoom;
   - run axe WCAG 2.2 AA, keyboard-only and screen-reader checks;
   - capture golden screenshots for agents, projects, providers and settings at each breakpoint and compare at the 0.1 percent threshold;
   - sample load and interaction performance and attach the evidence per `E2E-XXX`.
10. No management or office baseline, threshold or mask may be changed to hide a regression.

---

## 8. Phase 4 — Preflight gate (`.docs/e2e/req.md`)

The preflight is mandatory and runs before any test generation or execution. Implement it in `scripts/bdd-preflight.mjs` with unit tests in `scripts/bdd-preflight.test.mjs`.

### 8.1 Parsing rules

1. Read `.docs/e2e/req.md`. If the file is missing, empty or unreadable, exit `2` with `BDD suite not started: .docs/e2e/req.md is missing or unreadable.`
2. The file is a minimal manual setup checklist. It currently contains exactly one entry: the DeepSeek API key for live runs. It has no numbered sections, no stop conditions and no sign-off table; do not require or invent them.
3. Every `- [ ]` line is a missing prerequisite unless the same line contains `N/A` or `n/a`. A `- [x]` line is satisfied.
4. Ignore checkbox-shaped text inside fenced code blocks.
5. Extra checked items are optional setup notes and never block execution.
6. Do not add requirements that are not written in `.docs/e2e/req.md`. Anything the tooling can provision automatically is out of scope for this gate.

### 8.2 Output and exit codes

- `0`: every checkbox in `.docs/e2e/req.md` is `[x]` or `N/A`.
- `2`: at least one checkbox is `[ ]`. Print to stderr:

```text
BDD suite not started: missing manual setup in .docs/e2e/req.md.
Missing items:
  - line 5: DeepSeek API key (sk-..., valid and funded). Put it in AI-Lab at ...
Fix the checklist in .docs/e2e/req.md and run `make bdd` again. The suite was not started.
```

- The message MUST name the file, every missing item and its line, and state that the suite was not started. It MUST be printed before any generator or test process starts.
- Support `--json` for machine-readable output; the default is human-readable text.
- Provide `--help` that documents the rule and the exit codes.

### 8.3 Unit tests

Cover at least: all checked, one unchecked, `N/A` handling, a missing file, an empty file, a fenced checkbox that must be ignored, an extra checked item that does not block, a malformed line and a `--json` output.

---

## 8b. Phase 4b — API key loading command (`make bdd-set-api-key`)

The DeepSeek API key is the only manual prerequisite for live runs. Add a dedicated setup command so the operator never edits `.env` or a tracked file. Automated default runs still use the fake LLM harness and do not require the key.

Add exactly one setup target to the `Makefile`:

```make
.PHONY: bdd-set-api-key
bdd-set-api-key: ## Load the DeepSeek API key for live BDD runs (hidden prompt)
	@scripts/bdd-set-api-key.sh
```

Implement `scripts/bdd-set-api-key.sh` with these rules:

1. Read the key from a hidden terminal prompt (`read -rs`). Never accept it as a command-line argument, environment variable, file path or shell history entry.
2. Validate the key before sending it: `sk-` prefix, 20–256 characters, no whitespace.
3. Verify that the prepared stack is reachable. If it is not, exit `2` with a clear message and do not store the key.
4. Create the DeepSeek provider if it does not exist; otherwise rotate the existing provider key. Use the documented provider API and the provider contract; never write directly to the database.
5. Send the key only over the loopback API. In remote mode require the operator token and TLS. Never echo the key or include it in an error.
6. Print only the provider status and the last four characters.
7. Clear the key from memory on every exit path with a `trap`, and never enable shell tracing.
8. Be idempotent and safe to run repeatedly.
9. Do not write the key to `.env`, a secret file, git, logs, reports or the terminal scrollback.
10. Provide `--help`. Do not provide a non-interactive flag that accepts the key on the command line; tests use a fake provider transport instead.
11. Add `scripts/bdd-set-api-key.test.sh` covering: empty input, wrong prefix, short key, long key, provider create, provider rotate, unreachable stack, no echo, trap cleanup and idempotent rerun.
12. Do not require `AILAB_BDD_VIA_MAKE=1`; the key loader is a setup command, not the test runner. Only `make bdd` runs the suite.

Update `.docs/e2e/req.md` so its only entry says: obtain the DeepSeek key, run `make bdd-set-api-key`, and enter it at the hidden prompt. Do not ask the operator to edit `.env` or a source file. The preflight gate checks the checklist marker only; it never reads or stores the key.

---

## 9. Phase 5 — Make-only entry point and guard

Add the two user-facing targets to the `Makefile`:

```make
.PHONY: bdd bdd-set-api-key
bdd-set-api-key: ## Load the DeepSeek API key for live BDD runs (hidden prompt)
	@scripts/bdd-set-api-key.sh
bdd: ## Run the BDD E2E suite (requires .docs/e2e/req.md to be complete)
	@AILAB_BDD_VIA_MAKE=1 scripts/run-bdd.sh
```

Implement `scripts/run-bdd.sh` with this order and no shortcuts:

1. If `AILAB_BDD_VIA_MAKE` is not `1`, print `bdd: run this suite only via 'make bdd'.` to stderr and exit `2`.
2. Run `node scripts/bdd-preflight.mjs`. If it exits non-zero, stop immediately; do not generate features and do not run tests.
3. Run the coverage/drift check `.tools/gen-bdd-features.mjs --check` and `.tools/check-bdd-coverage.mjs`. If either fails, stop with a precise message.
4. Verify that the prepared stack is reachable through the configured BDD base URL. If it is not reachable, print `BDD suite not started: the prepared test environment is not reachable.` and exit `2`; do not start Docker, do not run `make up`, do not reset or seed data.
5. Run `npx bddgen --config web/e2e-bdd/playwright.config.ts`.
6. Run `npx playwright test --config web/e2e-bdd/playwright.config.ts` with the configured reporters.
7. Write the result summary and exit with the Playwright exit code.

Enforce the guard in three places:

- `scripts/run-bdd.sh` (first check);
- `web/e2e-bdd/playwright.config.ts` (throw if `AILAB_BDD_VIA_MAKE !== '1'`);
- a short README section or comment in `web/e2e-bdd/` stating that only `make bdd` is supported.

Do not add a public `npm run bdd` script. If an internal npm script is unavoidable, name it `bdd:internal` and make it fail unless the guard is set; do not document it as a supported entry point.

---

## 10. Phase 6 — Reporting and evidence

1. Reporters: `list`, `html`, `junit` and a custom JSON reporter that emits `web/e2e-bdd/reports/results.json`.
2. Each result MUST contain: `id` (`E2E-XXX`), `title`, `area`, `priority`, `status` (`passed|failed|skipped`), `duration_ms`, `error` (redacted), `tags` and `artifact_paths`.
3. On failure, capture a Playwright trace, a screenshot and a video where configured. Store them under `reports/` and reference them in the JSON result.
4. Print a final console summary: total, passed, failed, skipped, manual, live, and the list of failed `E2E-XXX` ids.
5. Exit non-zero if any non-skipped scenario fails.
6. Do not edit `.docs/e2e/BDD.md` automatically. The QA reviewer updates the checklist markers from `results.json` and the evidence, following the rules in `BDD.md`.
7. Fail the run if a feature file or a step definition drifts from `BDD.md`, or if the coverage check reports a missing or duplicate id.
8. `make bdd` MUST write the full console output to `web/e2e-bdd/reports/make-bdd.log` (override with `AILAB_BDD_LOG`), in addition to the structured reports. The log covers the preflight message, the drift and coverage checks, `bddgen`, Playwright and the final summary.

---

## 11. Phase 7 — Test data and execution rules

1. Run against the prepared local stack. Do not auto-start, auto-reset or auto-seed; the manual preflight is the preparation gate.
2. Use `AILAB_DEV_FAKES=1` for all automated scenarios. Never call the real DeepSeek or GitHub APIs.
3. Create isolated fixtures per feature or scenario through the API: unique lab, project, agents, cards, SOW and plan names derived from the scenario id. Clean up where safe.
4. Use the deterministic seed only for scenarios that explicitly reference the demo data. Never mutate the seeded lab in a way that breaks later scenarios.
5. Set the test timezone, locale and reduced-motion preference explicitly in the Playwright context where the scenario needs them.
6. Use one worker locally unless a scenario explicitly requires concurrency; keep the suite deterministic. If parallelism is enabled, all fixtures must be isolated.
7. Quarantine policy: a flaky scenario is tagged `@quarantine`, reported as skipped with an owner and a card, and never silently retried. Follow `.docs/product/testing.md`.

---

## 12. Phase 8 — Documentation, registry and ADR

1. Add the tools to the `AGENTS.md` Tool registry:
   - `bdd-preflight` — validates `.docs/e2e/req.md` and blocks execution;
   - `gen-bdd-features` — generates feature files from `.docs/e2e/BDD.md`;
   - `gen-bdd-steps` — generates exact-match steps for narrative scenarios;
   - `check-bdd-coverage` — verifies 480-id coverage and drift;
   - `run-bdd` — guarded make-only runner.
2. Document `make bdd`, the preflight gate and the make-only rule in `.docs/product/testing.md`.
3. Create the ADR for the BDD library, the Make-only entry point and the preflight gate; update `.docs/adr/registry.md` in the same change.
4. Update `.docs/STACK.md` only if the command surface or a pinned test dependency changes.
5. Run `scripts/harness-lint.sh` and `.tools/check-doc-links.sh` after the documentation changes.

---

## 13. Phase 9 — Verification (definition of done)

The work is complete only when all of the following pass:

1. `.tools/gen-bdd-features.mjs --check` reports no drift.
2. `.tools/gen-bdd-steps.mjs --check` reports no drift and `.tools/check-bdd-coverage.mjs` reports exactly 480 unique ids, zero missing, zero duplicate and zero unmatched steps.
3. `node scripts/bdd-preflight.test.mjs` passes.
4. `.tools/bdd-self-test.sh` passes its negative fixtures.
5. `make bdd` with an incomplete `.docs/e2e/req.md` exits `2`, prints the missing-items message and does not run any test.
6. `make bdd` with a complete `.docs/e2e/req.md` and a reachable prepared stack generates the features and runs the suite.
7. Direct invocation of `scripts/run-bdd.sh` without `AILAB_BDD_VIA_MAKE=1` exits `2` with the make-only message.
8. Direct invocation of `npx playwright test --config web/e2e-bdd/playwright.config.ts` without the guard fails with the make-only message.
9. The suite produces `results.json` with one entry per `E2E-001` … `E2E-480`; manual and live scenarios appear as skipped with a reason. `make bdd` also writes the full console output to `web/e2e-bdd/reports/make-bdd.log`.
10. No test calls the real DeepSeek or GitHub API.
11. No secret appears in features, steps, reports, traces or logs.
12. `make check`, `npm --prefix web run lint`, `npm --prefix web run build`, `scripts/harness-lint.sh` and `.tools/check-doc-links.sh` still pass.
13. The ADR is accepted and the registry is updated.
14. The `AGENTS.md` Tool registry and the documentation index are consistent.
15. `make bdd-set-api-key` exists, prompts with hidden input, validates the key, creates or rotates the DeepSeek provider, prints only the status and last four characters, writes the key nowhere on disk and is idempotent.
16. `scripts/bdd-set-api-key.test.sh` passes with a fake provider transport and proves there is no command-line key input and no key echo.
17. The section-22 visual certification matrix (1x, 2x, 3x at DPR 1 and 2, reduced motion on and off) runs reproducibly and produces palette, integer-scale, anti-aliasing, scanline, frame-hash, FPS, memory, axe and visual-diff evidence per `E2E-XXX`.
18. No visual threshold, baseline or mask is weakened to make a section-22 scenario pass.
19. The section-23 management certification runs at desktop, tablet and phone widths and produces token, hierarchy, wizard, validation, responsive, a11y, performance and visual-diff evidence per `E2E-XXX`.
20. No management baseline, threshold or mask is weakened to make a section-23 scenario pass.

---

## 14. Non-goals

- Do not change the product specification, the SOW, the ADRs unrelated to this suite, or the scenarios in `.docs/e2e/BDD.md`.
- Do not implement missing product features to make a scenario pass. Report the gap as a defect.
- Do not weaken a scenario, skip it silently, or mark it passed without evidence.
- Do not add real credentials to the repository, CI, features, reports or logs.
- Do not accept the DeepSeek key as a command-line argument or write it to `.env`, a secret file or a source file.
- Do not add a CI job that runs `@live` scenarios with real keys.
- Do not make the suite runnable outside `make bdd`.

---

## 15. Final report

Return a concise report with:

1. the files created or updated;
2. the library and ADR number;
3. the exact `make bdd-set-api-key` and `make bdd` commands and the preflight message format;
4. the coverage count and the manual/live/skipped scenario map;
5. the verification commands run and their results;
6. any residual gap, with a defect reference and the affected `E2E-XXX` ids.
