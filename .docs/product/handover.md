# AI-Lab Operational Handover

| Field | Value |
|---|---|
| Version | 1.0.0 |
| Status | Review (P14) |
| Related | `.docs/product/operations.md`, `.docs/product/runbooks.md`, `.docs/product/security.md` |

## First run

1. `make setup` then `make generate-key`, and fill the remaining `.env` values.
2. `make doctor` then `make reset` (builds, migrates and seeds the deterministic demo).
3. Open `http://127.0.0.1:8080`; the dashboard health badge must be green.
4. Run `.tools/chaos-worker-kill.sh` and one restore drill (`scripts/restore-drill.sh <dump>`) before accepting ownership.

## Daily checks

- Health: `/api/v1/health/live`, `/api/v1/health/ready`; the dashboard badge mirrors readiness.
- Queues and leases: the orchestration dashboard; no run stuck beyond the lease TTL.
- Alerts: the provisioned Grafana alert rules; triage with `.docs/product/runbooks/`.
- Backups: the nightly dump and the artifact archive; verify the manifest.

## Ownership matrix

| Area | Owner | Evidence |
|---|---|---|
| API, orchestrator, worker | Backend maintainer | `crates/`, WP-13/WP-13B |
| Web and office | Frontend maintainer | `web/`, WP-17/WP-18/WP-19/WP-24 |
| Database, migrations, retention | Data maintainer | `.docs/DATA-MODEL.md`, WP-04 |
| Security, secrets, exposure | Security owner | `.docs/product/security.md`, WP-05 |
| Observability, SLOs, backups | SRE owner | `.docs/product/slo.md`, WP-20 |
| Release and supply chain | Release owner | `.docs/product/release-checklist.md`, WP-22 |

## Known limitations

- Single-operator model; no 24/7 support commitment.
- The LLM and SCM adapters run with the in-memory fakes in the demo (`AILAB_DEV_FAKES=1`); real credentials are operator-supplied.
- WP-24 T01/T03 real-data surfaces and T07 observability items are open.
- Coverage is below the 80/90/80 thresholds; see `.docs/product/qa/WP-24-FRONTEND-HARDENING-qa-report.md`.
- The demo recording and some AC evidence are still pending (`.docs/product/acceptance-report.md`).

## Support boundaries

- Security disclosure follows `SECURITY.md`; do not open public issues for vulnerabilities.
- Best-effort maintenance only; no guaranteed response time.
- Data retention and deletion follow the retention rules in the technical specification.

## Post-release review

| Review | When | Owner |
|---|---|---|
| 7-day review | One week after release 1.0 | SRE owner |
| 30-day review | One month after release 1.0 | Tech Lead |

## Handover checklist

- [x] Operator guide and runbooks linked.
- [x] Ownership matrix and support boundaries stated.
- [x] Known limitations listed.
- [x] Restore drill and stuck-run recovery documented.
- [ ] Signed by the Tech Lead and the Human Operator.
