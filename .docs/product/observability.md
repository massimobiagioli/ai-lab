# AI-Lab Observability & SRE

| Field | Value |
|---|---|
| Document | `.docs/product/observability.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/operations.md`, `.docs/product/failure-modes.md`, `.docs/product/metrics.md`, `crates/telemetry/src/lib.rs` |

AI-Lab is observable by construction: structured logs, Prometheus metrics, OTLP traces and flow analytics derived from immutable events.

## 1. Logs

- JSON by default (`AILAB_LOG_FORMAT=json`).
- Fields: `trace_id`, `span_id`, `lab_id`, `project_id`, `card_id`, `agent_id`, `run_id`, `request_id`.
- No secrets and no full prompts; the redactor runs before persistence.

## 2. Metrics

Stable names live in `crates/telemetry/src/lib.rs`:

| Metric | Purpose |
|---|---|
| `ailab_http_request_duration_seconds` | API latency histogram |
| `ailab_http_requests_total` | Request/error counter |
| `ailab_db_pool_connections` | Pool saturation |
| `ailab_redis_latency_seconds` | Redis latency |
| `ailab_run_queue_depth` | Scheduler backlog |
| `ailab_lease_age_seconds` | Stale leases |
| `ailab_run_duration_seconds` | Run duration |
| `ailab_llm_tokens_total` | Token usage |
| `ailab_llm_cost_microusd` | Cost |
| `ailab_budget_remaining_microusd` | Budget |
| `ailab_ws_connections` | WebSocket connections |
| `ailab_event_lag_seconds` | Outbox/relay lag |
| `ailab_office_fps` | Office frame rate |

## 3. Traces

OTLP HTTP/gRPC from api, worker and runner to the collector. Sampling: 100% errors, 10% baseline, 100% run pipelines. Traces link API → use case → DB → outbox → worker → LLM → GitHub by `trace_id`.

## 4. SLOs and alerts

`infra/prometheus/alerts.yml` covers API p95, 5xx rate, queue backlog, stale leases, budget exhaustion and disk pressure. Targets (spec §17): read p95 ≤ 120 ms, write p95 ≤ 250 ms, WS event → render p95 ≤ 250 ms, office ≥ 58 FPS, restore drill ≤ RTO.

## 5. Health and readiness

`/health/live` and `/health/ready` on api/worker/runner; readiness checks DB, Redis, migrations and runner reachability. Provider and SCM health are exposed separately.

## 6. Backup and runbooks

`make backup`, `make restore-drill`, and the runbooks in `.docs/product/runbooks/`. RPO ≤ 24 h (≤ 5 min with WAL), RTO ≤ 1 h. The metrics dashboard is populated from the analytics views (WP-00) and reviewed weekly.

## 7. Tests

`cargo test -p ailab-telemetry` checks the stable metric contract. `make compose-validate` validates the Prometheus configuration and alert rules. Load, soak and chaos suites are WP-21.

## 8. Deferred to the WP-20 follow-up

- OpenTelemetry SDK/exporter wiring in api, worker and runner.
- Grafana dashboards and datasource provisioning.
- Postgres/Redis exporters and SLO burn alerts.
- Runbook execution drills.

## 9. References

- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §17, §18, §20
- `.docs/sow/WP-20-OBSERVABILITY-SRE.md`
- `.docs/product/metrics.md`, `.docs/product/failure-modes.md`
