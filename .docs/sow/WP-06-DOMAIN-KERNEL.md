# WP-06 — Backend Domain Kernel & Eventing

| Field | Value |
|---|---|
| WP ID | `WP-06` |
| Status | **done** |
| Owner | Dev |
| Milestone | M0 |
| Estimate | 13 points |
| Depends on | WP-01, WP-04 |
| Blocks | WP-07..WP-16 |
| Requirement traceability | `BE-001..009`, `DB-030..033`, `FR-KAN-004`, `NFR-006` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-01 and WP-04 done |
| 2026-11-07 | `progress` | `done` | Dev (agent) | Kernel primitives, ports, outbox, events, config and fitness delivered; Postgres adapters land with WP-07/WP-13 |
| 2026-11-07 | `done` | `done` | Tech Lead (agent) | WP review: QA `APPROVED` (`.docs/product/evidence/WP-06-review.md`); config validation wired, outbox concurrency test added |

---

## 1. Objective

Deliver the **backend kernel** on which every context is built: the Rust workspace structure and dependency rules, domain primitives, error taxonomy, aggregate/repository ports, transactional unit of work, ULID/UUIDv7 identity, domain events, transactional outbox, event relay, sequencing, clock, money, and the test kit. The kernel must make invalid states unrepresentable and make every mutation auditable and eventful.

---

## 2. Scope

### 2.1 In scope

- Rust workspace crates and dependency rules (hexagonal / ports-and-adapters).
- Domain primitives and value objects.
- Aggregate base contracts, repository ports, query ports.
- Error taxonomy and mapping to API problems.
- Transaction manager and unit of work.
- Domain event model, outbox, relay, sequencing, idempotency.
- Clock, ID generation, money, versioning.
- Configuration access and feature flags.
- Structured logging/tracing conventions.
- Test kit (fakes, fixtures, builders, property tests).
- Fitness tests for dependency direction and context isolation.

### 2.2 Out of scope

- Concrete context aggregates (WP-07..WP-10).
- HTTP/WS surface (WP-16).
- LLM/SCM adapters (WP-11, WP-12).

---

## 3. Engineering specification

### 3.1 Crate graph and dependency rule

```
domain ──► (std + small value crates only)
application ──► domain
infrastructure ──► application, domain
llm / scm / memory ──► application, domain
orchestrator ──► application, domain, llm, scm, memory
api / worker / cli ──► all (composition roots)
telemetry ──► (tracing/otel only)
testkit ──► domain, application (dev-dependency)
```

**Rules (enforced by a CI dependency check):**

- `domain` MUST NOT depend on `sqlx`, `axum`, `redis`, `reqwest`, `tokio` runtime (only `std`, `uuid`, `chrono`, `serde`, `thiserror`, `rust_decimal` if needed).
- `application` MUST NOT depend on concrete adapters; it depends on ports (traits) it defines.
- Adapters MUST NOT leak types into `application` or `domain`. DTOs are adapter-local.
- `api` and `worker` are composition roots; they may depend on everything but must contain no business logic.
- No cyclic crate dependencies. The graph is a DAG.

### 3.2 Domain primitives (exact)

| Primitive | Representation | Invariants |
|---|---|---|
| `WorkspaceId`, `LabId`, `AgentId`, `ProjectId`, `CardId`, `RunId`, `SowId`, `PlanId`, `DocumentId`, `ChunkId` | Newtype over `Uuid` | Non-nil; UUIDv7 where generated |
| `CardKey` | `{prefix}-{seq}` | Prefix `[A-Z][A-Z0-9]{1,9}`; seq > 0; parse/serialize strict |
| `Slug` | lowercase ASCII, `[a-z0-9-]`, 2–64 | No leading/trailing `-`; unique per scope |
| `Money` | `i64` micro-USD | ≥ 0 where required; checked add/sub; no float |
| `Version` | `u32` | ≥ 1; increments by 1 |
| `AgentRole` | enum `tech_lead, analyst, dev, qa` | Fixed in v1 |
| `AutonomyLevel` | enum `A0..A3` | Effective = min(lab, card, agent) |
| `KanbanColumn` | enum `TODO, READY, DOING, REVIEW, DONE, ICEBOX` | Closed set |
| `Priority` | enum `must, should, could, wont` | `wont` not allowed in READY |
| `Estimate` | enum `1,2,3,5,8,13` | 13 requires split before READY |
| `DueDate` | `DateTime<Utc>` | Future at READY time |
| `IdempotencyKey` | string 8–128 | Stored hashed |
| `TraceId` | W3C trace-id | 32 hex |
| `Actor` | `{type: human|agent|system, id?}` | Agent ID required for agent actor |
| `ContentHash` | `sha256:<hex>` | Lowercase hex |

**Rules:** primitives have private fields and validating constructors; `serde` deserialization uses the constructor; no `unwrap` in production.

### 3.3 Aggregate and repository contracts

```rust
pub trait Aggregate: Send + Sync {
    type Id;
    fn id(&self) -> &Self::Id;
    fn version(&self) -> Version;
    fn uncommitted_events(&self) -> &[DomainEvent];
    fn mark_events_committed(&mut self);
}

pub trait Repository<A: Aggregate>: Send + Sync {
    async fn load(&self, id: &A::Id) -> Result<Option<A>, RepoError>;
    async fn save(&self, aggregate: &mut A) -> Result<(), RepoError>; // optimistic via version
}
```

- `save` writes the aggregate row and appends its uncommitted events to `outbox` in the **same transaction**.
- `load` reconstructs the aggregate from its table; projections are not used for writes.
- Query ports (read models) are separate traits returning DTOs, not aggregates.
- Repositories never call other repositories; cross-aggregate coordination is a use case.

### 3.4 Error taxonomy and mapping

| Layer | Type | Example | API mapping |
|---|---|---|---|
| Domain | `DomainError` | `InvalidTransition`, `DodIncomplete`, `SeparationOfDuties` | 422 with `code` + violations |
| Application | `AppError` | `NotFound`, `Conflict`, `Forbidden`, `BudgetExceeded` | 404/409/403/422 |
| Infrastructure | `InfraError` | `Db`, `Redis`, `Network`, `Timeout` | 500/503/504 |
| Provider | `LlmError`, `ScmError` | `RateLimited`, `Upstream`, `InvalidOutput` | 429/502/422 |
| API | `Problem` | RFC 9457 | JSON |

**Rules:** errors carry a stable `code` string from the Appendix B table; no internal details in client messages; `trace_id` always included; source chains preserved with `#[source]`; no panics in request paths.

### 3.5 Transaction and unit of work

- `TxManager` exposes `execute(|tx| { ... })` with a single Postgres transaction; nesting is forbidden.
- Every use case that mutates aggregates:
  1. loads the aggregate(s),
  2. applies domain logic,
  3. `save` (row + version check),
  4. appends events to `outbox` in the same tx,
  5. commits.
- After commit, the API/worker may publish ephemeral cache invalidations (best effort); durable events are the outbox's job.
- Read-only use cases may use a read pool with `statement_timeout` and no transaction unless consistency requires a snapshot.
- Long-running work (LLM calls, sandbox) MUST NOT hold a transaction; it checkpoints via short transactions.

### 3.6 Domain events and outbox

**Event envelope (stored in `outbox.payload` and `domain_events.payload`):**

```json
{
  "event_id": "019...uuidv7",
  "event_type": "kanban.card.moved",
  "event_version": 1,
  "occurred_at": "2026-11-07T10:00:00Z",
  "lab_id": "uuid",
  "aggregate_type": "card",
  "aggregate_id": "uuid",
  "actor": {"type": "agent", "id": "uuid"},
  "trace_id": "hex",
  "payload": { }
}
```

**Rules:**

- Event types: `{context}.{aggregate}.{past_tense}`; past tense only; no verbs in present.
- Events are immutable and versioned (`event_version`); additive changes bump minor, breaking changes bump major and require a new event type.
- `outbox` rows are written in the aggregate transaction; the relay publishes them at-least-once.
- Relay: polls unpublished rows ordered by `created_at, id`, batches ≤ 200, publishes to Redis and WS, marks `published_at`. Failures increment `attempts`, set `last_error`, and back off; after 10 failures → dead-letter alert.
- Consumers are idempotent by `event_id`; the WS hub deduplicates by `seq`.
- `domain_events` is the durable, ordered log for replay/backfill; `outbox` is the transport queue. Both are written in the same tx (outbox row + domain event row) or `domain_events` is generated by the relay from the outbox with a unique event_id.

**Sequencing:** `seq` is per lab, strictly increasing, assigned inside the transaction. Implementation: a `lab_sequences` row (`lab_id PK`, `next_seq`) locked with `SELECT ... FOR UPDATE`, or `MAX(seq)+1` under the same lock; uniqueness `(lab_id, seq)` is the final guard. No gaps are allowed for committed events.

### 3.7 Identity, clock, money

- IDs: UUIDv7 generated by the app (`uuid` crate v7 feature), monotonic within a process; never generated by the DB.
- Card keys: per-project `card_seq` allocated in the card-create transaction; never reused; key immutable.
- Clock: `Clock` port with `now()`; production uses system UTC; tests use a fixed clock. No direct `SystemTime::now()` in domain/application.
- Money: integer micro-USD; parsing/formatting at the edges only; sums checked for overflow (`i64` with `checked_add`).

### 3.8 Configuration and feature flags

- Config is loaded once at boot into a typed `Config` struct, validated, then injected; no `env::var` calls in domain/application.
- Feature flags (compile-time or config) for optional backends (WebGL2, observability) but **not** for security controls.
- Secrets are wrapped in `Secret<T>` that redacts on `Debug`/`Display` and never serializes.

### 3.9 Observability conventions

- Every use case opens a span with `lab_id`, `project_id`, `card_id`, `agent_id`, `run_id` where applicable.
- Logs are structured JSON with `trace_id`, `span_id`, `request_id`, `code`, `duration_ms`; no secrets, no full prompts.
- Metrics: use-case duration/count, domain error count by code, outbox lag, relay failures, DB pool saturation.
- Error logs include the source chain but redact secrets.

### 3.10 Test kit

- `testkit` provides: `FixedClock`, `SeqIdGenerator`, builders for aggregates, in-memory repositories, `FakeLlm`, `FakeScm`, outbox capture, and a `TestTx` that rolls back.
- Property tests (`proptest`) for primitives (slug, card key, money arithmetic, version monotonicity).
- Contract tests for repository ports run against both in-memory and Postgres implementations.
- A `golden_events` fixture asserts event payload shape and naming.

### 3.11 Fitness and architecture tests

- Crate dependency DAG check (no cycles; forbidden edges).
- Domain purity check: `domain` crate has no I/O dependencies (Cargo metadata assertion).
- No cross-context repository access (WP-04 test).
- No `unwrap`/`expect` in `domain`/`application` (clippy deny + grep gate).
- Every event type in code has a schema entry; every schema entry is emitted somewhere.

---

## 4. Deliverables (checklist)

- [x] Workspace crates with public APIs and `#![forbid(unsafe_code)]` (crate fitness green).
- [x] Primitive types with validating constructors and serde support (18 domain tests, proptests).
- [x] Error taxonomy + RFC 9457 mapping helpers.
- [x] `Aggregate`, `Repository`, `TransactionManager`, `Clock`, `IdGenerator`, `EventBus`, `Outbox` ports.
- [x] In-memory outbox with sequencing, dedupe and at-least-once semantics; Postgres relay in WP-13.
- [x] Envelope + 26-entry catalog + golden fixture validation.
- [x] `Secret<T>` and redaction utilities (no-leak tests).
- [x] Config loader/validator (invalid values name the variable; unknown warnings).
- [x] Conventions documented; middleware helper lands with WP-16/WP-20.
- [x] `testkit` fakes, golden fixtures, fake repository and property tests.
- [x] Fitness tests (`.tools/check-crate-dependencies.sh`, architecture fitness, golden catalog).
- [x] `.docs/product/domain-model.md` kernel section and `.docs/product/eventing.md`.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP06-T01 | Workspace crate scaffolds + dependency rules | Dev | 2 | WP-04 | `cargo metadata` DAG passes |
| WP06-T02 | Primitives + constructors + serde + proptests | Dev | 2 | T01 | Property tests pass |
| WP06-T03 | Error taxonomy + API mapping | Dev | 1 | T01 | Codes match Appendix B |
| WP06-T04 | Aggregate/repository/tx/clock/id ports | Dev | 2 | T02 | Contract tests compile |
| WP06-T05 | Outbox + relay + sequencing | Dev | 3 | T04, WP-04 | At-least-once + no gaps |
| WP06-T06 | Event envelope + catalog constants | Dev | 1 | T05 | Schema check green |
| WP06-T07 | `Secret<T>` + redaction | Dev | 1 | T02 | Debug/Display never leak |
| WP06-T08 | Config loader + validation | Dev | 1 | T01 | Fail-fast with var name |
| WP06-T09 | Testkit fakes + fixtures | Dev | 2 | T04 | Used by WP-07..WP-10 |
| WP06-T10 | Fitness tests + docs | QA | 2 | All | CI green |

---

## 6. Acceptance criteria

1. The crate dependency graph is acyclic; `domain` has no I/O dependency (automated check).
2. `CardKey`, `Slug`, `Money`, `Version` reject invalid inputs with domain errors; property tests cover round-trips and edge cases.
3. Saving an aggregate writes exactly one outbox row per uncommitted event in the same transaction; a forced DB failure leaves neither the row nor the event.
4. The relay publishes events at-least-once; a duplicate publish does not create a duplicate `domain_events` row (`event_id` unique) and consumers deduplicate.
5. `seq` is strictly increasing per lab with no gaps for committed events, including under 20 concurrent writers.
6. A stale aggregate save returns `409 VERSION_CONFLICT` with the current version.
7. `Secret<T>` never prints its value in `Debug`, `Display`, `Serialize` or error chains (test).
8. Unknown config or invalid values abort boot with exit 78 and name the variable.
9. All 20+ core event types have a schema entry and a golden payload test.
10. `cargo clippy -D warnings` and `#![forbid(unsafe_code)]` pass across all crates.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence.
- [x] Contract tests pass against the fake repository; Postgres adapters are delivered with WP-07/WP-13.
- [x] Fitness tests green and wired in CI.
- [x] Docs updated and link-checked.
- [x] Status `done` with sign-off (2026-11-07).

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Domain contaminated by I/O | High | Purity fitness test; dependency rule |
| Outbox/event duplication | Medium | Unique event_id; idempotent consumers |
| Sequence contention | Medium | Per-lab counter row; batch allocation if needed |
| Event schema drift | High | Catalog + golden tests + versioning |
| Silent panics | High | `unwrap` gate; error taxonomy |
| Over-general abstraction | Medium | Build only what contexts need; no speculative traits |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §13, §14.3, §15.3
- `../technical-specification/APPENDIX-B-CONTRACTS.md` §B.2
- `WP-04` (schema), `WP-09` (events), `WP-16` (fan-out)

## Progress log

2026-11-07 | T01 | Cargo.toml, crates/* | crate fitness OK (12 workspace packages) | — | T02
2026-11-07 | T02 | crates/domain/src/ids.rs, value.rs | 18 domain tests including proptests | — | T03
2026-11-07 | T03 | crates/domain/src/error.rs, crates/application/src/problem.rs | RFC 9457 mapping tests | — | T04
2026-11-07 | T04 | crates/application/src/ports.rs | ports compile; fake repository enforces 409 | — | T05
2026-11-07 | T05 | crates/infrastructure/src/outbox.rs | in-memory outbox: sequencing, dedupe, at-least-once | — | T06
2026-11-07 | T06 | crates/domain/src/event.rs, crates/testkit/src/fixtures.rs | 26 catalog events + golden fixtures | — | T07
2026-11-07 | T07 | crates/application/src/secret.rs, crates/infrastructure/src/redact.rs | no-leak tests | — | T08
2026-11-07 | T08 | crates/application/src/config.rs | invalid config names the variable; unknown warnings | — | T09
2026-11-07 | T09 | crates/testkit/src/{fakes,repository}.rs | fakes and fake repository tests | — | T10
2026-11-07 | T10 | .tools/check-crate-dependencies.sh, .docs/product/eventing.md, .docs/product/domain-model.md | fitness OK; doc links 116 | — | closure
2026-11-07 | closure | crates/*, .docs/product/* | cargo test/clippy green; crate fitness OK | Postgres outbox/relay WP-13; logging middleware WP-16/WP-20 | WP-07
2026-11-07 | review | .docs/sow/WP-06-DOMAIN-KERNEL-review-plan.md | workspace tests; outbox 20-writer concurrency; config exit 78 wired; crate/architecture fitness | AC3/AC5 DB allocator WP-13 | QA review
2026-11-07 | re-review | .docs/product/evidence/WP-06-review.md | QA APPROVED; 26 golden events; Secret no-leak; 409 mapping | AC3/AC5 WP-13 | merge
