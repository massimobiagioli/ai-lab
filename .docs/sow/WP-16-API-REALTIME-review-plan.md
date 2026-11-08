# WP-16 — API & Realtime Contracts Review Plan

| Field | Value |
|---|---|
| WP | `WP-16` |
| Contract | `.docs/sow/WP-16-API-REALTIME.md` |
| Review branch | `review/wp-16-api-realtime` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_progress` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present.
- [x] Requirement traceability maps to `API-001..008`, `WS-001..008`, `BE-010..017`, `FR-RT-*`.

## 2. Specification conformance

- [x] RFC 9457 Problem Details — evidence: `application::problem`, server 404/422/429 payloads.
- [x] Cursor pagination defaults/bounds — evidence: `PageQuery`.
- [x] Rate limits and origin/JSON guards — evidence: `RateLimiter`, `RateLimitPolicy`, `OriginPolicy`.
- [x] WS envelope/`events_after`/hello — evidence: `lib.rs`, `server.rs`.
- [x] Axum router with health and run endpoints — evidence: `crates/api/src/server.rs` (increment).
- [x] Bearer auth middleware (when `AILAB_API_TOKEN` is set; health stays open).
- [x] In-memory idempotency store (24 h) for the POST endpoints.
- [x] WS hub with live event broadcast and 20 s heartbeat.
- [x] Outbox relay → API backfill store + live hub (`/api/v1/events`, `relay_once`) — test.
- [x] `GET /api/v1/openapi.json` serves the generated spec; `POST /api/v1/telemetry` ingests frontend telemetry with validation — tests.
- [ ] Full DTO/OpenAPI annotations, Redis idempotency, WS backpressure, Swagger UI assets, contract/perf tests.

## 3. Deliverables

- [x] Axum router (health, runs, labs, cards) — increment.
- [x] RFC 9457 Problem Details — increment.
- [x] Pagination, rate limiting, origin/JSON guards — increment.
- [x] WS hello/subscribe + `events_after` — increment.
- [x] OpenAPI determinism with the run paths — `make openapi`.
- [x] Auth middleware, idempotency store, WS broadcast + heartbeat, outbox relay/backfill, OpenAPI endpoint, telemetry ingest — increment.
- [ ] Full DTO surface, Redis idempotency, WS backpressure, Swagger UI assets — open.

## 4. Acceptance criteria

- [x] Health and run endpoints return the expected status/payload — API tests.
- [x] 404 returns a valid RFC 9457 Problem — test.
- [x] Rate limits are class-specific — test.
- [x] Auth middleware over the protected router + health open — test.
- [x] Idempotency replay for POST — test.
- [x] WS hello/subscribe + live broadcast + heartbeat — implementation.
- [x] Outbox relay drains the outbox, persists for backfill and broadcasts live — test.
- [x] OpenAPI endpoint and frontend telemetry ingest — tests.
- [ ] WS backpressure under load, Redis idempotency, Swagger UI assets, perf — open.

## 5. Implementation and tests

- [x] `server.rs` router, in-memory `RunStore`, Problem Details, rate limiter, auth, idempotency, WS hub/heartbeat, outbox relay/backfill, OpenAPI endpoint, telemetry ingest — 13 API tests + 2 contract tests.
- [ ] Postgres/Redis adapters, full DTOs, WS hub — build phase.

## 6. Security and quality gates

- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p ailab-api` green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/api/README.md`/`realtime.md` present.
- [x] `AGENTS.md` indexes updated for this review plan.
- [ ] WP-16 status remains `progress` until the full edge contract is delivered.

## 8. QA review

- [ ] QA subagent verdict: `PENDING`.
- [x] Author blocker: full DTO/middleware/WS-hub/outbox/telemetry/perf contract not yet implemented (CR-0010).

## 9. Deferred items and residual risks

- [ ] Full DTO surface + Swagger UI, idempotency store, WS hub/backpressure, outbox fan-out, telemetry, perf — build phase + WP-20/WP-21.
- [ ] No merge/closure until the blockers are delivered and QA returns `APPROVED`.

## 10. Decisions

- [x] CR-0010 reopens WP-16 for the edge build.
