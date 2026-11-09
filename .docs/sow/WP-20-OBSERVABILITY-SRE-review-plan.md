# WP-20 — Observability, SRE & Operations Review Plan

| Field | Value |
|---|---|
| WP | `WP-20` |
| Contract | `.docs/sow/WP-20-OBSERVABILITY-SRE.md` |
| Review branches | `review/p08-wp20-metrics`, `review/p09-wp20-tracing`, `review/p10-wp20-dr` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_progress` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks and references present and consistent.
- [x] CR-0017 reopens WP-20; the history and progress log record it.

## 2. Specification conformance

- [x] Prometheus text metrics with requests, 5xx, in-flight and latency buckets — `crates/api/src/metrics.rs`, `/metrics`.
- [x] Redacted structured request logs (bearer/token/api_key) — `metrics::redact` and the request middleware.
- [x] W3C traceparent/x-trace-id propagation with trace ids in Problem Details and logs — P09; OTel span export depends on WP-13B T03.
- [x] SLOs (`slo.yml`), Grafana dashboard and alert rules (`alerts.yml`) — P09, validated by `.tools/check-observability.sh`.
- [x] Backup/restore scripts and the incident/DR/rollback runbooks — P10 (`.docs/product/runbooks.md`); the drill is executed by the operator/CI on the Compose stack.
- [ ] Health/readiness component checks — WP-13B T10.

## 3. Deliverables

- [x] `/metrics` Prometheus endpoint and metrics middleware — P08.
- [x] Structured JSON request logs with redaction — P08.
- [x] Trace propagation and the SLO/dashboard/alert artifacts — P09.
- [x] Backup/DR/runbooks — P10.
- [ ] Operations evidence and QA — P10.

## 4. Acceptance criteria

1. [x] Metrics expose requests, errors, in-flight and latency buckets — test `metrics_endpoint_renders_prometheus`.
2. [x] Logs are structured and never contain bearer/token/api_key values — test `redacts_tokens`.
3. [ ] OTel span export links LLM/SCM/sandbox spans — WP-13B T03; request trace propagation is delivered.
4. [x] SLOs/alerts/dashboards exist and are validated — `.tools/check-observability.sh`.
5. [x] Backup/restore scripts and the drill procedure are documented; execution is operator/CI-owned.
6. [x] Runbooks cover incidents, backup/restore and rollback — P10.

## 5. Implementation and tests

- [x] Metrics registry unit tests and the `/metrics` route test — `crates/api/src/metrics.rs`, `server.rs`.
- [x] No real external API.
- [ ] Coverage — WP-24 T06.

## 6. Security and quality gates

- [x] No secrets in logs (redaction), metrics contain no secrets and trace ids are non-sensitive.
- [x] `harness-lint`, `sow-lint`, `make check`, frontend gates green on P08.
- [ ] Full gate run on P10.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/observability.md` updated with `/metrics` and redaction.
- [x] CR-0017 and this review plan indexed in `AGENTS.md`.
- [x] `.docs/sow/SOW.md` and the WP status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED` on P10.
- [ ] QA evidence recorded.
- [ ] Every blocking defect fixed.

## 9. Deferred items and residual risks

- [ ] OTel exporter — owner: Dev, card: `WP-13B` T03, due 2026-12-31.
- [ ] Live load/chaos evidence — owner: QA, card: `WP-24` T06, due 2026-12-31.

## 10. Decisions

- [x] CR-0017 — reopen WP-20.
2026-11-09 | P09 | crates/api/src/{trace,server,metrics}.rs, infra/observability/**, .tools/check-observability.sh | trace propagation; SLOs/dashboard/alerts; validator | OTel exporter WP-13B T03 | P10
2026-11-09 | P10 | .docs/product/runbooks.md, scripts/{backup,restore-drill}.sh | incident/DR/rollback runbooks; drill procedure | operator drill execution | QA
