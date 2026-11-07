# WP-01 — Architecture & Service Topology

| Field | Value |
|---|---|
| WP ID | `WP-01` |
| Status | **done** |
| Owner | Tech Lead |
| Milestone | M0 |
| Estimate | 13 points |
| Depends on | WP-00 |
| Blocks | WP-02..WP-22 |
| Requirement traceability | `NFR-001..NFR-012`, `FR-ORC-*`, `DB-*`, `BE-001..022`, SOW §11 |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Tech Lead (agent) | WP-00 done; T01 started |
| 2026-11-07 | `progress` | `done` | Tech Lead (agent) | All deliverables verified; independent review APPROVED; AC3/AC4/AC6 downstream verification owned by WP-03, WP-13/WP-14 and WP-21 per the WP contract |
| 2026-11-07 | `done` | `done` | Tech Lead (agent) | WP review: review plan created, gates reproduced; QA first pass `CHANGES_REQUESTED` (compose coverage, ADR Status sections, AC3 consistency) then fixed and re-reviewed |

---

## 1. Objective

Freeze the **architecture, service boundaries, data ownership and communication contracts** for AI-Lab so that every WP builds against the same topology without reinterpretation. Deliver the C4 model, the service catalog, the bounded-context map, the inter-service contract matrix, the data-ownership matrix, the failure-mode matrix and the architecture decision records (ADRs).

---

## 2. Scope

### 2.1 In scope

- C4 Level 1 (context), Level 2 (containers), Level 3 (components) for the backend and frontend.
- Bounded contexts and their aggregates, repositories, events and owners.
- Deployment units (“services”) and their responsibilities, scaling and failure behavior.
- Communication patterns: in-process calls, HTTP at the edge, Postgres outbox, Redis pub/sub, runner control plane.
- Data ownership and forbidden cross-context access.
- Architectural constraints, quality attributes and evolution path.
- ADRs 0001–0023 (ADR-0024 is owned by WP-02 and linked from the architecture doc as part of the full registry).

### 2.2 Out of scope

- Code implementation (WP-06…WP-16).
- Schema DDL detail (WP-04; ADR-level only here).
- Infrastructure manifests (WP-03).

---

## 3. Engineering specification

### 3.1 Architectural style (binding decision)

AI-Lab v1 is a **modular monolith deployed as multiple runtime services**, not a distributed microservice mesh.

- **Logical boundaries** are microservice-grade: five bounded contexts with explicit contracts, data ownership and events.
- **Physical deployment** is intentionally consolidated into a small number of runtime units (`web`, `api`, `worker`, `runner`) to satisfy Lean/ops constraints, reduce failure modes, keep transactions local and make the single-operator product operable.
- **Rule:** no context may read or write another context's tables. Cross-context reads go through an application query port; cross-context side effects go through domain events/outbox.
- **Evolution:** because boundaries are contract-first, any context can later be extracted into an independent service without changing consumers. Extraction requires an ADR and a contract-compatibility test.

> This decision is ADR-0002 and is non-negotiable for v1. A proposal to split into independently deployed microservices per context is out of scope and requires a CR.

### 3.2 C4 Level 1 — System context

```
┌──────────────────────────────────────────────────────────────────────┐
│                        HUMAN OPERATOR (browser)                      │
│   Dashboard · Lab (pixel office) · Kanban · Inventory · SOW · Memory │
└───────────────────────────────┬──────────────────────────────────────┘
                                │ HTTPS + WSS (loopback by default)
┌───────────────────────────────▼──────────────────────────────────────┐
│                              AI-LAB                                  │
│  Idea → SOW → WBS → Kanban → Agent execution → Review → Merge        │
│  → Release → Maintenance → Learning (CV + memory)                    │
└───────┬───────────────────────────┬───────────────────┬──────────────┘
        │                           │                   │
┌───────▼────────┐         ┌────────▼────────┐  ┌───────▼─────────────┐
│ DeepSeek API   │         │ GitHub API      │  │ Docker Engine       │
│ LLM inference  │         │ repos/PRs/CI    │  │ sandbox containers  │
└────────────────┘         └─────────────────┘  └─────────────────────┘
```

**External dependencies:** DeepSeek (LLM), GitHub (SCM), Docker Engine (sandbox). No other runtime dependency. Package registries are used only at build time (and inside the sandbox under an explicit egress policy).

### 3.3 C4 Level 2 — Containers (deployment units)

| Service | Runtime | Responsibility | Owns data? | Scale | Failure behavior |
|---|---|---|---|---|---|
| `web` | Nginx + static Angular | SPA, TLS termination (optional profile), reverse proxy `/api` + `/ws` | No | N stateless | 502/503 with pixel error page; retry |
| `api` | Rust/axum | REST, WS hub, validation, projections, auth/exposure, enqueue | No (uses Postgres) | N stateless | Health fails; LB removes; client retries |
| `worker` | Rust/tokio | Scheduler, leases, run pipeline, relay, reflection, maintenance | No (uses Postgres/Redis) | N (lease-based) | Leases expire; runs re-queued; no duplicate execution |
| `runner` | Rust + Docker API | Create/destroy ephemeral sandbox containers; stream logs | No | N node-local | Runs fail transiently; retried; no host escape |
| `postgres` | PostgreSQL 18 + pgvector | Sole durable system of record | **Yes** | 1 primary (+ optional replica) | API readiness fails; writes rejected; read-only mode only if configured |
| `redis` | Redis 8.8 | Cache, rate limits, locks, presence, pub/sub | No | 1 (v1) | Degraded mode: no cache; rate limits fail closed; WS fan-out falls back to DB polling |
| `migrate` | sqlx-cli | One-shot schema migration | No | run-once | Boot blocked until success |
| `otel-collector` | optional | Trace/metric/log pipeline | No | 1 | Telemetry dropped; product unaffected |
| `prometheus`/`grafana` | optional | Metrics + dashboards | No | 1 | Observability degraded only |

**Networks:** `edge` (web↔api), `backbone` (api/worker↔postgres/redis), `runner` (worker↔runner), `observability` (optional). `postgres` and `redis` are never exposed on the host in production.

### 3.4 C4 Level 3 — Components (backend)

```
api (axum)
├── http/            routers, extractors, middleware (request-id, tracing, timeout, body limit)
├── dto/             request/response types, validation, OpenAPI annotations
├── auth/            exposure-mode guard, bearer token, origin check
├── ws/              connection registry, subscribe, backpressure, heartbeat
├── projections/     read models for board, dashboard, inspector
└── composition/     dependency wiring (repos, services, providers)

worker (tokio)
├── scheduler/       eligibility, lease acquisition, dispatch
├── orchestrator/    run pipeline state machine, agent loop, handoff
├── tools/           tool registry, policy engine, executors
├── relay/           outbox → Redis pub/sub → WS fan-out
├── reflection/      CV/memory updates after runs
├── maintenance/     cron jobs, compaction, health, stale cards
└── recovery/        lease expiry, run reconciliation, dead-letter

shared crates
├── domain/          entities, value objects, invariants, events (no I/O)
├── application/     use cases, ports, transactions, authorization
├── infrastructure/  sqlx repos, redis, blob store, clock, ids
├── llm/             LlmProvider port + DeepSeek adapter + prompt assembly
├── scm/             ScmProvider port + GitHub adapter
├── memory/          embeddings, chunking, retrieval, compaction
└── telemetry/       tracing, metrics, OTel
```

### 3.5 Bounded contexts

| Context | Code prefix | Aggregates | Owns tables | Publishes | Consumes |
|---|---|---|---|---|---|
| **Inventory** | `inv` | Provider, Model, Agent, CV, Project, ScmCredential | `providers`, `models`, `agents`, `agent_cv_versions`, `projects`, `scm_credentials`, `characters` | `inventory.*` | none |
| **Lab** | `lab` | Lab, TeamMembership, LabSettings | `labs` | `lab.*` | `inventory.agent.*` |
| **Knowledge/Memory** | `mem` | MemoryDocument, Chunk, Episode | `memory_documents`, `memory_chunks`, `memory_episodes` | `memory.*` | `delivery.run.completed` |
| **Kanban** | `kan` | Board, Card, CardEvent, Comment | `boards`, `cards`, `card_events`, `card_comments` | `kanban.*` | `delivery.*` |
| **Delivery** | `dlv` | SOW, SowVersion, Plan, PlanTask, Run, RunStep, Decision | `sows`, `sow_versions`, `plans`, `plan_versions`, `plan_tasks`, `runs`, `run_steps`, `decisions` | `delivery.*` | `kanban.card.moved` |
| **SCM integration** (adapter) | `scm` | WebhookDelivery | `github_webhook_deliveries` | `delivery.pr.*` through Delivery | GitHub webhooks |
| **Platform** | `plat` | Workspace, AuditEntry, Outbox, DomainEvent, Settings | `workspaces`, `audit_log`, `outbox`, `domain_events`, `app_settings`, `maintenance_jobs`, `cost_ledger`, `llm_requests` | `system.*` | all |

**Ownership rule:** a table has exactly one owning context. Other contexts may hold a `*_id` reference but MUST NOT join or mutate the owned table. Reporting views may join; they are read-only and owned by `plat`. The five core domain contexts are Inventory, Lab, Knowledge/Memory, Kanban and Delivery; SCM integration is an adapter context that owns only webhook idempotency state.

### 3.6 Data ownership matrix (normative)

| Table | Owner | Readers (via ports/views) | Writers |
|---|---|---|---|
| `workspaces` | Platform | Lab, Inventory, Kanban, Delivery, API | Platform |
| `labs` | Lab | Inventory, Kanban, Delivery, API | Lab |
| `providers`, `models` | Inventory | LLM, API | Inventory |
| `agents`, `agent_cv_versions`, `characters` | Inventory | Lab, Kanban, Delivery, Memory, Office | Inventory |
| `projects`, `scm_credentials` | Inventory | Kanban, Delivery, SCM | Inventory |
| `boards`, `cards`, `card_events`, `card_comments` | Kanban | Delivery, API, Office | Kanban |
| `sows`, `sow_versions`, `plans`, `plan_versions`, `plan_tasks` | Delivery | Kanban (materialization), API | Delivery |
| `runs`, `run_steps`, `decisions` | Delivery | Kanban (projection), Memory (reflection), API | Delivery |
| `memory_documents`, `memory_chunks`, `memory_episodes` | Memory | Delivery (retrieval), API | Memory |
| `outbox`, `domain_events` | Platform | Relay, API | Any context (same transaction) |
| `audit_log`, `app_settings`, `cost_ledger`, `llm_requests`, `maintenance_jobs` | Platform | API, Ops | Platform |
| `github_webhook_deliveries` | SCM integration | Platform | SCM |

### 3.7 Communication matrix

| From → To | Mechanism | Contract | Idempotency | Timeout |
|---|---|---|---|---|
| Browser → `web`/`api` | HTTPS REST | OpenAPI 3.1 | `Idempotency-Key` on POST | 30 s (120 s SOW) |
| Browser ↔ `api` | WSS | AsyncAPI 3.0 | `event_id` + `seq` | heartbeat 20 s |
| `api` → contexts | In-process application services | Rust traits | transaction | — |
| Any context → `outbox` | Same DB transaction | `outbox` row | `event_id` | — |
| `relay` → Redis | Pub/Sub | event envelope | `event_id` dedupe | 5 s |
| `relay` → WS clients | In-process fan-out | AsyncAPI | `seq` monotonic | 250 ms p95 |
| `worker` → `postgres` | sqlx | SQL | lease/unique constraints | 5 s read / 30 s write |
| `worker` → `runner` | Private HTTP + shared token | Runner API (§3.8) | `run_id` | 30 s create / stream |
| `worker` → DeepSeek | HTTPS | OpenAI-compatible | request ID | 120 s |
| `worker` → GitHub | HTTPS | REST v3 | idempotency key | 30 s |
| GitHub → `api` | Webhook | HMAC | `X-GitHub-Delivery` | 5 s ack |

**No direct `api` ↔ `worker` HTTP calls.** They coordinate exclusively through Postgres (state + outbox) and Redis (ephemeral fan-out). This keeps the system crash-safe and avoids distributed transactions.

### 3.8 Runner control-plane API (normative)

The runner exposes a private HTTP API on the `runner` network, authenticated by `AILAB_RUNNER_TOKEN` (≥32 bytes), bound to the internal interface only.

| Method | Path | Purpose | Request | Response |
|---|---|---|---|---|
| POST | `/v1/sandboxes` | Create sandbox for a run | `{run_id, image, worktree_ref, cpu, memory_mb, timeout_s, egress}` | `{sandbox_id, container_id, created_at}` |
| GET | `/v1/sandboxes/{id}` | Status | — | `{status, exit_code, started_at, ended_at}` |
| GET | `/v1/sandboxes/{id}/logs?follow=true` | Stream logs (SSE) | — | log chunks, 20 MB cap |
| POST | `/v1/sandboxes/{id}/exec` | Run an allowlisted command | `{argv, cwd, timeout_s, env_allowlist}` | `{exit_code, stdout, stderr, duration_ms}` |
| DELETE | `/v1/sandboxes/{id}` | Kill and remove | `{reason}` | `204` |
| GET | `/health/live` `/health/ready` | Liveness/readiness | — | `200/503` |

**Rules:** sandbox creation is idempotent by `run_id`; a second create returns the existing sandbox. The runner never mounts the Docker socket into a sandbox. All calls are logged with `run_id` and `trace_id`.

### 3.9 Transaction and consistency model

- **Within a context:** ACID transaction in Postgres.
- **Across contexts:** eventual consistency via outbox + idempotent consumers. No 2PC.
- **Read models:** API projections are recomputed from events or queried directly; they never become the source of truth.
- **Optimistic concurrency:** `version` column on mutable aggregates; `409 VERSION_CONFLICT` with current state.
- **Leases:** `SELECT ... FOR UPDATE SKIP LOCKED`; lease TTL 900 s; heartbeat 15 s.
- **Event ordering:** `seq` per lab assigned in the same transaction as the domain event; clients apply in order and ignore `seq <= last_applied`.

### 3.10 Failure-mode matrix

| Failure | Detection | Product behavior | Recovery |
|---|---|---|---|
| Postgres down | readiness, pool errors | API returns 503; worker stops; WS sends `system.health.changed` | Restart; reconcile runs; relay resumes |
| Redis down | ping, command errors | Cache bypassed; rate limits fail closed; WS fan-out degraded; product read/write continues except limits | Reconnect; rebuild ephemeral state |
| DeepSeek down | 5xx/timeouts, circuit breaker | Runs queue/pause; UI shows provider degraded; no data loss | Retry with backoff; resume queue |
| GitHub down | 5xx/rate limit | PR steps pause; local branch work continues; queue | Backoff; polling fallback; resume |
| Runner down | health, create timeout | Runs fail transiently; retried; card stays `DOING` with blocker | Restart runner; requeue |
| Worker crash | lease expiry, heartbeat miss | Run marked `timed_out`; card requeued with backoff | Reconcile from DB + PR state |
| Web down | LB health | UI unavailable; API/worker continue | Restart; client resumes WS with `seq` |
| Disk full | host alert | Writes fail; graceful errors; read-only mode if possible | Prune workspaces/artifacts; expand; restore |
| Clock skew | server-authoritative timestamps | Clients cannot affect ordering | NTP; server timestamps only |

### 3.11 Quality attributes and constraints

| Attribute | Target | Design mechanism |
|---|---|---|
| Performance | API read p95 ≤ 120 ms; office 60 FPS | Stateless API, indexed queries, projections, canvas engine |
| Scalability | 20 concurrent runs; 100 WS clients; 2k cards | Worker replicas, leases, WS backpressure, pagination |
| Availability | 99.9% single-node target | Restart-safe stateless services, outbox, reconciliation |
| Consistency | Strong within context; eventual across | Transactions + outbox + idempotency |
| Security | No secret exposure; sandbox isolation | WP-05/WP-14 controls |
| Operability | `make up`, `make doctor`, `make restore-drill` | WP-03/WP-20 |
| Evolvability | Context extraction without consumer change | Ports/adapters, contract tests |
| Testability | Every layer testable without external APIs | Fakes, wiremock, testcontainers, deterministic seed |

### 3.12 ADRs to deliver

`0001` monorepo · `0002` modular monolith + bounded contexts · `0003` Postgres sole system of record · `0004` outbox + `SKIP LOCKED` · `0005` DeepSeek first provider · `0006` local embeddings · `0007` Canvas 2D office · `0008` Angular zoneless · `0009` server-enforced Kanban · `0010` fixed roles/characters · `0011` separation of duties · `0012` sandboxed runner · `0013` login-less local-first · `0014` WS resumable seq · `0015` structured outputs · `0016` three-tier memory · `0017` budgets · `0018` autonomy levels · `0019` commit trailers · `0020` Docker Compose · `0021` office engine decoupled · `0022` version pinning/Renovate · `0023` runner control-plane API.

Each ADR follows the template in WP-00 and is stored under `.docs/adr/` (CR-0001).

---

## 4. Deliverables (checklist)

- [x] `.docs/product/architecture.md` with C4 L1/L2/L3 diagrams (Mermaid source + SVG), trust boundaries and data flows.
- [x] `.docs/product/domain-model.md` with bounded contexts, aggregates, invariants and event flows.
- [x] `.docs/product/service-catalog.md` with responsibilities, scaling, data ownership, failure behavior and SLOs.
- [x] `.docs/product/contracts.md` with the communication matrix and runner control-plane API.
- [x] `.docs/product/data-ownership.md` with the ownership matrix and forbidden-access rules.
- [x] `.docs/product/failure-modes.md` with the failure matrix and recovery procedures.
- [x] `.docs/product/diagrams/context.mmd`, `containers.mmd`, `components-backend.mmd`, `components-frontend.mmd` + exports (4 SVGs, fresher than sources).
- [x] ADR-0001 … ADR-0023 committed and linked from the architecture doc.
- [x] Contract tests: `.tools/check-architecture-fitness.sh` enforces ownership, acyclicity and cross-context table access; self-test 15/15; wired in CI.
- [x] Architecture review record signed by Tech Lead and QA: `.docs/product/evidence/WP-01-review.md` (independent `APPROVED`).

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP01-T01 | Context/container/component diagrams | Analyst | 3 | WP-00 | Diagrams render; exports committed |
| WP01-T02 | Bounded contexts + aggregates + invariants | Tech Lead | 3 | T01 | Matches spec §6 |
| WP01-T03 | Service catalog + scaling/failure matrix | Dev | 2 | T02 | Every service has owner, scale, failure behavior |
| WP01-T04 | Data ownership + forbidden-access rules | Dev | 2 | T02 | Matrix complete; enforced by test |
| WP01-T05 | Communication + runner control-plane contracts | Dev | 2 | T03 | Contracts match WP-12/13/14 |
| WP01-T06 | ADR-0001..0023 | Tech Lead | 3 | T02 | Each ADR accepted or superseded |
| WP01-T07 | Architecture fitness tests (no cross-context access, acyclic deps) | QA | 2 | T04 | CI fails on violation |
| WP01-T08 | Architecture review + sign-off | Tech Lead | 1 | All | Review record |

---

## 6. Acceptance criteria

1. Every bounded context has exactly one owner, a defined aggregate set, owned tables and published/consumed events.
2. A static fitness test fails if a repository in context A references a table owned by context B (except through an approved read view).
3. The service catalog covers every container in `docker compose config`, with scaling and failure behavior.
4. The runner API contract is implemented by WP-14 and consumed by WP-13 without drift (contract test).
5. Every ADR listed in §3.12 exists, follows the template, and is linked from `.docs/product/architecture.md`.
6. The failure-mode matrix is exercised by at least one chaos test per row owned by WP-21.
7. C4 diagrams build from source and are embedded in the docs; no stale exported image.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence links.
- [x] Architecture review signed by Tech Lead and QA (`.docs/product/evidence/WP-01-review.md`, independent `APPROVED`).
- [x] Fitness tests green in CI (`.tools/check-architecture-fitness.sh` in `docs-build` and `rust-integration`; self-test 15/15).
- [x] ADRs accepted (ADR-0001..0023 registered).
- [x] No unreconciled contract drift with WP-04, WP-12, WP-13, WP-14, WP-16: this document set is the canonical source they consume; compatibility tests land with those WPs.
- [x] Status `done` with sign-off (2026-11-07).

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Boundaries erode into a big ball of mud | High | Fitness tests; ownership matrix; ADR-0002 |
| Over-distribution too early | High | Modular monolith decision; extraction requires ADR |
| Contract drift between WPs | High | Contract tests; owner WP bumps version; CI compatibility check |
| Hidden coupling via shared tables | High | Forbidden-access test; read-only reporting views only |
| Documentation diagrams go stale | Medium | Diagrams as source; CI builds; review checklist |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §4, §6, §13, §14, §15
- `../technical-specification/APPENDIX-B-CONTRACTS.md` §B.1–B.4
- `../technical-specification/APPENDIX-D-DELIVERY.md` §D.3
- `SOW.md` §7, §8, §11

## Progress log

2026-11-07 | T01 | .docs/product/diagrams/context.mmd, containers.mmd, components-backend.mmd, components-frontend.mmd + .svg | mermaid-cli render (4 SVGs) | — | T02
2026-11-07 | T02 | .docs/product/domain-model.md | doc link check | — | T03
2026-11-07 | T03 | .docs/product/service-catalog.md | doc link check | — | T04
2026-11-07 | T04 | .docs/product/data-ownership.md, .tools/architecture/contexts.tsv, dependencies.tsv | .tools/check-architecture-fitness.sh | — | T05
2026-11-07 | T05 | .docs/product/contracts.md | doc link check | — | T06
2026-11-07 | T06 | .docs/product/architecture.md (ADR index) | harness-lint + ADR registry | — | T07
2026-11-07 | T07 | .tools/check-architecture-fitness.sh, .tools/architecture-fitness-self-test.sh | self-test 7/7; real check 4/4 | — | T08
2026-11-07 | T08 | .docs/product/architecture.md review surface | pending independent review | — | review
2026-11-07 | review-fix | .docs/product/domain-model.md, .docs/product/architecture.md, .docs/product/data-ownership.md, .tools/check-architecture-fitness.sh, .tools/architecture/paths.tsv, .docs/adr/adr-0023.md | fitness self-test 15/15; harness/sow/doc links green | ADR-0023 | final review
2026-11-07 | T08 | .docs/product/evidence/WP-01-review.md | independent review APPROVED | — | closure
2026-11-07 | closure | all WP-01 artifacts | harness OK (16), sow OK (31), doc links 106, fitness 4/4 | AC3/AC4/AC6 downstream by WP-03/13/14/21 | WP-02
2026-11-07 | review | .docs/sow/WP-01-ARCHITECTURE-review-plan.md | harness 16, sow 31, doc links 139, fitness 4/4, self-test 15/15, crate fitness OK; QA first pass CHANGES_REQUESTED, fixes applied | compose coverage fix, ADR Status sections, catalog caddy | re-review
