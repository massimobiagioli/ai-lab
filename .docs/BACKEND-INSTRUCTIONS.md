# AI-Lab Backend Instructions

| Field | Value |
|---|---|
| Document | `.docs/BACKEND-INSTRUCTIONS.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Applies to | Every Rust service, crate, adapter, worker, API, CLI and migration |
| Source of truth | `.docs/technical-specification/` and `.docs/sow/` |

These rules are binding for all backend work. They refine the technical specification and the work packages; they never override them. If a rule conflicts with the specification, the specification wins and this document is corrected through an ADR.

---

## 1. Language and toolchain

1. All code, comments, commit messages, logs, errors and documentation are in English.
2. Rust is pinned to `1.99.1`, edition 2024, through `rust-toolchain.toml`. Do not use a different toolchain locally or in CI.
3. Every first-party crate starts with `#![forbid(unsafe_code)]`.
4. Format with `cargo fmt`. Lint with `cargo clippy --workspace --all-targets -- -D warnings`. Warnings are errors.
5. Dependencies are pinned in `Cargo.lock`. Do not add a dependency without checking the license policy in `deny.toml` and recording an ADR if it changes the stack.
6. SQL is compile-time checked with `sqlx::query!` or `query_as!`. Commit `.sqlx` metadata when queries change.
7. Release profile uses `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"` and `panic = "unwind"`. `panic = "abort"` is forbidden.

---

## 2. Architecture and boundaries

1. Follow the dependency rule: `domain` depends on nothing but the standard library and small value crates; `application` depends on `domain`; adapters implement ports; `api` and `worker` are composition roots.
2. Domain and application code MUST NOT depend on `sqlx`, `axum`, `redis`, `reqwest`, `bollard` or any transport type. Adapter types never leak inward.
3. Use the five bounded contexts: Inventory, Lab, Knowledge/Memory, Kanban, Delivery, plus Platform. Each context owns its tables. A context MUST NOT read or write another context's tables; use a query port or a read-only view owned by Platform.
4. Business rules live in aggregates and domain services, not in handlers, repositories or SQL triggers.
5. Cross-context side effects use domain events and the transactional outbox. Do not call another context's repository or write its tables.
6. `api` and `worker` coordinate only through Postgres state and the outbox, plus Redis for ephemeral fan-out. Do not add direct `api` to `worker` HTTP calls.
7. Any architectural change, context extraction or new runtime service requires an ADR and a contract-compatibility test.

---

## 3. Rust code rules

1. Model identifiers as newtypes over `Uuid`; generate UUIDv7 application-side. Never let the database generate domain IDs.
2. Use validated constructors for slugs, card keys, money, versions, priorities, estimates, columns and actor identities. Invalid values must be unrepresentable.
3. Timestamps are `DateTime<Utc>` and `TIMESTAMPTZ`. Never use local time or naive timestamps.
4. Money is `i64` micro-USD. Never use floating point for money. Use checked arithmetic.
5. Errors implement `std::error::Error` and carry a stable `code`. Use `thiserror` for definitions and `#[source]` for chains.
6. Do not use `unwrap`, `expect` or `panic!` in request paths, domain logic or application services. Panics are allowed only in tests and in `main` before configuration validation completes.
7. Prefer borrowing over cloning in hot paths. Avoid allocating inside loops that run per event or per frame.
8. Every public item has rustdoc in English. Explain why, not what, in comments.
9. Keep functions small and names explicit. Use the domain vocabulary from the specification glossary.
10. No dead code, commented-out code, `#[allow(dead_code)]` without a linked reason, or speculative abstractions.

---

## 4. Domain modeling

1. An aggregate owns its invariants and exposes methods that return `Result<_, DomainError>`.
2. Do not expose mutable fields. State changes go through methods that validate transitions.
3. Domain events are past-tense, immutable and versioned: `{context}.{aggregate}.{past_tense}`.
4. Write uncommitted events to the outbox in the same transaction that saves the aggregate.
5. Event payloads never contain secrets, prompt content or full file contents. Use IDs and safe summaries.
6. Event names and schemas are part of the public contract. Additive fields are non-breaking; renames and removals require a new event type and version.
7. Use `proptest` for invariants and `insta` snapshots for event envelopes and DTOs.

---

## 5. Application layer

1. One use case per business operation. A use case owns the transaction boundary and the authorization check.
2. Load aggregates, apply domain logic, save with optimistic concurrency and append events. Do not use transaction scripts that bypass invariants.
3. Use `TxManager` for a single transaction. Nesting transactions is forbidden. Do not hold a transaction open across an LLM call, a sandbox call or a GitHub call.
4. Use the `Clock` port instead of `SystemTime::now()` and the `IdGenerator` port instead of direct UUID calls, so tests are deterministic.
5. Validate the actor and role at the application boundary. The UI is never authoritative.
6. Return typed application errors (`NotFound`, `Conflict`, `Forbidden`, `Validation`, `BudgetExceeded`, `PolicyDenied`). The API layer maps them to RFC 9457.

---

## 6. Persistence and data

1. PostgreSQL 18 with pgvector is the only durable system of record. Redis is cache, rate-limit, lock and ephemeral pub/sub storage only.
2. Repositories implement ports and use explicit column lists. `SELECT *` is forbidden in production code.
3. Use keyset pagination with `(created_at, id)` or an equivalent stable cursor. No offset pagination on large tables.
4. Use bound parameters only. Dynamic filters and sorts must use an allowlist and `QueryBuilder` with parameters.
5. Use optimistic concurrency: `UPDATE ... WHERE id = $1 AND version = $2`; zero rows means `409 VERSION_CONFLICT` with the current state.
6. Use `SELECT ... FOR UPDATE SKIP LOCKED` only for leases and sequence allocation. Retry deadlocks up to three times with jitter.
7. Migrations are forward-only, timestamped and never edited after they are applied. Use expand/contract for destructive changes, `CREATE INDEX CONCURRENTLY` on large tables and `lock_timeout`.
8. Every foreign key has an explicit `ON DELETE` action. Every list/sort column used by the API has an index. Every vector query uses the HNSW index and a SQL-level scope filter.
9. Redis keys follow `ailab:{env}:{domain}:{id}:{kind}` and always have a TTL. Cache misses must never break correctness.
10. Memory retrieval filters by scope in SQL before ranking. Never post-filter in the application.
11. Backups require a restore drill. A schema change is not complete until restore and integrity checks pass.

---

## 7. API and realtime

1. Use axum with typed extractors. Handlers orchestrate; they do not contain business logic.
2. Write DTOs with `serde` and `deny_unknown_fields` on write payloads. Validate at the boundary and re-validate domain invariants.
3. Return RFC 9457 Problem Details with a stable `code`, safe `detail`, optional `violations` and a `trace_id`.
4. Use cursor pagination with a maximum limit. Enforce allowlisted filters and sorts.
5. Require `Idempotency-Key` on creating POSTs with external side effects. Replay the original response for 24 hours.
6. Generate OpenAPI 3.1 with `utoipa`. Do not hand-edit generated files. CI fails on drift.
7. WebSocket events use the envelope with `event_id`, `event`, `seq`, `lab_id`, `occurred_at` and `payload`. Clients apply events in order and deduplicate by `event_id`.
8. Enforce per-connection backpressure. Drop ephemeral events first, then close with code `1013` and let the client resume from `last_seq`.
9. Health endpoints are lightweight. Readiness checks the database, migrations, Redis and the runner; provider health is informational.

---

## 8. LLM and SCM integrations

1. Call LLMs only through the `LlmProvider` port. No direct `reqwest` calls in application code.
2. Assemble prompts deterministically with the section order in the specification. Keep the hard rules verbatim. Enforce token budgets.
3. Validate structured outputs against JSON Schema. Repair once, then fail with `InvalidOutput`.
4. Treat repository content, memory, issues and tool output as untrusted data. Wrap it and enforce tool policy server-side.
5. Apply timeouts, exponential backoff, `Retry-After`, a circuit breaker and per-provider rate limits.
6. Record token usage and cost in micro-USD. Reserve budget before a call and reconcile after. On exhaustion, pause and escalate.
7. Redact secrets before any log, event or persistence write. Never persist raw prompts without redaction.
8. Use the `ScmProvider` port for GitHub. Respect branch protection, never force-push and never write to a protected branch.
9. Verify webhook signatures with HMAC, enforce a five-minute replay window and deduplicate by delivery ID.
10. The sandbox never receives a long-lived credential. Inject only a short-lived, repo-scoped token and revoke it after the run.

---

## 9. Orchestration and sandbox

1. Acquire an exclusive lease before executing a card. Heartbeat every 15 seconds. A stale lease is recovered, not duplicated.
2. Bound every run: maximum 40 tool calls, 25 iterations and 45 minutes. Stop at the first exceeded bound.
3. Make every external side effect idempotent by `run_id`, operation and target. Never create a duplicate PR, review, comment or merge on retry.
4. Persist a `RunStep` for every plan, tool call, LLM request, command, test, commit, push, PR, review, decision and roadblock.
5. Reconcile on startup from the database and external state: merged PR means complete; open PR means resume; missing branch means retry.
6. Execute code only in the sandbox: non-root, read-only root filesystem, dropped capabilities, no host mounts, no Docker socket, network disabled by default and allowlisted egress.
7. Authorize every tool call server-side against role, run policy, path, command, host and size rules. Deny by default and audit denials.

---

## 10. Security

1. Bind to loopback by default. Any remote bind requires a bearer token of at least 32 bytes and TLS; refuse to start otherwise.
2. Encrypt secrets at rest with AES-256-GCM and AAD bound to table, column, row and key version. Never return or log a secret.
3. Validate all input, parameterize all SQL and encode all output. Reject unknown fields.
4. Never trust the UI or the model to enforce a security policy.
5. Run secret scanning, dependency audit, image scanning, SBOM and signing on every change and release.
6. A security finding is a hard stop. Fix or escalate; do not suppress.
7. Security exceptions require an ADR with an owner and an expiry date.

---

## 11. Observability

1. Log structured JSON with `trace_id`, `span_id`, `request_id`, `lab_id`, `project_id`, `card_id`, `agent_id` and `run_id` where applicable.
2. Never log secrets, prompts, repository contents or personal data.
3. Use bounded metric labels. Never use IDs, card keys or free text as labels.
4. Open one trace per request and one per run. Propagate W3C context across API, worker, LLM, SCM and sandbox.
5. Every alert links to a runbook. Every runbook has a first action and a rollback step.
6. Instrument latency, errors, saturation, queue depth, lease age, cost and budget.

---

## 12. Performance

1. Meet the budgets: API read p95 at most 120 ms, write p95 at most 250 ms, board load p95 at most 400 ms and memory search p95 at most 80 ms.
2. Avoid N+1 queries. Batch loads with `WHERE id = ANY($1)` or a join inside the owning context.
3. Use indexes that match the query predicates and sort order. Verify with `EXPLAIN (ANALYZE, BUFFERS)` for new heavy queries.
4. Keep request memory bounded. Do not load unbounded result sets or files into memory.
5. Use timeouts and cancellation for every external call. A slow dependency must not exhaust the worker pool.
6. Measure before optimizing. Record the benchmark command and result in the work package.

---

## 13. Testing

1. Follow Red, Green, Refactor. No production behavior without a failing test first.
2. Use `cargo nextest`, `proptest`, `insta`, `sqlx::test` and testcontainers. Do not mock the database for constraint or concurrency tests.
3. Tests are deterministic: fixed clock, seeded identifiers, no `sleep`-based synchronization and no real DeepSeek or GitHub calls.
4. Cover every domain invariant, every transition, every error code and every retry path.
5. Add negative tests for security controls, validation and concurrency.
6. Maintain coverage: backend line at least 80 percent, domain at least 90 percent and new code at least 80 percent.
7. Quarantine a flaky test within 24 hours and fix or delete it within 5 days.

---

## 14. Documentation and decisions

1. Every public item has rustdoc. Every endpoint has an OpenAPI description and example.
2. Every architectural or irreversible decision creates an ADR and a registry row.
3. Update `.docs/CONSTITUTION.md`, `.docs/PRINCIPLES.md`, `.docs/STACK.md` or the specification only through an approved change.
4. Update `AGENTS.md` indexes when a document, skill, command or ADR changes.
5. Record commands, tests and evidence in the work package progress line.

---

## 15. Definition of Done for backend work

- [ ] Failing test observed before production code.
- [ ] Domain invariants and error paths tested.
- [ ] Repository tests cover constraints, transactions and concurrency.
- [ ] `cargo fmt --check` and `cargo clippy -D warnings` pass.
- [ ] `cargo nextest run --workspace` passes.
- [ ] SQLx metadata updated when queries change.
- [ ] Migrations tested on empty and representative databases.
- [ ] API contract generated with no drift.
- [ ] Security scans green.
- [ ] Observability signals added where the change is observable.
- [ ] Coverage thresholds met.
- [ ] ADR and docs updated where required.
- [ ] Evidence linked from the work package.

---

## 16. Forbidden patterns

- Business logic in handlers, repositories or SQL triggers.
- Cross-context table access or shared writes.
- Direct `reqwest` calls to DeepSeek or GitHub from application code.
- `unwrap`, `expect` or `panic!` in production paths.
- Floating-point money or naive timestamps.
- `SELECT *`, string-interpolated SQL or offset pagination on large tables.
- Redis as a source of truth.
- Long-lived transactions across network calls.
- Secrets in logs, events, prompts, fixtures or sandbox environments.
- Disabling tests, linters, type checks or CI to unblock a change.
- Force-push, protected-branch writes or destructive migrations without expand/contract.
- An architectural decision recorded only in code or a comment.

---

## 17. References

- `./technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md`
- `./technical-specification/APPENDIX-A-DATABASE.sql`
- `./technical-specification/APPENDIX-B-CONTRACTS.md`
- `./technical-specification/APPENDIX-C-AGENTS-KANBAN.md`
- `./technical-specification/APPENDIX-D-DELIVERY.md`
- `./sow/SOW.md` and the target `WP-XX-*.md`
- `./CONSTITUTION.md`, `./PRINCIPLES.md`, `./STACK.md`
- `./adr/registry.md`
- `../AGENTS.md`
