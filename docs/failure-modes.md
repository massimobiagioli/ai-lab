# AI-Lab Failure Modes and Recovery

| Field | Value |
|---|---|
| Document | `docs/failure-modes.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Source of truth | `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §18 |
| Related | `docs/service-catalog.md`, `.docs/sow/WP-20-OBSERVABILITY-SRE.md`, `.docs/sow/WP-21-TESTING.md` |

The product degrades safely. No failure may corrupt the system of record, duplicate a run, expose a secret or leave a card in an invalid state. Detection is automated; recovery is deterministic and documented.

---

## 1. Failure matrix

| Failure | Detection | Product behavior | Recovery | Owner |
|---|---|---|---|---|
| Postgres down | readiness probe, pool errors | API returns 503; worker stops; WS emits `system.health.changed` | Restart; reconcile runs; relay resumes | WP-04 |
| Redis down | ping, command errors | Cache bypassed; rate limits fail closed; WS fan-out degraded; reads/writes continue | Reconnect; rebuild ephemeral state | WP-04 |
| DeepSeek down | 5xx/timeouts, circuit breaker | Runs queue or pause; UI shows provider degraded; no data loss | Backoff and resume | WP-11 |
| GitHub down or rate limited | 5xx, `X-RateLimit-Remaining` | PR steps pause; local branch work continues; queue | Backoff; polling fallback; resume | WP-12 |
| Runner down | health, create timeout | Runs fail transiently and retry; card stays `DOING` with a blocker | Restart runner; requeue | WP-14 |
| Worker crash | lease expiry, heartbeat miss | Run `timed_out`; card requeued with backoff | Reconcile from DB and PR state | WP-13 |
| Web down | load balancer health | UI unavailable; API/worker continue | Restart; client resumes WS by `seq` | WP-03 |
| Disk full | host alert, write errors | Writes fail with graceful errors; optional read-only mode | Prune workspaces/artifacts; expand; restore | WP-20 |
| Clock skew | server-authoritative timestamps | Clients cannot affect ordering | NTP; server timestamps only | WP-20 |
| Budget exhausted | ledger check before each LLM call | Hard stop; run stops safely; `system.budget.exhausted` emitted | Operator raises budget or pauses lab | WP-11 |
| Secret scan finding | `security-secrets` gate | Merge blocked; P0 path | Revoke, rotate, purge, reopen | WP-05 |
| Sandbox escape attempt | policy violation, egress deny | Command blocked; event logged; run fails | Incident review; patch policy | WP-14 |
| Memory scope leak | leakage fixture fails | CI blocks; P0 | Fix SQL scope filter; re-run leakage suite | WP-08 |
| Stale card | maintenance scan | Card flagged overdue; not moved automatically | Operator or agent refinement | WP-23 |

## 2. Recovery procedures

| Scenario | Step 1 | Step 2 | Step 3 |
|---|---|---|---|
| Postgres restart | Verify readiness and migrations | Reconcile `running` runs against leases | Replay outbox from the last published cursor |
| Redis restart | Verify cache/rate-limit behavior | Rebuild presence from DB projections | Confirm WS clients resume by `seq` |
| Worker crash loop | Inspect lease and heartbeat | Mark stale runs `timed_out` | Requeue cards with backoff; stop after max attempts |
| Stuck run | Check heartbeat and sandbox | Cancel the sandbox and lease | Move the card to `ICEBOX` or retry with a reason |
| GitHub outage | Queue PR steps | Keep local branches and commits | Resume when the rate limit recovers |
| Disk pressure | Stop non-essential jobs | Prune worktrees and old artifacts | Expand volume; verify restore drill |
| Budget exhaustion | Pause affected runs | Emit and display the budget event | Operator raises the budget or closes the card |

Detailed runbooks live in `docs/runbooks/` (WP-20): `provider-outage.md`, `db-restore.md`, `stuck-runs.md`, `budget-exhausted.md`, `github-rate-limit.md`, `disk-full.md`.

## 3. Recovery objectives

- **RPO:** ≤ 24 h with daily backups; ≤ 5 min with WAL archiving (recommended).
- **RTO:** ≤ 1 h.
- A `make restore-drill` command must exist and pass before release (WP-20/WP-22).
- Data-integrity job nightly: FK orphan checks, card column vs latest event, run status vs steps; report-only, no automatic destructive repair.

## 4. Idempotency and crash safety

1. State changes and outbox writes are in the same transaction.
2. Leases use `SELECT ... FOR UPDATE SKIP LOCKED` with a 900 s TTL and a 15 s heartbeat.
3. Consumers deduplicate by `event_id`; clients apply events by `seq`.
4. Sandbox creation is idempotent by `run_id`.
5. GitHub writes use idempotency keys; webhooks deduplicate by delivery ID.
6. A restart never duplicates a run or loses a committed state change.

## 5. Chaos and resilience testing

WP-21 owns the chaos suite. Every row in §1 that is marked testable has at least one scenario: Postgres stop, Redis stop, worker kill, runner stop, provider timeout, rate-limit simulation, disk pressure simulation, budget exhaustion. A failure-mode row without a test is a release gap.

## 6. References

- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §18
- `.docs/sow/WP-20-OBSERVABILITY-SRE.md`
- `.docs/sow/WP-21-TESTING.md`
- `docs/service-catalog.md`
