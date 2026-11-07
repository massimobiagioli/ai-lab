# Retrospective — M0 Governance Slice (WP-00)

| Field | Value |
|---|---|
| Date | 2026-11-07 |
| Facilitator | Tech Lead |
| Participants | Tech Lead, Analyst, Dev, QA |
| Period | 2026-11-07 → 2026-11-07 |
| Scope | WP-00 delivery system, templates and automation |

This record covers the WP-00 governance slice of M0. The full M0 exit gate remains open until WP-01..WP-06, the WP-11 adapter skeleton and the WP-17 shell are `done`.

## 1. What went well

- Governance shipped as executable policy, not prose: `scripts/sow-lint.sh` passes an 11-case self-test covering status, index drift, dependencies, cycles, checklists, stale logs, links, CRs, required checks and template structure.
- Metrics are verified, not asserted: `.tools/test-analytics-views.sh` applies the normative schema and asserts 25 exact values on fixed fixtures.
- The split-root documentation convention was resolved through CR-0001 instead of silently choosing between `.docs/product/` and `.docs/`.
- Every rule in `.docs/product/governance.md` has a control and a measurement, satisfying Lean L-06 (built-in quality).

## 2. What did not go well

- Branch protection cannot be applied in this environment because there is no Git repository or GitHub remote. The configuration is delivered but the operator action is pending.
- CI gates for Rust, Angular, contracts and images are present as named jobs with manifest guards. Until WP-02 lands the toolchains, those jobs are inert.
- Flow metrics have no live data because the API, database runtime and seed do not exist yet. The views are verified against fixtures only.
- Independent review could not be performed by a second actor in this execution environment.

## 3. Metrics snapshot

| Metric | Value | Target | Source |
|---|---|---|---|
| Lead time p50 / p95 | not instrumented | ≤ 2 h / ≤ 8 h | `analytics.flow_summary` |
| Cycle time p50 / p95 | not instrumented | ≤ 45 min / ≤ 4 h | `analytics.flow_summary` |
| Throughput | not instrumented | ≥ 20/week | `analytics.throughput_weekly` |
| WIP | not instrumented | ≤ limits | `analytics.wip_current` |
| Blocked time share | not instrumented | ≤ 10% | `analytics.blocked_time` |
| Flow efficiency | not instrumented | ≥ 55% | `analytics.flow_efficiency` |
| Rework rate | not instrumented | ≤ 10% | `analytics.rework_rate` |
| Cost per card median | not instrumented | ≤ USD 1.50 | `analytics.cost_per_card` |
| Escaped P0/P1 | 0 | 0 | defect log |

Live values begin after WP-04 (database runtime), WP-16 (metrics endpoint) and WP-20 (dashboard) land.

## 4. Incidents

| Incident | Severity | Root cause | Prevention |
|---|---|---|---|
| None | — | — | — |

## 5. Actions — exactly three

| # | Action | Owner | Due date | TODO card |
|---|---|---|---|---|
| 1 | Apply the default-branch protection rule from `.github/branch-protection.json` as soon as a GitHub remote exists, and record the API response as evidence. | Tech Lead | 2026-11-20 | [AIL-101](../backlog/AIL-101.md) |
| 2 | Remove the manifest guards from the CI jobs once the Rust workspace and Angular app land, and make every required check unconditional. | Dev | 2026-11-30 | [AIL-102](../backlog/AIL-102.md) |
| 3 | Persist the autonomy level used by each run so `analytics.autonomous_success` stops relying on the current agent value. | Dev | 2026-12-15 | [AIL-103](../backlog/AIL-103.md) |

## 6. Decisions

| Decision | ADR or CR | Owner |
|---|---|---|
| Split documentation roots between `.docs/product/` and `.docs/` | CR-0001 | Tech Lead |
| Enforce the 16 required CI check names through branch protection | `.github/branch-protection.json`, `.docs/product/quality-gates.md` | Tech Lead |
| Treat the A2 attribution as experimental until per-run autonomy is persisted | AIL-103, `.docs/product/metrics.md` §3.8 | Dev |
