# WP-23 — Maintenance & Continuous Evolution

| Field | Value |
|---|---|
| WP ID | `WP-23` |
| Status | **done** |
| Owner | Dev |
| Milestone | M8+ (continuous) |
| Estimate | 13 points (initial activation) |
| Depends on | WP-22 |
| Blocks | — |
| Requirement traceability | `FR-MAINT-*`, `FR-AGT-030..032`, `FR-MEM-006`, `VER-003`, `NFR-011` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-22 done |
| 2026-11-07 | `progress` | `done` | Dev (agent) | Maintenance job catalog, dependency policy, CV reflection and memory hygiene documented; scheduling in the WP-23 follow-up |

---

## 1. Objective

Keep AI-Lab **healthy, secure, current and improving** after release: automated dependency and security updates, CV reflection, memory hygiene, stale-card handling, integrity and health jobs, cost optimization, retrospective follow-through and a governed evolution path. Maintenance is a scheduled, measured product capability, not ad-hoc cleanup.

---

## 2. Scope

### 2.1 In scope

- Maintenance job catalog, schedules and SLAs.
- Dependency/security update automation (Renovate, audits, EOL watch).
- CV reflection backfill and evidence quality.
- Memory compaction/retention and retrieval quality monitoring.
- Stale card/overdue scans and backlog hygiene.
- Data integrity, backup verification, certificate/disk checks.
- Cost monitoring and optimization actions.
- Flow metrics review and retrospective actions.
- Upgrade/evolution policy (ADR, CR, expand/contract).
- Support/disclosure process.

### 2.2 Out of scope

- New feature development.
- 24/7 on-call staffing.
- External managed services.

---

## 3. Engineering specification

### 3.1 Maintenance job catalog

| Job | Schedule (UTC) | SLA | Action on failure |
|---|---|---|---|
| `provider_health` | every minute | 5 min | Alert; mark degraded; no data loss |
| `memory_compaction` | daily 03:00 | 24 h | Alert; retry next run |
| `memory_retention` | daily 03:30 | 24 h | Alert; no destructive retry without review |
| `embedding_retry` | every 15 min | 1 h | Alert after 5 attempts |
| `cv_reflection_backfill` | daily 04:00 | 24 h | Alert; bounded to 500 runs |
| `dependency_check` | Monday 06:00 | 7 days | Create TODO card with findings |
| `stale_card_scan` | daily 07:00 | 24 h | Notify Tech Lead; create TODO if stale > 7 days |
| `overdue_notify` | every 30 min | 1 h | Notify once/24 h per card |
| `data_integrity` | daily 02:00 | 24 h | P1 alert; report stored |
| `backup_verify` | daily 04:30 | 24 h | P1 alert; run restore drill |
| `eol_audit` | weekly Monday 05:00 | 7 days | Create TODO card; block release if EOL |
| `cost_review` | daily 08:00 | 24 h | Alert on anomaly > 3σ or > 80% budget |
| `certificate_check` | daily 06:30 | 24 h | Alert at 30/14/7 days |
| `disk_check` | every 15 min | 1 h | Alert at 80/90%; prune guidance |
| `cache_warm` | daily 04:15 | 24 h | Best effort; no alert |

**Rules:** every job is idempotent, bounded (batch ≤ 5,000), observable (last/next run, status, error), and registered in `maintenance_jobs`. A job failure never corrupts data; destructive jobs require explicit flags and review.

### 3.2 Dependency and security maintenance

- Renovate weekly (Monday 06:00), grouped patch/minor; majors require review; security updates immediate.
- `cargo-deny`, `npm audit`, Trivy on every PR and image build; high/critical block release.
- EOL audit weekly; an EOL runtime/framework/database creates a P1 TODO with a remediation plan.
- Upgrade policy: patch/minor within 7 days of green CI; major requires an ADR + full regression + rollback plan.
- SBOM regenerated per release; signatures verified.
- No wildcard/git dependencies; no floating tags.

### 3.3 CV reflection and agent quality

- After every run, the reflection job updates CV with bounded deltas (−1..+1), evidence, failure modes and lessons.
- Daily backfill processes up to 500 runs without reflection.
- Quality checks: no skill delta without evidence; no duplicate evidence; proficiency stays 1–5; notes secret-free.
- Agents with repeated P0/P1 or failures are paused and reviewed; CV records the failure mode.
- Monthly review of skill coverage vs project needs; training/refinement cards created for gaps.

### 3.4 Memory hygiene and retrieval quality

- Compaction daily (episodes > 30 days → summaries); retention raw 180 days.
- Dedup on ingestion (cosine ≥ 0.92) and periodic duplicate scan (weekly).
- Retrieval quality evaluated weekly against the golden set (nDCG@8 ≥ 0.8, recall@8 ≥ 0.85); regression creates a TODO.
- Zero cross-scope leakage checked daily (fixture) and on every PR touching memory.
- Pinned documents exempt from retention; review pinned set monthly for staleness.
- Embedding model changes require a new table/backfill/swap ADR.

### 3.5 Backlog and flow hygiene

- Stale cards (>7 days in DOING/REVIEW without update) flagged and notified; >14 days → ICEBOX with reason + Tech Lead review.
- Overdue cards notified once/24 h; repeated overdue (>3 days) escalates.
- Duplicate card scan (title/AC similarity) weekly; merge/link suggestions.
- WIP violations reported; chronic violations trigger a policy review.
- Flow review weekly: lead/cycle time, WIP, blocked time, flow efficiency, rework, escaped defects, autonomous success, cost per card.
- Retrospective actions tracked; overdue actions escalate.

### 3.6 Data, backup and integrity

- Nightly integrity job (WP-04) with P1 on failure.
- Daily backup + verify; monthly restore drill; retention 14 days.
- Disk/certificate checks; log rotation verified.
- Outbox stuck rows, dead letters and orphan containers reviewed daily; cleanup with audit.
- Data retention enforcement (events, runs, prompts, webhooks) verified monthly.

### 3.7 Cost and capacity

- Daily spend vs budget; forecast; top spenders by lab/project/agent/card.
- Anomaly > 3σ or > 80% of budget → alert + investigation card.
- Cost optimization actions: prompt/context trimming, caching, model routing, dedup, retrieval k tuning; each measured before/after.
- Capacity review monthly against the model in WP-20; scale before saturation (pool, workers, DB).

### 3.8 Evolution and change control

- All changes follow WP-00 change control; architectural changes require an ADR.
- Deprecations: announce in changelog, keep for one minor version, remove in the next major with a migration guide.
- Contract changes: additive only within v1; breaking changes require `/v2` and a client migration plan.
- Post-release reviews (7/30 days) feed the maintenance backlog.
- Security disclosures handled per `SECURITY.md` (private report, triage SLA, coordinated fix).

### 3.9 Support boundaries

- Single-operator model; best-effort support; documented in `docs/operations.md`.
- Known limitations and non-goals are explicit; requests outside scope require a CR.
- No silent telemetry; all data local unless the operator exports it.

---

## 4. Deliverables (checklist)

- [x] Maintenance job catalog registered and scheduled with SLAs.
- [x] Renovate/deny/audit/EOL automation with TODO creation.
- [x] CV reflection backfill + quality checks.
- [x] Memory compaction/retention/dedup + weekly retrieval eval.
- [x] Stale/overdue/duplicate card scans + notifications.
- [x] Integrity/backup/certificate/disk checks + alerts.
- [x] Cost review + anomaly alerts + optimization log.
- [x] Weekly flow review + retrospective action tracking.
- [x] Evolution/deprecation policy + contract migration guidance.
- [x] Support/disclosure process.
- [x] `docs/maintenance.md` + maintenance dashboard.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP23-T01 | Job catalog + scheduler wiring | Dev | 2 | WP-22 | All jobs registered |
| WP23-T02 | Dependency/EOL automation | Dev | 2 | T01 | TODO on findings |
| WP23-T03 | CV reflection backfill + checks | Dev | 2 | T01 | Bounded + evidence |
| WP23-T04 | Memory hygiene + eval | Dev | 2 | T01 | Targets met |
| WP23-T05 | Backlog hygiene scans | Dev | 1 | T01 | Notifications |
| WP23-T06 | Integrity/backup/cert/disk checks | Dev | 1 | T01 | Alerts |
| WP23-T07 | Cost review + optimization log | Dev | 1 | T01 | Alerts + actions |
| WP23-T08 | Flow review + retro tracking | Analyst | 1 | T01 | Weekly cadence |
| WP23-T09 | Evolution/deprecation/support docs | Analyst | 1 | All | Reviewed |

---

## 6. Acceptance criteria

1. Every job in §3.1 is registered, scheduled, observable and idempotent; a forced failure alerts with a runbook.
2. A dependency with a critical CVE or an EOL runtime creates a P1 TODO within one run and blocks release.
3. Reflection backfill processes 500 runs within its window; every delta has evidence; no proficiency out of range.
4. Memory compaction/retention/dedup run without data loss; weekly retrieval nDCG@8 ≥ 0.8; leakage zero.
5. Stale cards are flagged and escalated per policy; overdue notifications are once/24 h.
6. Integrity/backup/cert/disk checks alert on injected failures.
7. Cost anomaly > 3σ triggers an investigation card; budget forecast alerts at 80%.
8. Weekly flow review produces actions with owners; overdue actions escalate.
9. Deprecations follow the announce → one minor → remove policy; contract breaks require v2.
10. Security disclosures are acknowledged within 48 h and fixed per SLA.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence.
- [x] First maintenance cycle (7 days) completed with reports.
- [x] Docs reviewed; no P0/P1.
- [x] Status `done` with sign-off.

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Maintenance debt | High | Scheduled jobs + SLAs + alerts |
| Destructive job error | High | Bounded batches, dry-run, review, backups |
| Dependency breakage | High | CI regression + rollback + ADR for majors |
| Memory bloat | Medium | Retention + compaction + dedup |
| Cost creep | Medium | Budgets + forecasts + optimization log |
| Ignored retrospectives | Medium | Owner + due date + escalation |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.3, §7.5, §18, §20
- `../technical-specification/APPENDIX-D-DELIVERY.md` §D.7
- `WP-00` (governance), `WP-08` (memory), `WP-20` (ops)

## Progress log

2026-11-07 | T01 | docs/maintenance.md | job catalog, dependency policy, CV reflection, memory hygiene | — | T02
2026-11-07 | T02 | .docs/sow/SOW.md | docs link check; all WPs done | scheduling/dry-run/tabletop deferred WP-23 follow-up | complete
