# AI-Lab QA, Review & Merge Policy

| Field | Value |
|---|---|
| Document | `.docs/product/qa-merge.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | QA |
| Related | `.docs/product/kanban.md`, `.docs/product/github.md`, `.docs/product/orchestration.md` |

QA verifies every acceptance criterion with independent evidence, produces a structured verdict and the merge policy decides whether the change may land under the effective autonomy level.

## 1. Review verdicts and matrix

- Verdicts: `approved`, `changes_requested`, `blocked`.
- Acceptance status per criterion: `pass`, `fail`, `not_tested`.
- Findings: `{id, severity (P0..P3), file, line?, issue, fix}`.
- `approved` requires every criterion to be `pass` and no P0/P1/P2 finding.
- A P0 finding produces `blocked` and requires an incident/escalation before release.
- The review record carries `review_id`, `run_id`, `reviewer_id`, `head_sha`, findings and the acceptance matrix.

## 2. Merge conditions

A merge is allowed only when all conditions hold:

1. reviewer ≠ author and merger ≠ author (`SEPARATION_OF_DUTIES`),
2. QA verdict `approved` (`QA_NOT_APPROVED`),
3. CI green (`CI_NOT_GREEN`),
4. PR open, non-draft and targeting the default branch (`PR_NOT_READY`),
5. the diff touches no forbidden path and contains no secret (`FORBIDDEN_DIFF`),
6. the PR is merged and the merge SHA recorded (`PR_NOT_MERGED`, `MERGE_SHA_MISSING`),
7. DoD complete (`DOD_INCOMPLETE`),
8. no P0/P1 open (`P0_P1_OPEN`),
9. budget remaining (`BUDGET_EXHAUSTED`),
10. the head SHA equals the reviewed SHA (`STALE_REVIEW`), the branch is not protected (`PROTECTED_BRANCH`), and the autonomy level allows the merge (`HUMAN_MERGE_REQUIRED` for A0/A1).

A repeated merge of the same head is idempotent and returns success without merging twice.

## 3. Hard stops (11)

`budget_exceeded`, `secret_detected`, `forbidden_path`, `force_push`, `destructive_sql`, `dependency_risk`, `disable_checks`, `large_pull_request` (>1,500 lines or >40 files), `self_review`, `self_merge`, `prompt_injection`.

A hard stop cancels the run, moves the card to `ICEBOX` (or `DOING` with a blocker), emits `system.escalation` and requires human resolution.

## 4. Rework and autonomy

- At most three `changes_requested` rounds; the fourth moves the card to `ICEBOX` with an escalation.
- Effective autonomy is `min(lab, card, agent)`; A0/A1 require a human merge, A2/A3 may merge autonomously after the checks above.

## 5. Error codes

`SEPARATION_OF_DUTIES`, `QA_NOT_APPROVED`, `QA_BLOCKED`, `AC_NOT_PASS`, `BLOCKING_FINDING`, `CI_NOT_GREEN`, `PR_NOT_READY`, `FORBIDDEN_DIFF`, `PR_NOT_MERGED`, `MERGE_SHA_MISSING`, `DOD_INCOMPLETE`, `P0_P1_OPEN`, `BUDGET_EXHAUSTED`, `STALE_REVIEW`, `PROTECTED_BRANCH`, `HUMAN_MERGE_REQUIRED`, `AC_INCOMPLETE`, `AC_DUPLICATE`, `BLOCKED_WITHOUT_REASON`, `CHANGES_REQUESTED_WITHOUT_FINDING`, `MISSING_HEAD_SHA`.
