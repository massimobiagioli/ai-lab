---
name: WP & Task Delivery
description: Use when selecting, executing, updating or closing a work package or task from .docs/sow, including status changes, evidence and quality gates.
---

# WP & Task Delivery

## When to use

- The operator invokes `/impl-task` or `/impl-wp`.
- A work package must move from `ready` to `progress` or from `progress` to `done`.
- A task needs status, evidence, ADR or documentation updates.

Do not use this skill for a one-off explanation or for product code unrelated to a work package task.

## Preconditions

1. Read `AGENTS.md` and follow its source-of-truth order.
2. Read `.docs/CONSTITUTION.md`, `.docs/PRINCIPLES.md`, `.docs/STACK.md`.
3. Read `.docs/sow/SOW.md` and the target `WP-XX-*.md`.
4. Read `.docs/adr/registry.md` and any ADR referenced by the WP.
5. Confirm every dependency WP is `done` or has a recorded waiver.

## Selection algorithm

1. If the argument names a task (`WP-XX/Tnn`), select exactly that task.
2. Else if the argument names a WP (`WP-XX`), select its first unchecked task in dependency order.
3. Else select the first WP in index order with status `ready` or `progress`, then its first unchecked task in dependency order.
4. If no eligible task exists, report that the backlog is complete and stop.

## Workflow

1. Set the WP status to `progress` if it is `ready`; append a status-history row with date, from, to, actor and reason.
2. Load the specialist skill for the task: `tdd-rust`, `tdd-angular`, `security-first`, `kanban-governance`, `api-contracts`, `data-migrations`, `agent-prompts`, `docker-operations`, `observability-sre`, `quality-gates`, `release-management` or `docs-sync`.
3. Execute the TDD protocol:
   - Red: write the smallest failing test for the next acceptance criterion; run it and confirm the expected failure.
   - Green: implement the minimum production code; run the test and the relevant suite.
   - Refactor: improve structure with tests green; run the affected suite again.
4. Update only the task row and the specific deliverables that are verified. Never bulk-check deliverables.
5. Append one progress-log line: `YYYY-MM-DD | Tnn | files | tests | decision/ADR | next`.
6. If a decision was made, create an ADR with `adr-authoring` and update `.docs/adr/registry.md`.
7. If docs, skills or commands changed, update the indexes in `AGENTS.md` with `docs-sync`.
8. Run `scripts/harness-lint.sh` and the task-specific quality gates from `quality-gates`.
9. If this was the last unchecked task and all deliverables and gates pass, set the WP to `done`; otherwise leave it `progress`.
10. Report: task, files, tests with exact commands and results, ADR IDs, status, next task.

## Checklist

- [ ] Source-of-truth files read.
- [ ] Dependencies satisfied or waived.
- [ ] WP status and history updated.
- [ ] Failing test written and observed before production code.
- [ ] Test passes after implementation.
- [ ] Refactor completed with tests green.
- [ ] Acceptance criteria evidenced.
- [ ] Deliverables updated individually.
- [ ] Progress log appended.
- [ ] ADR created and registered if needed.
- [ ] `AGENTS.md` indexes updated if docs/skills changed.
- [ ] `scripts/harness-lint.sh` green.
- [ ] Quality gates green.
- [ ] Final report produced.

## Anti-patterns

- Marking a task done without a failing test first.
- Checking multiple deliverables with one command.
- Writing product code unrelated to the selected task.
- Changing a WP status without evidence.
- Skipping `AGENTS.md` updates after creating docs or skills.
- Asking the operator a question instead of recording a blocker and stopping.

## References

- `../../../.docs/sow/SOW.md`
- `../../../.docs/sow/WP-00-GOVERNANCE.md`
- `../../../.docs/CONSTITUTION.md`
- `../../../.docs/PRINCIPLES.md`
- `../../../.docs/adr/registry.md`

## Supporting files

- None.
