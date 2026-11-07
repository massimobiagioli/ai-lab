# AI-Lab Audit Coverage Matrix

| Field | Value |
|---|---|
| Document | `docs/security/audit-matrix.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | QA |
| Related | `docs/security.md`, `docs/database.md` |

Every sensitive action writes an audit row with actor type, actor id, action, target, safe before/after, IP, user agent and `trace_id`. Run and card actions also keep their immutable domain records.

| Action | Audit row (`audit_log.action`) | Domain record | Owner |
|---|---|---|---|
| Create/update/disable provider | `provider.create`, `provider.update`, `provider.disable` | `inventory.*` outbox | WP-07 |
| Verify provider | `provider.verify` | `llm_requests` | WP-07 |
| Create/retire agent | `agent.create`, `agent.retire` | `agents`, `inventory.agent.*` | WP-07 |
| Pause/resume agent | `agent.pause`, `agent.resume` | `lab.agent.status_changed` | WP-07 |
| Create/update project | `project.create`, `project.update` | `inventory.project.connected` | WP-07 |
| Create/rotate SCM credential | `scm_credential.create`, `scm_credential.rotate` | `scm_credentials` | WP-07 |
| Card create/update | `card.create`, `card.update` | `card_events` | WP-09 |
| Card transition | `card.move` | `card_events` | WP-09 |
| Card block/unblock | `card.block`, `card.unblock` | `card_events` | WP-09 |
| Assign/reassign | `card.assign` | `card_events` | WP-09 |
| Run start/end | `run.start`, `run.finish` | `runs`, `delivery.run.*` | WP-13 |
| Merge | `card.merge` | `delivery.merge.completed` | WP-15 |
| Review verdict | `card.review` | `delivery.review.submitted` | WP-15 |
| SOW approve | `sow.approve` | `sows`, `sow_versions` | WP-10 |
| Plan materialize | `plan.materialize` | `cards` | WP-10 |
| Memory delete | `memory.delete` | `memory.*` | WP-08 |
| Budget/autonomy change | `lab.budget`, `lab.autonomy` | audit only + CR | Operator |
| Exposure mode change | `security.exposure` | audit only | Operator |
| Master key rotation | `security.key_rotate` | audit only | Operator |
| Remote token use (failure) | `security.auth_denied` | audit only | WP-16 |
| Webhook rejection | `scm.webhook_rejected` | `github_webhook_deliveries` | WP-12 |

Rules:

1. Audit rows are append-only; no update or delete API exists.
2. Secrets are never written to `before`/`after`; the redactor runs before persistence.
3. A sensitive action without an audit row is a P1 defect.
4. The coverage matrix is tested action by action as the owning WP lands.
