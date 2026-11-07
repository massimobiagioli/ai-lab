# WP-00 Independent Review

| Field | Value |
|---|---|
| Reviewer | review-agent (independent subagent) |
| Author | Dev (agent) |
| Date | 2026-11-07 |
| Scope | WP-00 governance, templates, linter, analytics views, CI gate mapping |
| Verdict | `APPROVED` |

The reviewer is a different actor from the author and performed a read-only review against `.docs/CONSTITUTION.md`, `.docs/PRINCIPLES.md`, `.docs/STACK.md`, `.docs/sow/SOW.md` §4–§13, `.docs/sow/WP-00-GOVERNANCE.md` and the normative technical specification.

## Acceptance criteria

| AC | Result | Evidence |
|---|---|---|
| AC1 `sow-lint` fails on status, link, dependency, checklist and stale-log violations | PASS | `.tools/sow-lint-self-test.sh` 11/11 cases |
| AC2 every WP validates against the template structure | PASS | `scripts/sow-lint.sh` exit 0, 31 checks |
| AC3 every required check name exists in the workflow and branch protection | PASS | All 16 names agree across `required-checks.txt`, `ci.yml`, `quality-gates.md` and `branch-protection.json`; applied to the remote `main` and verified with `gh api repos/massimobiagioli/ai-lab/branches/main/protection` (CR-0003). |
| AC4 metrics views return expected fixture values | PASS | `.tools/test-analytics-views.sh` 25/25 assertions |
| AC5 sample CR is submitted, approved and recorded | PASS | `.docs/product/change-requests/CR-0001.md` |
| AC6 retrospective has exactly three actions linked to TODO cards | PASS | `.docs/product/retrospectives/M0-retro.md`, `.docs/product/backlog/AIL-101.md`..`AIL-103.md` |
| AC7 separation-of-duties rejection returns `422 SEPARATION_OF_DUTIES` | PASS | `cargo test -p ailab-api --test separation_of_duties` → 2/2 passed: `author_cannot_review_own_card_and_maps_to_422` asserts the RFC 9457 payload status 422 and code `SEPARATION_OF_DUTIES`; `merge_policy_rejects_author_as_reviewer` asserts the merge violation. |

## Blocking defects

None.

## Non-blocking observations

- `.docs/product/metrics.md` documents `cards.created_at` as the lead-time origin proxy until an `ideas` table exists. Accepted.
- `.docs/product/risk-register.md` legend does not list `Critical`, used by R-08. Cosmetic.
- `analytics.autonomous_success` includes done cards without a READY event; the Kanban engine forbids that state. Edge case only.
- The schema DDL, seed characters and analytics fixtures apply cleanly on PostgreSQL 18 + pgvector.

## Reproduced evidence

- `scripts/harness-lint.sh` → 16 checks passed.
- `scripts/sow-lint.sh` → 31 checks passed.
- `.tools/sow-lint-self-test.sh` → 11 cases.
- `.tools/check-doc-links.sh` → 138 links checked.
- `.tools/test-analytics-views.sh` → 25 assertions.
- `cargo test -p ailab-api --test separation_of_duties` → 2 passed.
- `gh api repos/massimobiagioli/ai-lab/branches/main/protection` → 16 required contexts (CR-0003).

## Reviewer statement

The deliverables satisfy WP-00. All seven acceptance criteria are verified: branch protection is applied and AC7 is evidenced by an integration test. Approved for WP closure.
