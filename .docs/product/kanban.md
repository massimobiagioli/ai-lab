# AI-Lab Kanban

| Field | Value |
|---|---|
| Document | `.docs/product/kanban.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/inventory.md`, `.docs/product/eventing.md`, `crates/domain/src/kanban.rs` |

The Kanban engine is server-side. The board and the card aggregate are the truth; the UI is a projection. This document records the transition matrix, guards, DoR/DoD and error codes.

## 1. Transition matrix

**A** = allowed, **—** = denied (`INVALID_TRANSITION`). Same-column moves are always denied.

| From \ To | TODO | READY | DOING | REVIEW | DONE | ICEBOX |
|---|---|---|---|---|---|---|
| **TODO** | — | A | — | — | — | A |
| **READY** | A | — | A | — | — | A |
| **DOING** | — | A | — | A | — | A |
| **REVIEW** | — | — | A | — | A | A |
| **DONE** | — | — | — | — | — | — |
| **ICEBOX** | A | — | — | — | — | — |

`DONE → TODO` is not a transition: reopening creates a **new** card in `TODO` with `parent_id` pointing to the old card. The 36 ordered pairs are tested exhaustively; exactly 12 are allowed.

## 2. Guards

| Transition | Guards |
|---|---|
| `TODO → READY` | DoR complete: story with ≥1 AC and ≥1 deliverable, not `needs_refinement`, estimate, priority ≠ `wont`, assignee, future due date, budget > 0 |
| `TODO → ICEBOX` | Blocked reason (category + text) |
| `READY → DOING` | Assignee set; `DOING` WIP limit not exceeded |
| `READY → TODO` | Actor is Tech Lead; reason; sets `needs_refinement` |
| `READY → ICEBOX` | Blocked reason |
| `DOING → REVIEW` | Reviewer set and ≠ assignee; PR URL; green CI; `REVIEW` WIP limit not exceeded |
| `DOING → READY` | Actor is Tech Lead or assignee; run cancelled; reason |
| `DOING → ICEBOX` | Blocked reason |
| `REVIEW → DONE` | QA approved; reviewer ≠ author; green CI; PR merged with merge SHA; DoD complete |
| `REVIEW → DOING` | Changes requested |
| `REVIEW → ICEBOX` | Blocked reason |
| `ICEBOX → TODO` | Resolution text; clears the blocked reason; sets `needs_refinement` |

Error codes: `INVALID_TRANSITION` (422), `VALIDATION_FAILED` (422), `SEPARATION_OF_DUTIES` (422), `WIP_LIMIT_EXCEEDED` (422), `VERSION_CONFLICT` (409).

## 3. Definition of Ready / Done

The engine evaluates every rule and reports stable code + field violations.

- **DoR codes:** `DOR_AC_MISSING` (`story.acceptance_criteria`), `DOR_DELIVERABLE_MISSING` (`story.deliverables`), `DOR_NEEDS_REFINEMENT` (`needs_refinement`), `DOR_ESTIMATE_INVALID` (`estimate`), `DOR_PRIORITY_INVALID` (`priority`), `DOR_ASSIGNEE_MISSING` (`assignee_id`), `DOR_DUE_DATE_INVALID` (`due_date`), `DOR_BUDGET_INVALID` (`budget_microusd`). `DOR_STORY_INCOMPLETE` and `DOR_DEPENDENCY_OPEN` depend on the story/dependency model owned by WP-10/WP-16.
- **DoD codes:** `DOD_REVIEWER_CONFLICT` (`reviewer_id`), `DOD_MERGE_MISSING` (`merge_sha`), `DOD_CI_NOT_GREEN` (`ci`), `DOD_QA_NOT_APPROVED` (`qa`), `DOD_AC_EVIDENCE_MISSING` (`evidence`), `DOD_TESTS_MISSING` (`tests`), `DOD_DOCS_MISSING` (`docs`), `DOD_DEFECT_OPEN` (`defects`), `DOD_LOG_MISSING` (`log`).

## 4. Events and immutability

Every transition appends one immutable `DomainEvent` (`kanban.card.moved`) with card id, key, from, to, actor and reason. Creation emits `kanban.card.created`. The `card.column` is a projection of the latest event; the SQL/API layer owns persistence and projection in WP-16.

## 5. Tests

`scripts/run-rust.sh cargo test -p ailab-domain` covers:

- the 36 ordered column pairs (12 allowed, 24 denied, same-column denied);
- DoR and DoD guards;
- separation of duties (`attach_pr` with reviewer = assignee);
- `DOING` WIP limit;
- ICEBOX reason and resolution;
- reopen with a parent link;
- event emission for create and move.

## 6. Deferred to consumer WPs

- Board aggregate and SQL repositories: WP-16 with the API repositories.
- Concurrent card-key allocation (100 cards) and concurrent transitions (409): WP-16 integration tests.
- Kanban API, DTOs, OpenAPI and projections: WP-16.
- Overdue notification job: WP-20/WP-23.
- Board load (500 cards) and move p95 budgets: WP-21.
- Board UI: WP-19.

## 7. References

- `.docs/sow/WP-09-KANBAN-ENGINE.md`
- `.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.3
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.4
- `crates/domain/src/kanban.rs`
