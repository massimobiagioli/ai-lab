# WP-20 — Observability, SRE & Operations Review Plan

| Field | Value |
|---|---|
| WP | `WP-20` |
| Contract | `.docs/sow/WP-20-OBSERVABILITY-SRE.md` |
| Review branches | `review/p08-wp20-metrics`, `review/p09-wp20-tracing`, `review/p10-wp20-dr` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_review` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks and references present and consistent.
- [x] CR-0017 reopens WP-20; the history and progress log record it.

## 2. Specification conformance

- [x] HTTP metrics with method/status labels, seconds histogram, `_sum`/`_count`, errors and in-flight — `crates/api/src/metrics.rs`.
- [x] Redacted structured request logs with timestamp/service/version/request-id/trace-id — `server.rs`.
- [x] W3C `traceparent`/`x-trace-id` propagation echoed and attached to Problem Details — `crates/api/src/trace.rs`.
- [x] SLOs (9 SLIs), error-budget policy and alert rules with runbook links — `infra/observability/`.
- [x] Grafana provisioning (datasource + dashboard file) mounted in Compose — `infra/compose/docker-compose.observability.yml`.
- [x] Readiness debounce (3 consecutive failures) — `health_ready`.
- [x] Integrity P1 alert and the restore-drill report artifact.
- [x] Eight runbooks + on-call checklist — `.docs/product/runbooks/`.
- [x] `slo.md`, `observability.md`, `operations.md` updated.
- [ ] Full DB/Redis/queue/LLM/SCM/sandbox/domain/WS metric families and OTel span export — WP-13B T03 (owner Dev, due 2026-12-31).
- [ ] Seven Grafana dashboards, cost dashboard, post-incident template and the load/integrity/runbook tabletop — WP-24 T07 (owner QA, due 2026-12-31).

## 3. Deliverables

- [x] Logging conventions + redaction + correlation middleware (run-context fields land with WP-13B T03).
- [ ] All §3.2 metric families — HTTP families delivered; the rest WP-13B T03.
- [ ] OTel tracing/spans — WP-13B T03; W3C propagation delivered.
- [x] SLO definitions + error-budget policy (9 SLIs; pending metrics noted).
- [x] Alert rules with runbook links, including the integrity and budget alerts.
- [ ] Seven Grafana dashboards — the edge dashboard is provisioned; the rest WP-24 T07.
- [x] Health/readiness endpoints with debounce.
- [x] Integrity alert + restore-drill report (injected-mismatch test in WP-24 T07).
- [x] Backup/restore scripts + monthly drill procedure + stored report.
- [ ] Capacity/load/soak plan and thresholds — WP-24 T07.
- [ ] Incident process + post-incident template — runbooks/on-call delivered; template WP-24 T07.
- [x] Eight runbooks + on-call checklist.
- [ ] Cost dashboard — WP-24 T07; budget alert delivered.
- [x] `operations.md`, `slo.md`, `observability.md`.

## 4. Acceptance criteria (contract §6)

1. **FAIL (deferred WP-13B T03 for run context).** Logs are structured and redacted, but lab/project/card/agent/run ids are not yet in the general request log.
2. **FAIL (deferred WP-13B T03).** HTTP metric families are labeled and in seconds; the DB/Redis/queue/LLM/SCM/sandbox/domain/WS families depend on the WP-13B T03 pipeline. `/metrics` is internal-only.
3. **DEFERRED (WP-13B T03).** W3C propagation is delivered; span export is WP-13B T03.
4. **PASS.** SLO definitions, error-budget policy and alert rules exist with runbook links; `.tools/check-observability.sh` validates the alert/SLO names.
5. **PASS (edge).** The edge dashboard is provisioned with the Prometheus datasource; the remaining six dashboards are WP-24 T07.
6. **PASS.** Readiness debounces after three consecutive failures.
7. **PARTIAL (WP-24 T07).** The integrity alert and report exist; the injected-mismatch test is WP-24 T07.
8. **PASS.** The restore drill verifies SHA/row counts/checksums/integrity, writes a stored report and enforces the RTO; execution is operator/CI-owned.
9. **DEFERRED (WP-24 T07).** Load/soak evidence is WP-24 T07.
10. **PASS (tabletop WP-24 T07).** The eight runbooks and the on-call checklist exist and are link-checked; the tabletop is WP-24 T07.

## 5. Implementation and tests

- [x] Metrics registry and `/metrics` route tests — `crates/api/src/metrics.rs`, `server.rs` (19 API tests).
- [x] Trace propagation tests — `crate::trace`.
- [x] No secrets in logs (redaction tests).
- [ ] Coverage — WP-24 T07.

## 6. Security and quality gates

- [x] No secrets in logs/metrics; trace ids are non-sensitive.
- [x] `harness-lint`, `sow-lint`, `check-doc-links`, `check-observability`, `make check` green.
- [ ] Full gate run on the WP-20 QA pass.

## 7. Documentation, ADRs and indexes

- [x] `observability.md`, `slo.md`, `operations.md` and the runbooks updated/indexed.
- [x] CR-0017 and this review plan indexed in `AGENTS.md`.
- [x] `.docs/sow/SOW.md` and the WP status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED`.
- [ ] QA evidence recorded after the review.
- [ ] Every blocking defect fixed.

## 9. Deferred items and residual risks

- [ ] Full metric families, OTel spans, run-context log fields — owner: Dev, card: `WP-13B` T03, due 2026-12-31.
- [ ] Six remaining dashboards, cost dashboard, post-incident template, load/soak, injected-integrity test, runbook tabletop — owner: QA, card: `WP-24` T07, due 2026-12-31.

## 10. Decisions

- [x] CR-0017 — reopen WP-20.
- [x] The edge dashboard and HTTP metrics are the P10 scope; the wider families/spans move to the named WP-13B/WP-24 follow-ups.
