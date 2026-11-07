# AI-Lab Status Model, Definition of Ready and Definition of Done

| Field | Value |
|---|---|
| Document | `docs/status-model.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Tech Lead |
| Related | `docs/governance.md`, `docs/quality-gates.md`, `docs/change-control.md` |
| Enforcement | `scripts/sow-lint.sh`, CI job `sow-lint`, Kanban engine guards |

Only three statuses exist for work packages and cards: `ready`, `progress`, `done`. `blocked` is a flag, never a status. Any other value is a defect.

---

## 1. Work package statuses

| Status | Definition | Entry criteria | Exit criteria |
|---|---|---|---|
| `ready` | Scoped, estimated, dependencies defined, not started | Objective, scope, deliverables, tasks, acceptance criteria and DoD complete; dependencies `done` or explicitly waived | Status moved to `progress`, owner assigned, WIP slot acquired, first deliverable started |
| `progress` | Active work with at least one deliverable in progress | Status updated; owner assigned; WIP slot acquired; progress log opened | Every deliverable checked; acceptance evidence linked; WP DoD satisfied; Tech Lead + QA sign-off |
| `done` | Fully delivered and verified | All deliverables checked; acceptance tests pass; docs updated; no open P0/P1; change record closed | Reopened only through an approved CR; status becomes `progress` with a new version |

Transitions are `ready → progress → done`. Skipping is forbidden. Reopening `done → progress` requires an approved CR (`.docs/sow/SOW.md` ST-02, §12).

## 2. Card statuses (Kanban columns)

Cards use the six columns `TODO`, `READY`, `DOING`, `REVIEW`, `DONE`, `ICEBOX`. Each column has entry invariants and exit rules defined in `.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.3. The card's `column` is a projection of the latest immutable `card_event`; the UI is never the authority.

Same-column transitions are always denied. Invalid transitions return `422 INVALID_TRANSITION` with a machine-readable `violations[]` array. The UI surfaces the first violation verbatim.

## 3. Blocked flag

A blocked item keeps its current status and records:

| Field | Required content |
|---|---|
| `blocked` | `true` |
| `category` | `dependency`, `clarification`, `external`, `risk`, `budget` or `policy` |
| `owner` | Named role or agent accountable for unblocking |
| `impact` | What cannot proceed and which deliverables are affected |
| `next_action` | The concrete action that removes the blocker |
| `eta` | UTC date or `unknown` with the reason |

`blocked=false` is recorded when the blocker is resolved, with the resolution text. The resolution is append-only.

## 4. Status history

Status changes are append-only:

```
YYYY-MM-DD | from | to | actor | reason
```

A status change without an evidence link is a defect. The SOW index status and the WP file status MUST match; `scripts/sow-lint.sh` fails on drift.

## 5. Card Definition of Ready

A card is `READY` if and only if all of the following hold:

1. Canonical user story complete: persona, capability, outcome and context.
2. At least one acceptance criterion in Given/When/Then form, each testable.
3. At least one concrete deliverable: a path or named artifact with a completion definition.
4. Estimate in `{1,2,3,5,8}`; priority in `{must,should,could}`; `wont` is not allowed in `READY`.
5. Assignee set and role-compatible; due date set and in the future.
6. Dependencies listed and each resolved or explicitly waived by the Tech Lead with a reason.
7. WP is `progress`; card budget greater than zero; source recorded (`human` or `agent`).
8. `needs_refinement = false`.

Cards estimated at 13 MUST be split before `READY`. Cards above 8 points SHOULD be split.

## 6. Card Definition of Done

A card is `DONE` if and only if all of the following hold:

1. PR merged to the project default branch; merge SHA stored on the card.
2. All required CI checks green; coverage for new code at least 80% or an approved waiver.
3. Every acceptance criterion evidenced by a test, command output, screenshot, benchmark or run ID.
4. Documentation updated (README, ADR, API or runbook as applicable); changelog entry when user-visible.
5. No P0/P1 defects; security scans clean; no secret in the diff.
6. QA verdict `APPROVED` for behavior-changing cards; reviewer is not the author.
7. Card log contains the final summary, decisions and residual risks.
8. CV evidence appended by the reflection job.

## 7. Work package Definition of Ready

A WP may enter `progress` only when:

1. Its file contains objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks and references.
2. Every dependency is `done` or has a recorded waiver with category, owner, impact, next action and ETA.
3. The owner is assigned and a WIP slot exists (program WIP ≤ 2 in `progress`).
4. Acceptance criteria are testable and evidence types are named.
5. Required ADRs for the WP's decisions exist or are scheduled.

## 8. Work package Definition of Done

A WP is `done` only when:

1. Every task row is done and every deliverable is checked and individually verified.
2. Every acceptance criterion has linked evidence.
3. `scripts/harness-lint.sh` and `scripts/sow-lint.sh` are green.
4. All relevant quality gates are green.
5. No P0/P1 is open and no blocker is unresolved.
6. Documentation and ADRs are updated.
7. QA verdict is recorded and the reviewer is not the author.
8. A status-history row records `progress → done` with the evidence reference.

## 9. Evidence rules

A `done` claim MUST link to at least one of:

- a merged PR with its merge SHA;
- a test report with the exact command and result;
- a screenshot or GIF generated from the deterministic seed;
- a benchmark or load result with thresholds;
- a security scan report;
- an ADR or CR identifier;
- a run ID with its trace.

"It works" is not evidence. Self-reported success is not evidence.

## 10. Separation of duties

Reviewer ≠ author and merger ≠ author, for humans and agents. A conflict returns `422 SEPARATION_OF_DUTIES`. Human overrides are recorded decisions, never silent exceptions (ADR-0011).

## 11. Worked example

A card `AIL-42` moves `TODO → READY` only after DoR is complete. The transition writes:

```
2026-11-07 | TODO | READY | tech-lead | DoR complete; dependencies resolved
```

If the developer is blocked by a missing provider credential, the card stays `READY` and gains `blocked=true`, `category=external`, `owner=Human Operator`, `next_action=provide DeepSeek key`, `eta=2026-11-08`. The status does not change.

## 12. CI enforcement

`scripts/sow-lint.sh` fails when:

- a WP status is not one of the three allowed values;
- the index status and the WP file status disagree;
- a WP link is broken;
- a dependency is missing or the dependency graph is cyclic;
- a `done` WP has an unchecked deliverable;
- a `progress` WP has no progress-log entry within three working days;
- a CR referenced by a WP is missing or incomplete;
- a `done` WP lacks a sign-off row.

## 13. References

- `.docs/sow/SOW.md` §6
- `.docs/sow/WP-00-GOVERNANCE.md` §3.5–§3.6
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.4
- `.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.3
- `.docs/CONSTITUTION.md` §8
- `.docs/PRINCIPLES.md` §1, §4
