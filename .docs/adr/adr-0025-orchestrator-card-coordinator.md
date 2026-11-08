# ADR-0025 — Orchestrator as the delivery-card coordinator

| Field | Value |
|---|---|
| Status | Accepted |
| Date | 2026-11-07 |
| Deciders | Tech Lead, Backend |
| Supersedes | — |
| Superseded by | — |
| Related | ADR-0002, ADR-0004, ADR-0009, ADR-0023 |

## Context

The worker executes a run inside the delivery context, while the card lifecycle
(`cards.column_name`, `version`, `merge_sha`, `blocked_reason`) is owned by the
Kanban context. Claiming a card and creating its run lease must be one atomic
transaction, otherwise two workers can dispatch the same card or a crashed
worker can strand it. A network call to a Kanban API from inside the run
transaction would break that atomicity and add latency to every dispatch.

## Decision

The delivery orchestrator is the **coordinator** for the specific card fields
that the run lifecycle mutates:

- `READY → DOING` when a run is claimed.
- `DOING → READY` when a stale run is reaped or a retry is requeued.
- `DOING → ICEBOX` with a `risk` blocked reason when retries are exhausted.
- `READY/DOING/REVIEW → DONE` with the SCM `merge_sha` after the merge policy
  (`evaluate_pre_merge`) has accepted the run.

These updates are executed as SQL inside the same transaction as the run lease.
The scheduler enforces eligibility (`READY`, assignee, unblocked, budget,
paused lab, retry limit, concurrency) in the `WHERE` clause. The DONE update
requires a non-null merge SHA and bumps `version`; the domain Kanban guards
remain authoritative for every transition performed through the API/UI.

The architecture fitness check maps `crates/orchestrator/src/postgres.rs` and
the API Postgres store to the explicit `coordinator` context, so the exception
is visible and cannot silently expand.

## Consequences

- Dispatch and card transitions are atomic and crash-safe.
- The Kanban context stays the owner of the card model; this ADR is the only
  sanctioned cross-context write path.
- Any new card field or transition touched by the coordinator requires an ADR
  update and a corresponding fitness mapping.
- Auditability is preserved through `delivery.run.*` outbox events.

## Alternatives considered

- **Kanban RPC/outbox saga**: rejected because it cannot share the run lease
  transaction and adds a two-phase flow to every dispatch.
- **Worker-side polling with a separate transaction**: rejected because a crash
  between the two transactions can strand the card.

## Compliance

This ADR documents the ADR-0002 exception and the ADR-0009 gate application for
the DONE path (`evaluate_pre_merge` runs before `complete_success`).
