# AI-Lab Flow Metrics

| Field | Value |
|---|---|
| Document | `docs/metrics.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Tech Lead |
| SQL | `migrations/0002_analytics_views.sql` |
| Verification | `.tools/test-analytics-views.sh` (25 assertions on fixed fixtures) |
| Related | `docs/governance.md`, `docs/status-model.md`, `docs/quality-gates.md` |

Metrics are derived from immutable events and persisted records. No metric is computed from UI-only state, self-reported numbers or estimates. The read-only view set lives in the `analytics` schema and is exposed at `GET /api/v1/labs/{id}/metrics` once the API lands (WP-16).

---

## 1. Data sources

| Source | Used for |
|---|---|
| `cards.created_at` | Lead time origin (card creation). |
| `card_events` (`from_column`, `to_column`, `event_type`, `actor_type`, `created_at`) | Ready time, done time, blocked intervals, rework, human intervention. |
| `cards.column_name` | Current WIP gauge. |
| `runs`, `run_steps.duration_ms` | Active work time and flow efficiency. |
| `cost_ledger.amount_microusd` | Cost per card; authoritative over duplicated run cost. |
| `agents.autonomy_level` | Current autonomy level for autonomous-success attribution. |

All timestamps are server-generated UTC. Money is integer micro-USD (1 USD = 1,000,000).

## 2. Metric definitions

| Metric | Formula | View | Target | Owner |
|---|---|---|---|---|
| Lead time | `done_at - card.created_at` | `analytics.card_lead_time` | p50 ≤ 2 h, p95 ≤ 8 h | Tech Lead |
| Cycle time | `done_at - first_ready_at` | `analytics.card_cycle_time` | p50 ≤ 45 min, p95 ≤ 4 h | Tech Lead |
| Throughput | completed cards per week | `analytics.throughput_weekly` | ≥ 20/week at steady state | Tech Lead |
| WIP | cards in `DOING` + `REVIEW` | `analytics.wip_current` | ≤ configured limits | Tech Lead |
| Blocked time | Σ `unblocked_at - blocked_at` | `analytics.blocked_intervals`, `analytics.blocked_time` | ≤ 10% of cycle time | Dev |
| Flow efficiency | active run seconds / lead seconds, clamped to `[0,1]` | `analytics.flow_efficiency` | ≥ 55% | Tech Lead |
| Rework rate | cards with `REVIEW → DOING` / cards done | `analytics.rework_rate` | ≤ 10% | QA |
| Cost per card | `Σ cost_ledger / done cards` | `analytics.card_cost`, `analytics.cost_per_card` | median ≤ USD 1.50 | Tech Lead |
| Autonomous success | A2 cards done with no human event READY→DONE / cards done | `analytics.autonomous_success` | ≥ 85% | Tech Lead |
| Escaped defects | P0/P1 found after `DONE` | not in SQL (issue tracker) | 0 | QA |
| Flow summary | percentile-50/95 lead and cycle time | `analytics.flow_summary` | see above | Tech Lead |

## 3. Definitions and edge cases

### 3.1 Lead and cycle time

- Lead time starts at `cards.created_at`, the canonical card record. A future `ideas` table may add an earlier `idea_received_at`; until then the card creation time is the documented proxy.
- Cycle time starts at the first `card_events` row whose `to_column = 'READY'`. A card that never reached `READY` is excluded.
- A card is done at the first `to_column = 'DONE'` event. Reopening creates a **new** card with a parent link, so the old card keeps its original done time.

### 3.2 Throughput

- Grouped by `date_trunc('week', done_at)`; weeks start on Monday.
- Includes every card that reached `DONE` in the week, regardless of the column it started in.

### 3.3 WIP

- Current snapshot, not historical. Grouped by lab, project, column and assignee.
- `DOING` and `REVIEW` only. `READY` work is not WIP in the Kanban sense used here.
- The Kanban engine enforces the limits; the view reports.

### 3.4 Blocked time

- Each `blocked` event is paired with the next `unblocked` event for the same card.
- `blocked_seconds_closed` sums only closed intervals. Open intervals are counted in `open_blocked_intervals` and are not folded into closed time, so the metric is deterministic.
- Blocked time as a share of cycle time is computed by joining `analytics.blocked_time` to `analytics.card_cycle_time` in the dashboard layer.

### 3.5 Flow efficiency

- Active time is the sum of `run_steps.duration_ms` for every run of the card, across attempts.
- Efficiency is clamped to `[0,1]`; values above 1 indicate overlapping or mis-recorded timestamps and are reported as a data-quality defect, not as 100% efficiency.
- Draft prompts and reflection steps count as active time only when they are recorded as `run_steps`.

### 3.6 Rework

- Numerator: distinct done cards with at least one transition `REVIEW → DOING`.
- Denominator: cards in `analytics.card_lead_time` (done cards).
- A card that returns multiple times counts once.

### 3.7 Cost per card

- `cost_ledger` is canonical; `runs.cost_microusd` is a convenience projection and is not summed to avoid double counting.
- Median uses `percentile_cont(0.5)`; average is `sum / count`.
- Sandbox and other ledger sources are included; the dashboard can filter `cost_ledger.source`.

### 3.8 Autonomous success

- A done card counts as autonomous when its assignee's current `autonomy_level = 2` and no human event (`moved`, `updated`, `assigned`, `commented`, `blocked`, `unblocked`) occurs between `READY` and `DONE`.
- **Limitation:** `agents.autonomy_level` is the current value, not a snapshot at run time. WP-13 must persist the autonomy level used by a run; until then this metric is an approximation and is labeled `experimental` on the dashboard.

### 3.9 Escaped defects

- Not derivable from the current schema because there is no issue-tracker table. Counted manually from the defect log and reported weekly by QA.
- The target is zero. Any escaped P0/P1 creates a retrospective action and a prevention card.

## 4. Verification

The views are verified against fixed fixtures on a disposable PostgreSQL + pgvector container:

```bash
.tools/test-analytics-views.sh
```

The script applies `migrations/0001_init.sql`, then `migrations/0002_analytics_views.sql`, then the fixture, and asserts 25 exact values covering lead time, cycle time, throughput, WIP, blocked time, flow efficiency, rework, cost, autonomy and flow percentiles. Exit code 0 is the evidence.

## 5. Dashboard mapping

| Dashboard panel | Views |
|---|---|
| Flow KPIs | `analytics.flow_summary`, `analytics.throughput_weekly` |
| WIP and limits | `analytics.wip_current` |
| Blocked and aging | `analytics.blocked_time`, `analytics.card_cycle_time` |
| Rework | `analytics.rework_rate` |
| Cost | `analytics.card_cost`, `analytics.cost_per_card` |
| Autonomy | `analytics.autonomous_success` |

The dashboard is read-only. It never writes to `analytics.*` or to the base tables.

## 6. Cadence

- The weekly flow review reads `analytics.*` on Monday at 10:00 UTC.
- Metrics older than the selected window are filtered by `done_at` or `week_start`; the views have no hidden global state.
- A metric definition change requires a CR and, when it changes a contract, an ADR.

## 7. References

- `.docs/sow/WP-00-GOVERNANCE.md` §3.7
- `.docs/sow/SOW.md` §5.3
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.4
- `.docs/technical-specification/APPENDIX-A-DATABASE.sql`
- `migrations/0002_analytics_views.sql`
- `.tools/test-analytics-views.sh`
