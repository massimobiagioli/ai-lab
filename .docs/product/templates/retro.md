# Retrospective — {Milestone or Event}

| Field | Value |
|---|---|
| Date | {YYYY-MM-DD} |
| Facilitator | Tech Lead |
| Participants | {roles} |
| Period | {start} → {end} |

## 1. What went well

- {observation with evidence}

## 2. What did not go well

- {observation with evidence}

## 3. Metrics snapshot

| Metric | Value | Target | Source |
|---|---|---|---|
| Lead time p50 / p95 | {value} | ≤ 2 h / ≤ 8 h | `analytics.flow_summary` |
| Cycle time p50 / p95 | {value} | ≤ 45 min / ≤ 4 h | `analytics.flow_summary` |
| Throughput | {value}/week | ≥ 20/week | `analytics.throughput_weekly` |
| WIP | {value} | ≤ limits | `analytics.wip_current` |
| Blocked time share | {value} | ≤ 10% | `analytics.blocked_time` |
| Flow efficiency | {value} | ≥ 55% | `analytics.flow_efficiency` |
| Rework rate | {value} | ≤ 10% | `analytics.rework_rate` |
| Cost per card median | {value} | ≤ USD 1.50 | `analytics.cost_per_card` |
| Escaped P0/P1 | {value} | 0 | defect log |

## 4. Incidents

| Incident | Severity | Root cause | Prevention |
|---|---|---|---|
| {id} | {P0..P3} | {cause} | {action} |

## 5. Actions — exactly three

| # | Action | Owner | Due date | TODO card |
|---|---|---|---|---|
| 1 | {action} | {owner} | {YYYY-MM-DD} | {CARD_KEY} |
| 2 | {action} | {owner} | {YYYY-MM-DD} | {CARD_KEY} |
| 3 | {action} | {owner} | {YYYY-MM-DD} | {CARD_KEY} |

An action without an owner, a due date and a card is invalid. Actions older than one milestone without progress escalate to the Human Operator.

## 6. Decisions

| Decision | ADR or CR | Owner |
|---|---|---|
| {decision} | {ADR-NNNN / CR-NNNN} | {owner} |
