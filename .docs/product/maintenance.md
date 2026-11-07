# AI-Lab Maintenance

| Field | Value |
|---|---|
| Document | `.docs/product/maintenance.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/operations.md`, `.docs/product/memory.md`, `.docs/product/observability.md`, `.docs/product/release-checklist.md` |

Maintenance keeps the lab healthy after release: dependency hygiene, memory compaction, CV reflection, stale-card scans, integrity reports and provider health.

## 1. Job catalog

| Job | Cron (UTC) | Behavior | Owner |
|---|---|---|---|
| `data_integrity` | `0 2 * * *` | Report-only integrity checks; P1 on any finding | QA |
| `memory_compaction` | `0 3 * * *` | Summarize episodes > 30 days, delete raw > 180 days unless pinned | Dev |
| `cv_reflection_backfill` | `0 4 * * *` | Rebuild missing CV evidence, bounded to 500 runs | Dev |
| `dependency_check` | `0 6 * * 1` | Renovate/EOL audit; create `TODO` cards for findings | Dev |
| `stale_card_scan` | `0 7 * * *` | Flag cards idle > 7 days; never auto-move | Tech Lead |
| `overdue_notify` | `*/30 * * * *` | Emit overdue nudges and badges | Dev |
| `provider_health` | `* * * * *` | Probe DeepSeek; update status and emit health events | Dev |

All jobs are idempotent, observable, bounded and non-destructive unless explicitly marked.

## 2. Dependency and supply-chain policy

- Renovate weekly; grouped patch/minor; majors need review; TypeScript major blocked (ADR-0024).
- Security patches within 48 h; no EOL dependency; lockfiles committed; SBOM and signatures at release.
- `cargo-deny`, `npm audit`, gitleaks and Trivy remain release gates.

## 3. CV reflection

- On `delivery.run.completed`, reflection updates the agent CV with evidence links.
- Deltas are bounded; CV versions are append-only and immutable.
- A retro action or incident can add a failure mode; it is never silently removed.

## 4. Memory hygiene

- Daily compaction and 180-day raw retention; pinned episodes are preserved.
- Deduplicate at cosine ≥ 0.92 within the same scope.
- Cross-scope leakage is a P0; the leakage fixture runs in CI.
- Re-embedding on model change uses a new table, backfill and swap.

## 5. Stale cards and overdue work

- Stale cards are flagged for refinement; the engine never moves them automatically.
- Overdue cards get an `overdue` badge and an agent nudge, never an automatic transition.
- Reopening a done card creates a new card with a parent link.

## 6. Health and cleanup

- Orphan sandbox containers are reconciled on runner boot.
- Worktrees and artifacts are pruned; 14 daily backups retained.
- Disk, queue depth, leases and budget alerts are defined in `.docs/product/observability.md`.

## 7. Tests and follow-ups

The job catalog is seeded by `migrations/0001_init.sql`. Scheduling and execution are owned by WP-20/WP-23; the follow-up adds dry-run, job metrics and a tabletop restore drill.

## 8. References

- `.docs/sow/WP-23-MAINTENANCE.md`
- `.docs/technical-specification/APPENDIX-A-DATABASE.sql` (maintenance_jobs seed)
- `.docs/product/memory.md`, `.docs/product/observability.md`, `.docs/product/release-checklist.md`
