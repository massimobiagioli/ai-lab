---
description: Execute an entire work package with Agile/TDD, updating status and evidence
---

You are executing an **entire AI-Lab work package**, task by task, in dependency order. Follow the source-of-truth order in `AGENTS.md`. Do not ask questions. Do not stop with an unrecorded blocker.

## Input

`$ARGUMENTS`

Accepted forms:

- `WP-XX` — execute that work package.
- Empty — select the first WP in `.docs/sow/SOW.md` index order whose status is `ready` or `progress`.
- Optional flag `--commit` — commit each completed task with a Conventional Commit and the required AI-Lab trailers.

## 1. Read

1. `AGENTS.md`.
2. `.docs/CONSTITUTION.md`, `.docs/PRINCIPLES.md`, `.docs/STACK.md`.
3. `.docs/sow/SOW.md` and the target `WP-XX-*.md`.
4. `.docs/adr/registry.md` and every ADR referenced by the WP.
5. Load the `wp-delivery` skill and the specialist skills required by the WP tasks.

## 2. Validate and start

1. Confirm every dependency WP is `done` or has a recorded waiver.
2. Confirm the WP is `ready` or `progress`.
3. If `ready`, set it to `progress` and append a status-history row: `date | ready | progress | <actor> | <reason>`.
4. If blocked, record the blocker with category, owner, impact, next action and ETA, then stop.
5. Enforce WIP = 1: work on one task at a time.

## 3. Task loop

For each unchecked task in dependency order:

1. Announce the task ID and title.
2. **Red** — write the smallest failing test that expresses the next acceptance criterion; run it and confirm the expected failure.
3. **Green** — implement the minimum production code; run the focused test and the relevant suite.
4. **Refactor** — improve structure with tests green; run all affected tests.
5. Update the task row, check only the verified deliverables, and append a progress line: `YYYY-MM-DD | Tnn | files | tests | decision/ADR | next`.
6. Create an ADR with the `adr-authoring` skill if the task made an architectural or irreversible decision.
7. Update `AGENTS.md` with the `docs-sync` skill if docs, skills, commands or diagrams changed.
8. Run `scripts/harness-lint.sh` and the task-specific `quality-gates` checks.
9. If a task reveals missing work, append a new task row with the next stable `Tnn` ID and continue. Never delete or renumber existing tasks.
10. If blocked, record `blocked=true` with category, owner, impact, next action and ETA, then stop the loop.
11. If `--commit` is present, commit the task's files with a Conventional Commit and the required trailers.

## 4. Completion

Set the WP to `done` only when all of the following are true:

1. Every task row is done.
2. Every deliverable is checked and individually verified.
3. Every acceptance criterion has evidence.
4. `scripts/harness-lint.sh` is green.
5. All relevant quality gates are green.
6. No P0 or P1 is open.
7. Documentation and ADRs are updated.

Append: `date | progress | done | <actor> | all tasks, deliverables, gates and evidence verified`.

If any condition is false, leave the WP in `progress` and report the exact gap.

## 5. Commit policy

- Default: do not stage, commit, amend, push or tag.
- With `--commit`: one commit per completed task, Conventional Commits, and the trailers:
  ```
  AI-Lab-Run: <run_uuid or "n/a">
  AI-Lab-Card: <WP-XX/Tnn>
  AI-Lab-Agent: <agent identifier>
  ```
- Never commit secrets, build artifacts or changes outside the task scope.

## 6. Report

Return exactly:

1. **WP:** `WP-XX` and title.
2. **Status:** final status.
3. **Tasks:** completed task IDs and titles.
4. **Deliverables:** checked count and total count.
5. **Tests:** exact commands and results.
6. **Evidence:** links to reports, screenshots, benchmarks or logs.
7. **ADRs:** IDs created or superseded.
8. **Gates:** pass/fail per gate.
9. **Residual risks:** open blockers or follow-up tasks.
10. **Next:** the next eligible WP in index order.
