# WP-09 — Agile Kanban Engine Review Plan

| Field | Value |
|---|---|
| WP | `WP-09` |
| Contract | `.docs/sow/WP-09-KANBAN-ENGINE.md` |
| Review branch | `review/wp-09-kanban-engine` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_review` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present and consistent.
- [x] Requirement traceability maps to spec §6.4/§7.2, Appendix C §C.3 and Appendix A §7.

## 2. Specification conformance

- [x] Specification §6.4 Kanban and §7.2 delivery — evidence: `crates/domain/src/kanban.rs`, `.docs/product/kanban.md`.
- [x] Appendix C §C.3 transition matrix and codes — evidence: `kanban.rs`, `kanban.md`.
- [x] Appendix A §7 card/board schema — evidence: `migrations/0001_init.sql`.

## 3. Deliverables

- [x] Card aggregate with invariants; one board per project — evidence: `Card`, `Board` unique constraint.
- [x] Project card sequence in WP-07; 100-card concurrency in WP-16 — evidence: `Project::next_card_seq`; DB unique keys.
- [x] Transition engine + 36-case matrix + guards — evidence: `is_allowed`, `transition_matrix_has_exactly_twelve_allowed_moves`.
- [x] DoR/DoD guards with stable codes + fields — evidence: `dor_violations`/`dod_violations`, `DomainError::Violation`, tests.
- [x] WIP limits, due dates, blocked reasons — evidence: `guard_doing`, `guard_review`, `BlockedReason`.
- [x] Reviewer = assignee rejected — evidence: `attach_pr`, `guard_review`, `guard_done` → `SEPARATION_OF_DUTIES`.
- [x] Immutable move/create events — evidence: `DomainEvent` push on create/move.
- [x] Reopen creates a TODO card with a parent link — evidence: `Card::reopen`.
- [x] Events delivered; API/DTO WP-16 — evidence: `kanban.card.created|moved`.
- [x] Matrix and guard tests — evidence: `cargo test -p ailab-domain`.
- [x] `.docs/product/kanban.md` — evidence: file.

## 4. Acceptance criteria

- [x] AC1 all 36 ordered transitions behave as specified; denied → `INVALID_TRANSITION` — evidence: matrix test (12 allowed / 24 denied).
- [x] AC2 every DoR/DoD check has an exact code and field — evidence: `dor_violations`/`dod_violations` + per-code tests; `DOR_STORY_INCOMPLETE`/`DOR_DEPENDENCY_OPEN` deferred to WP-10/WP-16 (story/dependency model).
- [x] AC3 100 concurrent cards unique/contiguous — DEFERRED to WP-16 (DB sequence); WP-07 allocator delivered.
- [x] AC4 two concurrent transitions → one success, one `VERSION_CONFLICT` — evidence: constraint suite + DB version guard; HTTP mapping WP-16.
- [x] AC5 READY requires assignee/future due date/estimate in {1,2,3,5,8}/priority/story — evidence: `dor_violations`; estimate 13 rejected with `DOR_ESTIMATE_INVALID`.
- [x] AC6 DONE requires merge SHA, QA, CI, DoD — evidence: `dod_violations`.
- [x] AC7 reviewer = assignee / merger = author rejected — evidence: `SEPARATION_OF_DUTIES` tests; merge policy in `qa.rs`.
- [x] AC8 WIP limits enforced for DOING/REVIEW — evidence: `WIP_LIMIT_EXCEEDED`.
- [x] AC9 one immutable event per transition; projection matches — evidence: event push; SQL projection WP-16.
- [x] AC10 board/move performance — DEFERRED to WP-21.

## 5. Implementation and tests

- [x] Transition matrix, guards, DoR/DoD codes, WIP, separation, ICEBOX, reopen — tests.
- [x] Per-code DoR/DoD tests.
- [x] No real external API in tests.
- [x] Coverage thresholds met (domain crate).

## 6. Security and quality gates

- [x] Separation-of-duties enforced; no direct column writes.
- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p ailab-domain` green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/kanban.md` updated with DoR/DoD codes.
- [x] `.docs/adr/registry.md` consistent (ADR-0009).
- [x] `AGENTS.md` indexes updated for this review plan.
- [x] `.docs/sow/SOW.md` and WP-09 status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED`.
- [ ] QA evidence: reproduced gates (harness 16, sow-lint 31, doc links 143, 55 domain tests, clippy, per-code DoR/DoD).
- [x] First QA pass returned `CHANGES_REQUESTED` for estimate 13 bypassing DoR; fixed by validating the allowed estimate set and adding a regression test, plus a REVIEW WIP test.

## 9. Deferred items and residual risks

- [x] AC3 100-card concurrency and AC4 HTTP conflict mapping — owner: Dev, WP-16.
- [x] `DOR_STORY_INCOMPLETE`/`DOR_DEPENDENCY_OPEN` — owner: Dev, WP-10/WP-16 (story/dependency model).
- [x] AC10 performance — owner: QA, WP-21.
- [x] No unexplained deferral remains.

## 10. Decisions

- [x] ADR-0009 server-enforced Kanban invariants recorded.
