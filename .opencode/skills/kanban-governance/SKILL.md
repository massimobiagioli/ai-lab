---
name: Kanban Governance
description: Use when creating, refining, moving, reviewing or closing AI-Lab cards, or when checking Definition of Ready, Definition of Done, WIP limits and separation of duties.
---

# Kanban Governance

## When to use

- A card is created, refined, assigned, moved, reviewed, blocked, reopened or closed.
- A transition is rejected and the violation must be understood.
- A WP task needs a card or a card needs evidence.

## Preconditions

- Read `.docs/sow/WP-09-KANBAN-ENGINE.md` and `.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.3.
- Know the card's column, version, assignee, reviewer and budget.
- Know the actor role and the effective autonomy level.

## Workflow

1. Confirm the requested transition is in the allowed matrix. If it is not, stop and return the exact violation.
2. Evaluate the guards:
   - Todo to Ready: DoR complete, assignee role-compatible, due date in the future, estimate in {1,2,3,5,8}, priority not `wont`, budget positive, dependencies resolved or waived.
   - `READY` to `DOING`: lease acquired by the assignee, agent active, WIP available, provider healthy, budget available.
   - `DOING` to `REVIEW`: run succeeded, PR open, branch pushed, verification green, log has summary and decisions, reviewer assigned and different from the assignee.
   - `REVIEW` to `DONE`: QA approved, reviewer different from author, CI green, PR merged, merge SHA present, DoD complete, no P0 or P1.
   - Any transition to `ICEBOX`: blocked reason with category and text.
3. Use the server transition endpoint; never write the column directly.
4. Record the immutable `card_event` and the reason.
5. Enforce WIP: one `DOING` card per agent by default and the configured review limit per QA.
6. Enforce separation of duties: reviewer is not the assignee; merger is not the author.
7. For a rejected transition, return the structured violations and do not retry with a bypass.
8. For a completed card, verify the evidence: PR, merge SHA, tests, coverage, docs, QA verdict, CV evidence.

## Checklist

- [ ] Transition exists in the matrix.
- [ ] Every guard evaluated.
- [ ] `expected_version` supplied.
- [ ] Actor authorized for the transition.
- [ ] WIP limit respected.
- [ ] Separation of duties respected.
- [ ] `card_event` written with reason.
- [ ] Evidence attached for `REVIEW` and `DONE`.
- [ ] Blocked reason complete for `ICEBOX`.
- [ ] Reopen creates a new card linked to the old one.

## Anti-patterns

- Writing `column_name` directly in the database or API.
- Moving a card to `DONE` before the merge SHA exists.
- Assigning the reviewer to the assignee.
- Treating `blocked` as a column instead of a flag with a reason.
- Creating a `READY` card from an agent without DoR.
- Bypassing WIP by creating a second `DOING` card for the same agent.

## References

- `../../../.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.4
- `../../../.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.3
- `../../../.docs/sow/WP-09-KANBAN-ENGINE.md`
- `../../../.docs/sow/WP-15-QA-MERGE.md`

## Supporting files

- None.
