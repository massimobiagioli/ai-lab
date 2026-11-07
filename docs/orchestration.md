# AI-Lab Orchestration

| Field | Value |
|---|---|
| Document | `docs/orchestration.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `docs/kanban.md`, `docs/sandbox.md`, `docs/agents.md`, `crates/orchestrator/src/orchestration.rs` |

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

## 8. Deferred to the WP-13 follow-up

- Postgres-backed leases/heartbeats and the scheduler loop.
- The full agent pipeline (plan → tools → tests → PR → review).
- Worker concurrency, dead-letter persistence, event emission and metrics.
- Integration/chaos suites in WP-21.

## 9. References

- `.docs/sow/WP-13-ORCHESTRATION.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §8, §18
- `crates/orchestrator/src/orchestration.rs`
