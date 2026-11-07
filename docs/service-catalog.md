# AI-Lab Service Catalog

| Field | Value |
|---|---|
| Document | `docs/service-catalog.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Source of truth | `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §4, §17 |
| Related | `docs/architecture.md`, `docs/failure-modes.md`, `.docs/sow/WP-03-INFRASTRUCTURE.md` |

This catalog covers every runtime unit. WP-03 owns the Compose implementation; this document is the architecture contract that Compose must satisfy. When Compose gains a service, this catalog and the fitness check must be updated in the same change.

---

## 1. Services

| Service | Runtime | Responsibility | Owns data | Scale | Failure behavior | SLO | Owner WP |
|---|---|---|---|---|---|---|---|
| `web` | Nginx 1.29 + static Angular | Serve SPA, optionally terminate TLS, reverse proxy `/api` and `/ws` | No | N stateless | 502/503 with the pixel error page; client retries | 99.9% | WP-03, WP-17 |
| `api` | Rust 1.99 / axum 0.8 | REST, WS hub, validation, projections, exposure guard, enqueue | No (Postgres) | N stateless | Readiness fails; load balancer removes; clients retry | read p95 ≤ 120 ms, p99 ≤ 300 ms | WP-16 |
| `worker` | Rust / tokio | Scheduler, leases, run pipeline, relay, reflection, maintenance, recovery | No (Postgres/Redis) | N lease-based | Leases expire; runs re-queued; no duplicate execution | run overhead ≤ 2 s | WP-13 |
| `runner` | Rust + Docker API | Create/destroy ephemeral sandboxes; stream logs; exec allowlisted commands | No | N node-local | Runs fail transiently and are retried; no host escape | sandbox create ≤ 5 s p95 | WP-14 |
| `postgres` | PostgreSQL 18 + pgvector 0.8.7 | Sole durable system of record | **Yes** | 1 primary (+ optional replica) | API readiness fails; writes rejected; optional read-only mode | RPO ≤ 24 h (≤ 5 min with WAL), RTO ≤ 1 h | WP-04 |
| `redis` | Redis 8.8 | Cache, rate limits, locks, presence, ephemeral pub/sub | No | 1 (v1) | Cache bypassed; rate limits fail closed; WS fan-out degraded | degradable, not authoritative | WP-04 |
| `migrate` | sqlx-cli 0.9 | One-shot forward-only migration | No | run-once | Boot blocked until success | must complete before `api`/`worker` ready | WP-03, WP-04 |
| `docker-socket-proxy` | Proxy | Restrict Docker API methods for the runner | No | 1 | Runner cannot create sandboxes; runs fail transiently | — | WP-03 |
| `otel-collector` | OpenTelemetry Collector | Trace/metric/log pipeline | No | 1 (optional) | Telemetry dropped; product unaffected | best effort | WP-20 |
| `prometheus` | Prometheus | Metrics storage | No | 1 (optional) | Observability degraded only | best effort | WP-20 |
| `grafana` | Grafana | Dashboards | No | 1 (optional) | Observability degraded only | best effort | WP-20 |
| `egress-proxy` | Squid/Envoy (profile `egress`) | Allowlisted sandbox egress | No | 1 (optional) | Sandbox package fetches fail; product unaffected | — | WP-14 |

## 2. Networks and exposure

| Network | Members | Host exposure |
|---|---|---|
| `edge` | `web`, `api` | `web` only, `127.0.0.1:8080` in the default profile |
| `backbone` | `api`, `worker`, `migrate`, `postgres`, `redis` | none |
| `runner` | `worker`, `runner`, `docker-socket-proxy` | none |
| `observability` | `otel-collector`, `prometheus`, `grafana` | Grafana `127.0.0.1:3001` when the profile is active |

Postgres and Redis are never published in production. Sandboxes never join `edge` or `backbone`.

## 3. Startup and health

Startup order: `postgres`, `redis` → `migrate` → `api`, `worker` → `runner` → `web`. Optional observability starts independently.

| Endpoint | Service | Meaning |
|---|---|---|
| `GET /health/live` | `api`, `worker`, `runner` | process alive |
| `GET /health/ready` | `api` | migrations applied, Postgres reachable, Redis reachable |
| `GET /health/ready` | `worker` | Postgres and Redis reachable; scheduler loop running |
| `GET /health/ready` | `runner` | Docker API reachable through the socket proxy |
| `GET /api/v1/health/providers` | `api` | DeepSeek provider status |
| `GET /api/v1/health/scm` | `api` | GitHub reachability and rate-limit headroom |

Graceful shutdown: stop accepting work, drain ≤ 20 s, checkpoint or finish runs, release leases, flush telemetry.

## 4. SLOs

| SLO | Target | Measurement |
|---|---|---|
| API read latency | p95 ≤ 120 ms, p99 ≤ 300 ms | Prometheus histogram |
| API write latency | p95 ≤ 250 ms, p99 ≤ 600 ms | Prometheus histogram |
| Board load with 500 cards | p95 ≤ 400 ms | k6 + Playwright |
| WS event → render | p95 ≤ 250 ms | tracing + Playwright |
| Office frame rate | ≥ 58 FPS desktop, ≥ 30 low-end | RAF probe |
| Initial load | LCP ≤ 2.5 s, TTI ≤ 3.5 s on 4G | Lighthouse CI |
| Run overhead | ≤ 2 s per run excluding LLM | worker metrics |
| DB query | p95 ≤ 50 ms | `pg_stat_statements` |
| API memory | ≤ 256 MB RSS steady | cAdvisor |

## 5. Compose coverage check

The fitness check `.tools/check-architecture-fitness.sh` validates the context and dependency manifests. When `compose.yaml` or `docker-compose.yml` exists (WP-03), the check also verifies that every declared service appears in this catalog and that every catalog service with `Owner WP` WP-03 exists in Compose. Until then the check reports the catalog as the planned set.

## 6. References

- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §4, §17
- `.docs/sow/WP-03-INFRASTRUCTURE.md`
- `.docs/sow/WP-13-ORCHESTRATION.md`, `.docs/sow/WP-14-SANDBOX-RUNNER.md`
- `.docs/sow/WP-20-OBSERVABILITY-SRE.md`
- `docs/failure-modes.md`
