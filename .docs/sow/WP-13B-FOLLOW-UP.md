# WP-13B — Backend Hardening Follow-up (WP-13/WP-16 residuals)

| Field | Value |
|---|---|
| WP ID | `WP-13B` |
| Status | **ready** |
| Owner | Dev |
| Milestone | M8+ |
| Estimate | 34 points |
| Depends on | WP-13, WP-16, WP-20 |
| Blocks | — |
| Requirement traceability | `FR-ORC-010`, `FR-ORC-030`, `FR-ORC-031`, `FR-LLM-002`, `FR-GH-002`, `API-001..008`, `WS-005`, `WS-007`, `BE-012`, `BE-015`, `BE-016` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead (agent) | Created by CR-0012: WP-13/WP-16 residual hardening deferred to the final delivery slot so the frontend work packages can proceed |

---

## 1. Objective

Close the residual hardening items that WP-13 and WP-16 could not deliver before the frontend milestone: real context retrieval, open/branch resume, OpenTelemetry spans/metrics, server-enforced DoD, real LLM/SCM adapters, Redis rate limiting/fan-out, the full REST operation surface, WebSocket ephemeral/backpressure, and generated OpenAPI with breaking-change detection. This work package is intentionally last: it hardens the backend once the frontend and product surfaces are in place.

---

## 2. Scope

### 2.1 In scope

- Open-PR/branch-only recovery that resumes at review or push instead of re-running the whole pipeline.
- Context packet assembled from the repository map, top-8 memory retrieval with citations, relevant files, conventions and prior review feedback.
- OpenTelemetry spans for LLM, SCM and sandbox calls plus queue-depth, lease-age and run-duration metrics.
- DONE gated by the Kanban transition service and the card Definition of Done.
- Production `DeepSeekProvider` + `GitHubClient` wiring through a real HTTP transport (no fakes outside an explicit dev flag).
- Redis token-bucket rate limiting that fails closed and Redis pub/sub fan-out for the WebSocket hub.
- Real implementations (or explicitly documented read models) for the remaining Appendix B operations, with schemathesis coverage.
- WebSocket per-connection buffer with ephemeral-first drop and integration tests.
- OpenAPI generated from the router/DTOs with `openapi-diff` breaking-change checks and the full DTO schema set.
- Readiness checks for PostgreSQL, migrations, Redis and the runner.

### 2.2 Out of scope

- New product features outside the deliveries listed in WP-13/WP-16.
- Changes to the accepted domain model, event catalog semantics or autonomy policy.
- Frontend/office/product UI work (WP-17..WP-19).

---

## 3. Engineering specification

### 3.1 Resume

`RunExecutor::recover` returns the action and the external state. `ResumeReview` resumes at the review/merge tail; `ResumePush` resumes at commit/push/PR. The tail reuses the persisted branch and PR, validates the head SHA and never repeats the implement/verify stages unless the external state is absent.

### 3.2 Context

The context packet is composed from `ContextProvider` ports: repository map, memory retrieval (top-8, scoped, with citations), relevant-file selection, conventions excerpts and previous review feedback. The token budget and section priority stay as in WP-13 §3.4.

### 3.3 Observability

Every run opens a root span (`run_id`, `card_id`, `lab_id`, `trace_id`) and child spans for each LLM, SCM and sandbox call. Metrics: queue depth, lease age, run duration, tool duration, token and cost counters. Export through the WP-20 collector configuration.

### 3.4 DoD and adapters

The DONE transition goes through the Kanban transition service (`REVIEW → DONE`) with the server-evaluated DoD and the SCM merge SHA. The worker composes `DeepSeekProvider`, `GitHubClient` and `HttpSandboxBackend`; fakes are available only when `AILAB_DEV_FAKES` is set.

### 3.5 Redis and edge

The rate limiter uses a Redis token bucket (`INCR`/`EXPIRE`) keyed by actor+IP and fails closed on Redis errors. The outbox relay publishes to Redis; the API subscribes and fans out per lab/topic. WebSocket connections use a 256-event buffer, drop `office.agent.speech`/ephemeral events first, then close `1013` for undeliverable state events.

### 3.6 Contracts

The remaining Appendix B operations are implemented with typed DTOs. `openapi.json` is generated from the router/DTOs; CI runs `openapi-diff` and schemathesis. Readiness returns per-component status and `NOT_READY` Problem Details.

---

## 4. Deliverables (checklist)

- [ ] Open-PR and branch-only resume.
- [ ] Real context provider (repo map, memory top-8 with citations, files, conventions, feedback).
- [ ] OTel spans and run/queue/lease metrics.
- [ ] Kanban-service DONE transition with DoD.
- [ ] Real DeepSeek/GitHub adapters behind the HTTP transport; fakes dev-gated only.
- [ ] Redis rate limiter (fail closed) and pub/sub fan-out.
- [ ] Full Appendix B operation implementations (or documented read models) with schemathesis.
- [ ] WS per-connection buffer, ephemeral-first drop, integration tests.
- [ ] Generated OpenAPI + `openapi-diff` + full DTO schemas.
- [ ] Readiness component checks.
- [ ] Tests and docs for every item above.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP13B-T01 | Resume open/branch recovery | Dev | 4 | WP-13 | Tail reuses branch/PR and head SHA |
| WP13B-T02 | Real context provider | Dev | 5 | WP-08, WP-13 | Citations from real sources |
| WP13B-T03 | OTel spans + metrics | Dev | 3 | WP-20 | Spans/metrics exported |
| WP13B-T04 | Kanban DONE + DoD | Dev | 3 | WP-09, WP-15 | DONE only with DoD |
| WP13B-T05 | Real LLM/SCM adapters | Dev | 4 | WP-11, WP-12 | Production wiring |
| WP13B-T06 | Redis limiter + fan-out | Dev | 4 | WP-16 | Fail-closed; fan-out test |
| WP13B-T07 | Full REST surface + schemathesis | Dev | 5 | WP-16 | Contract green |
| WP13B-T08 | WS buffer/ephemeral + tests | Dev | 3 | WP-16 | Drop order + 1013 |
| WP13B-T09 | Generated OpenAPI + breaking gate | Dev | 3 | WP-16 | CI gate green |
| WP13B-T10 | Readiness components | Dev | 2 | WP-16 | Per-component 503 |
| WP13B-T11 | Tests + docs | QA | 3 | All | Gates green |

---

## 6. Acceptance criteria

1. A worker killed after merge reaches DONE from the external merge SHA; an open PR resumes at review and a branch-only state resumes at push.
2. The context packet contains real repository, memory, file, convention and feedback content with citations.
3. Every run and every LLM/SCM/sandbox call has a span; queue depth, lease age, duration, token and cost metrics are exported.
4. DONE is written only through the Kanban transition service after the DoD passes.
5. The production worker uses real DeepSeek/GitHub adapters; fakes are reachable only through `AILAB_DEV_FAKES`.
6. Rate limiting uses a Redis token bucket that denies when Redis is unavailable; the relay fans out through Redis.
7. Every Appendix B operation returns its documented schema; schemathesis reports zero failures.
8. WebSocket backpressure drops ephemeral events first and closes `1013` only for undeliverable state events; integration tests cover replay, resync and heartbeat.
9. OpenAPI is generated, drift-free and guarded by a breaking-change check; the full DTO schema set is published.
10. `/health/ready` returns per-component status and `NOT_READY` Problem Details.

---

## 7. WP Definition of Done

- [ ] All deliverables checked with evidence.
- [ ] WP13B-T01..T11 complete; no P0/P1 open.
- [ ] Quality gates green (backend, frontend, contracts, schemathesis, load).
- [ ] Docs updated; ADRs recorded for any new decision.
- [ ] Independent QA `APPROVED`.

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Memory/repo providers not available | Context remains stubbed | Port interfaces with a deterministic local provider; swap in WP-08 infra |
| Redis unavailable in CI | Limiter/fan-out untested | Ignored integration test plus in-memory fallback for unit tests |
| Full REST surface is large | Schedule slip | Implement per resource with schemathesis-driven contracts |
| OpenAPI generator mismatch | CI drift | Generate from the router/DTO source of truth and diff in CI |

---

## 9. References

- `.docs/sow/WP-13-ORCHESTRATION.md`, `.docs/sow/WP-16-API-REALTIME.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §7–§9, §13, §15
- `.docs/product/api/openapi.json`, `.docs/product/api/asyncapi.json`
- ADR-0004, ADR-0009, ADR-0011, ADR-0014, ADR-0023, ADR-0025
- `.docs/product/change-requests/CR-0012.md`
