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
| Edge latency (read) | p95 ≤ 120 ms | 30d | latency histogram |
| Edge latency (write) | p95 ≤ 250 ms | 30d | latency histogram |
| Run success | 99% | 7d | completed / (completed + failed) |
| Queue latency | p95 ≤ 60 s | 7d | queue-wait histogram |
| LLM error rate | ≤ 1% | 30d | LLM errors / requests |
| SCM error rate | ≤ 1% | 30d | SCM errors / requests |
| Sandbox failure | ≤ 1% | 30d | sandbox failures / starts |
| Memory retrieval | p95 ≤ 500 ms | 7d | retrieval histogram |

The edge SLOs use the `/metrics` HTTP series. The run, queue, LLM, SCM, sandbox and memory series are
exported with the WP-13B T03 OTel/metrics pipeline; until then the definitions are declared and the
dashboard panels are wired to the HTTP series.

## 2. Error-budget policy

- The error budget is `1 - target` over the window (e.g., 0.1% for 99.9%).
- Freeze non-critical releases when the remaining budget reaches zero.
- Burn-rate alerts at 2% in 1 hour and 6% in 6 hours page the on-call.
- A post-incident review is required after any budget-exhausting incident.

## 3. Review cadence

Review the SLOs monthly; adjust only with a CR that records the customer and operational impact.
