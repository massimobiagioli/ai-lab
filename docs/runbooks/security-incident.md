# Runbook — Security Incident

| Field | Value |
|---|---|
| Document | `docs/runbooks/security-incident.md` |
| Severity | P0 or P1 |
| Owner | QA + Tech Lead |
| Related | `docs/security.md`, `docs/failure-modes.md` |

## 1. Detect

Symptoms: gitleaks finding, anomaly in `audit_log`, unexpected outbound traffic, provider key misuse, repeated 401/403, suspicious run steps, cross-scope memory hit.

Preserve evidence before changing anything: container logs, `audit_log` rows, `run_steps`, `github_webhook_deliveries`, trace IDs. Do not paste secrets into tickets.

## 2. Contain

1. Stop the affected service: `docker compose -f infra/compose/docker-compose.yml stop api worker runner`.
2. Revoke the affected credential: rotate the provider key, GitHub PAT or webhook secret in the provider console.
3. Rotate `AILAB_MASTER_KEY` and `AILAB_RUNNER_TOKEN` if either may be exposed.
4. Disable the affected provider or lab in the database/UI.
5. If the host is suspect, isolate it from the network.

## 3. Eradicate

- Patch the root cause (code, policy or configuration).
- Add a regression test or fixture.
- Re-scan: gitleaks, Trivy, dependency audit, sandbox escape suite.
- Confirm no persistence: forged tokens, new agents, new webhooks, modified autonomy.

## 4. Recover

1. Restore from the last trusted backup if data integrity is in doubt (`make restore-drill` first).
2. Re-enable services one at a time; verify `make ps` and the golden path.
3. Rotate all credentials that shared the compromised path.
4. Monitor `audit_log` and egress for 24 hours.

## 5. Review

- Post-incident report within 48 hours: timeline (UTC), root cause, contributing factors, detection, response, corrective actions with owners and dates.
- Update `docs/security/abuse-cases.md` and `docs/risk-register.md`.
- Create a `TODO` card for each corrective action; the Tech Lead owns the follow-up.

## 6. Tabletop checklist

- [ ] Simulate a leaked provider key; measure time to revoke and rotate.
- [ ] Simulate a forged webhook; confirm 401 and no state change.
- [ ] Simulate a sandbox egress attempt; confirm denial and log.
- [ ] Simulate cross-scope retrieval; confirm zero hits and a failing leakage test.
- [ ] Confirm the master key is not in the last backup.
- [ ] Confirm the incident report template is ready.
