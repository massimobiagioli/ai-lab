# AI-Lab SLOs and Error Budgets

| Field | Value |
|---|---|
| Document | `.docs/product/slo.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | SRE |
| Related | `.docs/product/observability.md`, `.docs/product/runbooks.md`, `infra/observability/slo.yml` |

## 1. Service level objectives

| SLO | Target | Window | Indicator |
|---|---|---|---|
| Edge availability | 99.9% | 30d | `1 - 5xx/requests` |
| Edge latency (read) | p95 ≤ 120 ms | 7d | latency histogram |
| Edge latency (write) | p95 ≤ 250 ms | 7d | latency histogram |
| Run success | 95% | 7d | completed / (completed + failed) |
| Outbox lag | p95 ≤ 2 s | 1d | outbox-lag histogram |
| WS event latency | p95 ≤ 250 ms | 1d | WS delivery histogram |
| Cost per day | within budget | 1d | cost counter |
| Backup age | ≤ 24 h | 1d | backup timestamp |
| Memory retrieval | p95 ≤ 80 ms | 1d | retrieval histogram |

The edge SLOs use the `/metrics` HTTP series. The run, outbox, WebSocket, memory, cost and backup series are exported with the WP-13B T03
OTel/metrics pipeline; until then the definitions are declared and the dashboard panels are wired to
the HTTP series.

## 2. Error-budget policy

- The error budget is `1 - target` over the window (e.g., 0.1% for 99.9%).
- Freeze non-critical releases when the remaining budget reaches zero.
- Burn-rate alerts at 2% in 1 hour and 6% in 6 hours page the on-call.
- A post-incident review is required after any budget-exhausting incident.

## 3. Review cadence

Review the SLOs monthly; adjust only with a CR that records the customer and operational impact.
