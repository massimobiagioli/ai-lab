# WP-06 — Backend Domain Kernel & Eventing Review Plan

| Field | Value |
|---|---|
| WP | `WP-06` |
| Contract | `.docs/sow/WP-06-DOMAIN-KERNEL.md` |
| Review branch | `review/wp-06-domain-kernel` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_review` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present and consistent.
- [x] Requirement traceability maps to spec §13/§14.3/§15.3 and Appendix B §B.2.

## 2. Specification conformance

- [x] Specification §13 (domain), §14.3 (events), §15.3 (contracts) — evidence: `crates/domain`, `crates/application`, `crates/infrastructure`, `.docs/product/domain-model.md`, `.docs/product/eventing.md`.
- [x] Appendix B §B.2 event envelope — evidence: `crates/domain/src/event.rs`, `crates/testkit/src/fixtures.rs`.
- [x] ADR-0004 outbox, ADR-0002 bounded contexts — evidence: registry.

## 3. Deliverables

- [x] Workspace crates with `#![forbid(unsafe_code)]` — evidence: `.tools/check-crate-dependencies.sh`.
- [x] Primitive types with validating constructors + serde — evidence: `crates/domain/src/{ids,value}.rs`.
- [x] Error taxonomy + RFC 9457 mapping — evidence: `crates/domain/src/error.rs`, `crates/application/src/problem.rs`.
- [x] Ports (Aggregate, Repository, TransactionManager, Clock, IdGenerator, EventBus, Outbox) — evidence: `crates/application/src/ports.rs`, `crates/domain/src/aggregate.rs`.
- [x] In-memory outbox (sequencing, dedupe, at-least-once, 20-writer concurrency) — evidence: `crates/infrastructure/src/outbox.rs`; Postgres relay WP-13.
- [x] Envelope + 26-entry catalog + golden fixtures — evidence: `crates/domain/src/event.rs`, `crates/testkit/src/fixtures.rs`.
- [x] `Secret<T>` no-leak — evidence: `crates/application/src/secret.rs`.
- [x] Config loader/validator — evidence: `crates/application/src/config.rs`; wired into the API startup (exit 78).
- [x] Conventions documented — evidence: `.docs/product/domain-model.md`, `.docs/product/eventing.md`.
- [x] Testkit fakes + fake repository + properties — evidence: `crates/testkit/src/{fakes,repository,fixtures}.rs`.
- [x] Fitness tests — evidence: `.tools/check-crate-dependencies.sh`, architecture fitness, golden catalog.

## 4. Acceptance criteria

- [x] AC1 acyclic crate graph; `domain` has no I/O — evidence: `.tools/check-crate-dependencies.sh` (12 packages).
- [x] AC2 primitives reject invalid inputs; property tests — evidence: domain tests (49 passed).
- [x] AC3 aggregate save writes one outbox row per event in the same transaction; DB failure rolls back — DEFERRED to WP-13 (Postgres adapter); in-memory append semantics delivered.
- [x] AC4 relay at-least-once; duplicate `event_id` does not duplicate — evidence: in-memory dedupe test; Postgres unique constraint in Appendix A.
- [x] AC5 `seq` strictly increasing per lab, no gaps — evidence: in-memory monotonic test plus a 20-writer concurrency test; DB allocator WP-13.
- [x] AC6 stale save returns `409 VERSION_CONFLICT` — evidence: `crates/testkit/src/repository.rs`, fake repository test.
- [x] AC7 `Secret<T>` never leaks in Debug/Display/Serialize — evidence: `secret.rs` no-leak tests.
- [x] AC8 invalid config aborts dispatch with the variable named — evidence: `config.rs` tests; `ailab-api` now calls `Config::from_env` and exits 78.
- [x] AC9 20+ events with schema entry and golden fixture — evidence: 26-entry catalog; `every_catalog_event_has_a_golden_fixture`.
- [x] AC10 clippy `-D warnings` and `#![forbid(unsafe_code)]` — evidence: clippy green; crate fitness.

## 5. Implementation and tests

- [x] Primitives, errors, ports, outbox, catalog, secret, config, testkit — tests green.
- [x] Concurrency test for the outbox — 20 writers produce 1..=20.
- [x] No real external API in tests.
- [x] Coverage thresholds met (domain/kernel crate).

## 6. Security and quality gates

- [x] `Secret<T>` redaction and no-leak tests; config exposure guard.
- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` green.
- [x] `.tools/check-crate-dependencies.sh` and architecture fitness green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/domain-model.md` and `.docs/product/eventing.md` updated.
- [x] `.docs/adr/registry.md` consistent (ADR-0002, ADR-0004).
- [x] `AGENTS.md` indexes updated for this review plan.
- [x] `.docs/sow/SOW.md` and WP-06 status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED`.
- [ ] QA evidence: reproduced gates (harness 16, sow-lint 31, doc links 143, crate fitness, workspace tests, outbox concurrency, config exit 78).

## 9. Deferred items and residual risks

- [x] AC3 Postgres outbox/relay transaction — owner: Dev, WP-13.
- [x] AC5 DB sequence allocator under load — owner: Dev, WP-13.
- [x] Logging middleware — owner: Dev, WP-16/WP-20.
- [x] No unexplained deferral remains.

## 10. Decisions

- [x] ADR-0004 transactional outbox and ADR-0002 bounded contexts recorded.
