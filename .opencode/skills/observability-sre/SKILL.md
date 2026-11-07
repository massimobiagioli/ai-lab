---
name: Observability & SRE
description: Use when adding metrics, traces, logs, SLOs, alerts, dashboards, health checks, backup, disaster recovery or runbooks.
---

# Observability & SRE

## When to use

- A new component, route, job, run stage or dependency needs visibility.
- A metric, trace, log field, SLO, alert or dashboard changes.
- A health or readiness check changes.
- Backup, restore, retention or a disaster-recovery procedure changes.
- An incident requires a runbook or a post-incident review.

## Preconditions

- Read `.docs/sow/WP-20-OBSERVABILITY-SRE.md` and `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §17 and §18.
- Know the service level objective affected and the error budget.
- Know the alert receiver and the runbook location.

## Workflow

1. Define the signal: metric name and labels, span and attributes, or log fields.
2. Keep label cardinality bounded. Never use a UUID, card key or free text as a label.
3. Implement instrumentation with OpenTelemetry and Prometheus conventions.
4. Add a test that the signal is emitted with the expected fields.
5. If an SLO is affected, define or update the SLI query, target, window and burn-rate alert.
6. Every alert must link to a runbook. No alert without a runbook.
7. Add a dashboard panel provisioned as code.
8. Verify with a local stack:
   - `docker compose --profile observability up -d`
   - query the metric or trace in Prometheus or the collector;
   - confirm the alert rule parses.
9. For backup or restore, run the drill and record duration and checksums.
10. For an incident, write the post-incident report within 48 hours with three owned actions.

## Checklist

- [ ] Signal has bounded cardinality.
- [ ] No secret, prompt or PII in a log, metric or span.
- [ ] Correlation IDs propagate across services.
- [ ] SLO and error budget updated where relevant.
- [ ] Alert has a runbook link and a first action.
- [ ] Dashboard provisioned as code.
- [ ] Health and readiness checks are lightweight and debounced.
- [ ] Backup and restore drill recorded.
- [ ] Runbook updated.
- [ ] Evidence recorded.

## Anti-patterns

- Metric labels containing IDs or error messages.
- Logging full prompts or request bodies.
- Alerts without an owner or runbook.
- Sampling errors below 100 percent.
- Relying on manual dashboard edits.
- Backup without a restore drill.
- Health checks that call the LLM or a slow dependency on every probe.

## References

- `../../../.docs/sow/WP-20-OBSERVABILITY-SRE.md`
- `../../../.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §13.3, §17, §18
- `../../../.docs/technical-specification/APPENDIX-D-DELIVERY.md` §D.7

## Supporting files

- None.
