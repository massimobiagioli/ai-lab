---
description: Execute the next implementation task with Agile/TDD and update the SOW
---

You are executing **one task** of an AI-Lab work package. Follow the source-of-truth order in `AGENTS.md`. Do not ask questions. Do not implement product code outside the selected task.

## Input

`$ARGUMENTS`

Accepted forms:

- `WP-XX` — execute that work package's first unchecked task in dependency order.
- `WP-XX/Tnn` — execute exactly that task.
- Empty — select the first WP in `.docs/sow/SOW.md` index order whose status is `ready` or `progress`, then its first unchecked task in dependency order.
- Optional flag `--commit` — after all gates pass, commit the changed files with a Conventional Commit and the required AI-Lab trailers.

## 1. Read

1. `AGENTS.md`.
2. `.docs/CONSTITUTION.md`, `.docs/PRINCIPLES.md`, `.docs/STACK.md`.
3. `.docs/sow/SOW.md` and the target `WP-XX-*.md`.
4. `.docs/adr/registry.md` and any ADR referenced by the target WP.
5. Load the `wp-delivery` skill and the specialist skill required by the task (`tdd-rust`, `tdd-angular`, `security-first`, `kanban-governance`, `api-contracts`, `data-migrations`, `agent-prompts`, `docker-operations`, `observability-sre`, `quality-gates`, `release-management` or `docs-sync`).

## 2. Select and validate

1. Resolve the target task using the input rules.
2. Confirm every dependency WP is `done` or has a recorded waiver.
3. Confirm the task is unchecked and its acceptance criteria are clear.
4. If the WP is `ready`, set it to `progress` and append a status-history row: `date | ready | progress | <actor> | <reason>`.
5. If the WP is blocked or a dependency is missing, record the blocker with category, owner, impact, next action and ETA, then stop.

## 3. TDD loop (mandatory)

For each behavior required by the task:

1. **Red** — write the smallest failing test that expresses the next acceptance criterion. Run it and confirm it fails for the expected reason, not because of an unrelated compile error.
2. **Green** — implement the minimum production code that makes the test pass. Run the focused test and the relevant suite.
3. **Refactor** — improve names and structure without changing behavior. Run all affected tests.

Repeat until every acceptance criterion is covered. Do not write production behavior before a failing test, except for generated or scaffolding files explicitly required by the task.

Use the exact commands from the specialist skill. Examples:

- Rust: `cargo nextest run -p ailab-domain <test>` then `cargo nextest run --workspace`.
- Frontend: `npm --prefix web test -- --run <pattern>` then `npx --prefix web playwright test`.
- Data: `make migrate` then the repository and concurrency tests.
- Contracts: `make openapi`, `make asyncapi`, `schemathesis run .docs/product/api/openapi.json`.

## 4. Update the work package

1. Mark the task row as done without restructuring the table.
2. Change `- [ ]` to `- [x]` only for deliverables that are actually verified.
3. Append one progress line: `YYYY-MM-DD | Tnn | files | tests | decision/ADR | next`.
4. If the task made an architectural or irreversible decision, create an ADR with the `adr-authoring` skill and update `.docs/adr/registry.md`.
5. If docs, skills, commands or diagrams changed, update the indexes in `AGENTS.md` with the `docs-sync` skill.

## 5. Gates

Run and record:

1. `scripts/harness-lint.sh`
2. The task-specific gates from the `quality-gates` skill.
3. `make check` when the task touches Rust or the frontend.
4. Security scans required by the `security-first` skill when the task touches auth, secrets, sandbox, prompts, SCM, webhooks or dependencies.

Fix every failure. Do not declare the task done with a red gate.

## 6. Status and completion

- If this was the last unchecked task **and** every deliverable is checked **and** every gate is green, set the WP status to `done` and append: `date | progress | done | <actor> | all deliverables and gates verified`.
- Otherwise leave the WP in `progress`.
- Never set a WP to `done` from `ready` without passing through `progress`.

## 7. Commit policy

- Default: do not stage, commit, amend, push or tag.
- With `--commit`: stage only the files touched by this task and commit with a Conventional Commit. Required trailers:
  ```
  AI-Lab-Run: <run_uuid or "n/a">
  AI-Lab-Card: <WP-XX/Tnn>
  AI-Lab-Agent: <agent identifier>
  ```
- Never commit secrets, build artifacts or changes outside the task scope.

## 8. Report

Return exactly:

1. **Task:** `WP-XX/Tnn` and title.
2. **Status:** WP status after the task.
3. **Files:** created and changed paths.
4. **Tests:** exact commands and results.
5. **Evidence:** links to test output, screenshots, benchmarks or logs.
6. **ADRs:** IDs created or superseded.
7. **Gates:** pass/fail per gate.
8. **Next:** the next unchecked task or the reason the WP is blocked.
