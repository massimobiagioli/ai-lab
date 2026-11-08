# AI-Lab Orchestration

| Field | Value |
|---|---|
| Document | `.docs/product/orchestration.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/kanban.md`, `.docs/product/sandbox.md`, `.docs/product/agents.md`, `crates/orchestrator/src/orchestration.rs` |

The scheduler selects `READY` cards, acquires a lease, runs the agent pipeline and recovers safely after crashes.

## 1. Eligibility

A card is eligible when it is `READY`, the agent is active, the budget is available, the WIP limit is free and no active lease exists. Every failed condition is reported.

## 2. Leases

- One lease per card; the second worker gets `Conflict`.
- TTL 900 s, heartbeat 15 s.
- `renew` extends the lease; `release` frees it; `reap` removes expired leases.
- A lease is the only way to start a run.

## 3. Run state machine

`Queued → Running → Succeeded | Failed | TimedOut`; `Queued|Running → Cancelled`. Invalid transitions are rejected.

## 4. Retries and dead-letter

- Transient errors retry up to 3 attempts.
- Backoff: 100 ms, 200 ms, capped at 5 s.
- Permanent and policy errors, or a third failure, dead-letter the card to `ICEBOX` for review.

## 5. Budget

`RunBudget` stops a run before it exceeds its limit. The caller records the cost and pauses the lab on exhaustion.

## 6. Recovery

`reconcile` marks running runs whose heartbeat is older than the TTL as `TimedOut`, so the scheduler can requeue or dead-letter them.

## 7. Tests

`cargo test -p ailab-orchestrator` covers eligibility reasons, lease contention, expiry/reap, run transitions, retry classification, budget hard stop and reconciliation.

## 8. Pipeline, loops and recovery

`crates/orchestrator/src/pipeline.rs` implements the 13 normative stages (`lease → context → plan → implement → verify → commit → push → pr → transition → review → merge → done → reflect`) with:

- per-stage and total wall-clock budgets, checked at every checkpoint;
- loop bounds (40 tool calls, 25 iterations) and bounded verification (3) and review (3) reworks;
- cancellation checkpoints between tool calls;
- a step trace (`RunStep` with args hash, duration, exit status and output ≤ 20 KB) and one `trace_id`;
- the structured `Handoff` object, escalation (last 20 steps + cost) and human-decision records;
- idempotency keys (`run_id:operation:target`) and external-PR recovery (`merged → done`, `open → resume review`, `branch → resume push`, none → retry);
- catalog event mapping (`delivery.run.started|step|completed`).

`crates/orchestrator/src/context.rs` builds a token-budgeted context bundle: the task packet is never dropped, high-priority overflow is summarized and low-priority overflow is omitted, deterministically.

## 9. Deferred to the WP-13 follow-up

- Postgres-backed leases/heartbeats and the worker scheduler loop.
- Real adapters for the pipeline stages (LLM/sandbox/SCM) and event emission.
- Worker concurrency, dead-letter persistence and metrics/traces.
- Integration/chaos suites in WP-21.

## 10. References

- `.docs/sow/WP-13-ORCHESTRATION.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §8, §18
- `crates/orchestrator/src/orchestration.rs`
