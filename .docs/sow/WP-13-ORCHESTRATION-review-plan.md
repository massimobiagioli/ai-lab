# WP-13 — Orchestration, Scheduler & Run Pipeline Review Plan

| Field | Value |
|---|---|
| WP | `WP-13` |
| Contract | `.docs/sow/WP-13-ORCHESTRATION.md` |
| Review branch | `review/wp-13-orchestration` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_review` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present.
- [x] Requirement traceability maps to spec §8/§9/§18, Appendix B §B.2 and Appendix D §D.1 (M5).

## 2. Specification conformance

- [x] Scheduler/lease/run state/retry/budget/reconcile — evidence: `crates/orchestrator/src/orchestration.rs`.
- [ ] Full 13-stage pipeline, context builder, agent loop, verification loop, PR orchestration, handoff/review/merge, cancellation, escalation, run APIs/events/metrics — **NOT IMPLEMENTED**; the product doc defers them to a WP-13 follow-up.

## 3. Deliverables

- [x] Scheduler eligibility, leases, heartbeats — in-memory only; Postgres lease and scheduler loop missing.
- [x] Run state machine, attempts/retries/backoff/dead letter — state machine and retry classification present; dead-letter persistence missing.
- [ ] Pipeline stages 1–13 with budgets — missing.
- [ ] Context builder with token budget/citations — missing.
- [ ] Agent loop with tool authorization, step recording and bounds — missing.
- [ ] Verification runner + feedback loop — missing.
- [ ] Commit/push/PR orchestration + idempotency — missing (only the WP-12 adapter exists).
- [ ] Handoff/review/merge orchestration — missing.
- [x] Crash recovery and reconciliation — `reconcile` marks stale runs timed out; external-state reconciliation missing.
- [x] Cancellation/pause — `Run::cancel`; sandbox destruction missing.
- [ ] Escalation ladder + human intervention records — missing.
- [ ] Run APIs + events + metrics/traces — missing.
- [ ] Integration/chaos/idempotency suites — missing.
- [x] `.docs/product/orchestration.md` — present, but it documents the deferrals.

## 4. Acceptance criteria

- [x] AC1 one worker leases a card — in-memory `LeaseManager` test; Postgres/20-worker test missing.
- [ ] AC2 45 s heartbeat timeout requeues with backoff — `reconcile` times out; requeue missing.
- [ ] AC3 agent loop stops at 40 tool calls / 25 iterations / 45 min — not implemented.
- [ ] AC4 failed verification loops ≤ 3 — not implemented.
- [ ] AC5 exactly one PR/review/merge; retries dedupe — not implemented.
- [ ] AC6 worker kill recovers from external PR state — not implemented.
- [x] AC7 budget exhaustion stops with `BUDGET_EXCEEDED` — `RunBudget`; lab pause missing.
- [ ] AC8 cancellation stops at a tool boundary and destroys the sandbox — `Run::cancel` only.
- [x] AC9 DONE only after QA/CI/merge/DoD — enforced in the Kanban domain (WP-09), not wired here.
- [ ] AC10 complete step trace + trace_id across LLM/SCM/sandbox — not implemented.
- [ ] AC11 `FakeLlm` + `FakeScm` E2E — not implemented.

## 5. Implementation and tests

- [x] Eligibility, lease contention/expiry/reap, run transitions, retry classification, budget, reconcile — 12 tests.
- [ ] Pipeline/agent loop/verification/idempotency/recovery/E2E — missing.
- [x] No network in tests.

## 6. Security and quality gates

- [x] Lease contention prevents duplicate execution.
- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p ailab-orchestrator` green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/orchestration.md` documents the implemented subset and deferrals.
- [x] `AGENTS.md` indexes updated for this review plan.
- [ ] WP-13 status must not remain `done` until the pipeline and blocking ACs are delivered.

## 8. QA review

- [ ] QA subagent verdict: `PENDING`.
- [x] Blocking gaps identified by the author: pipeline stages, context builder, agent loop, verification loop, PR orchestration, handoff/review/merge, cancellation to sandbox, escalation, run APIs/events/metrics, E2E; plus AC2, AC3, AC4, AC5, AC6, AC8, AC10, AC11.

## 9. Deferred items and residual risks

- [ ] Postgres leases and the worker scheduler loop — owner: Dev, WP-13 follow-up.
- [ ] Full pipeline/agent loop/verification/PR/handoff — owner: Dev, WP-13 follow-up.
- [ ] Event emission, metrics/traces, E2E — owner: Dev/QA, WP-13 follow-up + WP-20/WP-21.
- [ ] No merge until every blocking AC is delivered and QA returns `APPROVED`.

## 10. Decisions

- [ ] No ADR needed for the current subset; a recorded blocker is added to the WP status history.
