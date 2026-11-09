# AI-Lab Operations Runbooks

| Field | Value |
|---|---|
| Document | `.docs/product/runbooks.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev / SRE |
| Related | `.docs/product/operations.md`, `.docs/product/observability.md`, `.docs/product/failure-modes.md` |

Each runbook lists the detection signal, the first checks, the mitigation and the escalation.

## 1. Edge high error rate (AilabHighErrorRate)

- **Detect:** `ailab_http_errors_total` / `ailab_http_requests_total` > 1% for 10 min.
- **Check:** `GET /metrics`, the structured logs for `status>=500`, the last deploy SHA and the DB/Redis/runner health.
- **Mitigate:** roll back the last release if the rate started with it; otherwise restart the failing dependency; keep the edge read-only if the DB is unavailable.
- **Escalate:** Tech Lead if the rate stays above 1% after rollback.

## 2. Edge high latency (AilabHighLatency)

- **Detect:** p95 `ailab_http_request_duration_ms` > 300 ms for 10 min.
- **Check:** in-flight gauge, Postgres slow queries, connection pool saturation, runner latency.
- **Mitigate:** shed load with the rate limiter, increase the pool only after confirming the DB, disable the expensive endpoint class.
- **Escalate:** Tech Lead if p95 stays above 300 ms after mitigation.

## 3. No traffic (AilabNoTraffic)

- **Detect:** `rate(ailab_http_requests_total[10m]) == 0` for 15 min.
- **Check:** the API process, the listener bind, the proxy, the load balancer and DNS.
- **Mitigate:** restart the API, verify the exposure/bind configuration, fail over to the standby.
- **Escalate:** Tech Lead + operator.

## 4. Database unavailable

- **Detect:** readiness `NOT_READY`, the API 5xx rate, `SELECT 1` failures.
- **Check:** the `postgres` container, disk, connections, the latest migration.
- **Mitigate:** restart Postgres, free disk, restore from the latest backup if corrupt.
- **Escalate:** Tech Lead; declare an incident if the RPO is at risk.

## 5. Redis unavailable

- **Detect:** rate limiter failures (fail closed), fan-out stops.
- **Check:** the `redis` container and the `AILAB_REDIS_URL`.
- **Mitigate:** restart Redis; the edge fails closed (429) until Redis returns; the in-process fallback keeps single-instance serving.
- **Escalate:** Tech Lead if the failure exceeds 15 min.

## 6. Runner unavailable

- **Detect:** sandbox create failures, `TOOL_TIMEOUT`, run failures at the implement stage.
- **Check:** the runner process, the Docker socket proxy, the runner token.
- **Mitigate:** restart the runner; requeue the affected runs with backoff; confirm the label reaper removed orphan containers.
- **Escalate:** Tech Lead; pause the lab if the runner is down for more than 30 min.

## 7. Backup and restore

- **Backup:** `make backup` writes `artifacts/backups/<UTC>.dump`, the artifact tarball and the manifest (SHA-256, git revision, migration version).
- **Restore drill:** `make restore-drill BACKUP=artifacts/backups/<UTC>.dump` restores into `ailab_restore_drill`, compares row counts, verifies integrity and records the RTO, then drops the scratch DB.
- **RPO:** ≤ 24 h with daily backups; ≤ 5 min with WAL archiving. **RTO:** measured by the drill.
- **Retention:** 14 daily backups locally; copy off-host. Run the drill monthly and before every release.

## 8. Release rollback

- **Trigger:** a failed release or a regression above the error/latency SLO.
- **Steps:** stop the rollout, redeploy the previous signed image digest, restore the previous schema only when the migration is backward-incompatible, verify `/metrics` and `/health/ready`, publish the incident note.
- **Escalate:** Tech Lead; open a CR if the rollback changes the contract.
