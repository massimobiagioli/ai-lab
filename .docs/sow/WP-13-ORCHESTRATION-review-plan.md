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
- [x] 13-stage pipeline, context builder, loop bounds, verification/review rework, handoff, recovery, cancellation checkpoints, escalation and step trace — evidence: `crates/orchestrator/src/{pipeline,context}.rs`. Real LLM/sandbox/SCM wiring and APIs/events remain adapter work (WP-13 follow-up/WP-16).

## 3. Deliverables

- [x] Scheduler eligibility, leases, heartbeats — in-memory; Postgres lease and scheduler loop deferred.
- [x] Run state machine, attempts/retries/backoff/dead letter — state machine + retry classification; dead-letter persistence deferred.
- [x] Pipeline stages 1–13 with budgets — `RunPipeline`.
- [x] Context builder with token budget/citations — `ContextBundle` builder.
- [x] Agent loop bounds and step recording + wired executor — `LoopLimits`, `RunStep`, `WiredExecutor` (LLM/sandbox/SCM); tool authorization is WP-14.
- [x] Verification/review feedback loop (≤ 3) and wired verification — `fail_verification`, `request_changes`, sandbox exec in `WiredExecutor`.
- [x] Commit/push/PR via the SCM port + idempotency keys and handoff — `WiredExecutor`, `idempotency_key`, `Handoff`.
- [x] Handoff/review/merge orchestration — `Handoff`, pipeline stages.
- [x] Crash recovery/reconciliation — `reconcile` + `ExternalPrState::recovery`.
- [x] Cancellation/pause — `request_cancel`, checkpoints; sandbox destruction is WP-14.
- [x] Escalation ladder + human intervention records — `escalate`, `HumanDecision`.
- [x] Run events mapping + trace/step recording — `PipelineStage::event`, `RunStep`.
- [x] Integration/idempotency/bounds/wiring tests — 43 orchestrator tests including the FakeLlm/FakeScm E2E; load/chaos in WP-21.
- [x] `.docs/product/orchestration.md` — present, but it documents the deferrals.

## 4. Acceptance criteria

- [x] AC1 one worker leases a card — in-memory `LeaseManager` test.
- [x] AC2 45 s heartbeat timeout requeues with backoff — `reconcile` + `retry_decision`.
- [x] AC3 agent loop stops at 40 tool calls / 25 iterations / 45 min — `RunPipeline` bounds.
- [x] AC4 failed verification loops ≤ 3 — `fail_verification`.
- [x] AC5 exactly one PR/review/merge; retries dedupe — `WiredExecutor` opens one PR, one review and one merge; idempotency keys + handoff; adapter dedupe WP-12/WP-16.
- [x] AC6 worker kill recovers from external PR state — `ExternalPrState::recovery`.
- [x] AC7 budget exhaustion stops with `BUDGET_EXCEEDED` and pauses the lab — `RunBudget::charge` typed code + `lab_paused`; test.
- [x] AC8 cancellation stops at a tool boundary — `request_cancel` + checkpoint.
- [x] AC9 DONE only after QA/CI/merge/DoD — Kanban domain (WP-09).
- [x] AC10 complete step trace + trace_id — `RunStep` + `trace_id`.
- [x] AC11 `FakeLlm` + `FakeScm` E2E — `WiredExecutor` drives all stages through the ports; deterministic E2E asserts one branch/commit/PR/merge.

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

- [x] QA subagent verdict: `CHANGES_REQUESTED` (HEAD `a18724a`).
- [x] QA evidence: harness 16, sow-lint 31, doc links 146, fmt/clippy clean, 43 orchestrator tests + 1 ignored.
- [x] Blocking defects from QA: Postgres `SKIP LOCKED` leases/dispatch not implemented; worker does not apply 45 s stale/backoff; no iterative agent loop or ordered verification runner with raw-output rework; context builder lacks repo map/memory citations/files/conventions/feedback; SCM commit/PR/review/merge are not idempotent on retry; run APIs are an in-memory store disconnected from the worker; no event emission/metrics/OTel spans; no card DoD/transition integration or recovery/cancel wiring.
- [x] Author fixes: 13-stage pipeline, context bundle, loop bounds, verification/review rework, handoff, recovery, cancellation, escalation, step trace, event mapping and the worker tick with retry/backoff delivered.

## 9. Deferred items and residual risks

- [x] Worker scheduler loop (`Worker::tick`) delivered; Postgres leases remain — owner: Dev, WP-13 follow-up.
- [x] Real LLM/sandbox/SCM adapters for the pipeline — owner: Dev, WP-13 follow-up/WP-14.
- [x] Event persistence, metrics/traces, full E2E — owner: Dev/QA, WP-20/WP-21.
- [ ] Merge only after QA returns `APPROVED`.

## 10. Decisions

- [ ] No ADR needed for the current subset; a recorded blocker is added to the WP status history.
