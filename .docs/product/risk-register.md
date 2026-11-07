# AI-Lab Risk Register

| Field | Value |
|---|---|
| Document | `.docs/product/risk-register.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Tech Lead |
| Review cadence | Weekly, Monday 10:00 UTC (weekly flow review) |
| Next review | 2026-11-09 |
| Related | `.docs/product/governance.md`, `.docs/product/metrics.md`, `.docs/sow/SOW.md` §13 |

Every risk has an identifier, description, impact, likelihood, mitigation, owner and review date. A risk without a mitigation owner escalates to the Human Operator in the weekly review. A risk is retired only with a recorded rationale; it is never deleted.

Legend — impact/likelihood: `Critical`, `High`, `Medium`, `Low`. Owner is a role, not a person.

---

## 1. Program risks

| ID | Risk | Impact | Likelihood | Mitigation | Owner | Review date |
|---|---|---|---|---|---|---|
| R-01 | LLM output quality below bar | High | Medium | Structured outputs, schema validation, retries, escalation model, QA gate, human override | Tech Lead | 2026-11-09 |
| R-02 | Prompt injection via repository or memory | High | Medium | Untrusted-content wrapping, server-side tool policy, egress allowlist, secret scanning | QA | 2026-11-09 |
| R-03 | Runaway cost | High | Medium | Per-card/day/lab budgets, hard stops, circuit breaker, cost dashboard | Tech Lead | 2026-11-09 |
| R-04 | Stuck runs or agent loops | Medium | High | Bounded iterations and time, heartbeats, lease TTL, dead-letter, escalation | Dev | 2026-11-09 |
| R-05 | GitHub rate limits or outage | Medium | Medium | Backoff, queue, polling fallback, local branch work continues | Dev | 2026-11-09 |
| R-06 | Canvas performance on low-end devices | Medium | Medium | Internal 384×216, batching, offscreen caching, FPS telemetry, quality auto-tune | Dev | 2026-11-09 |
| R-07 | Data growth in events and vectors | Medium | Low | Retention, compaction, partitioning-ready schema, archival | Dev | 2026-11-09 |
| R-08 | Sandbox escape | Critical | Low | Ephemeral containers, no host mounts, capability drop, network none, seccomp | QA | 2026-11-09 |
| R-09 | Scope creep (auth, multi-tenant) | High | Medium | Explicit non-goals, CR gate, milestone discipline | Tech Lead | 2026-11-09 |
| R-10 | Version drift or EOL dependency | High | Medium | Pinned baseline, Renovate, EOL watch, upgrade ADR | Dev | 2026-11-09 |
| R-11 | Documentation drift | Medium | High | Docs-as-code, release gate, screenshot generation, link check | Analyst | 2026-11-09 |
| R-12 | Flaky tests erode trust | Medium | Medium | Quarantine within 24 h, fix or delete within 5 days, flaky blocks release | QA | 2026-11-09 |

## 2. WP-00 risks

| ID | Risk | Impact | Likelihood | Mitigation | Owner | Review date |
|---|---|---|---|---|---|---|
| R-WP00-01 | Governance becomes ceremony without effect | Medium | Medium | Every rule has a metric or CI enforcement; no rule without a control | Tech Lead | 2026-11-09 |
| R-WP00-02 | Status drift between index and WP files | Medium | Medium | `scripts/sow-lint.sh` in CI; one canonical field | Tech Lead | 2026-11-09 |
| R-WP00-03 | Over-process slows delivery | Medium | Medium | Lean rule L-08; minimal templates; automation over manual checks | Tech Lead | 2026-11-09 |
| R-WP00-04 | Metrics gamed by agents | Medium | Low | Metrics derive from immutable events; periodic audit; no self-reported numbers | QA | 2026-11-09 |
| R-WP00-05 | Bootstrap CI guards hide a missing gate | High | Low | Guards activate on manifest presence; WP-02 removes them; review checks the diff | QA | 2026-11-09 |
| R-WP00-06 | Branch protection not applied before a remote exists | Medium | Medium | `.docs/product/quality-gates.md` §7 and `.github/branch-protection.json` make the operator action deterministic | Tech Lead | 2026-11-09 |

## 3. WP risk rollup

Each WP owns its risks in `§8. Risks and mitigations` of its contract. This rollup tracks the review state. A WP with no owner or no review date in its own table is a defect caught in review.

| WP | Risk source | Owner | Review state |
|---|---|---|---|
| WP-00 | This document §2 | Tech Lead | reviewed 2026-11-07 |
| WP-01..WP-23 | `.docs/sow/WP-XX-*.md` §8 | WP owner | scheduled at WP activation |

## 4. Review process

1. The weekly flow review reads this register and every active WP's §8.
2. New risks are added with an owner and a review date.
3. A risk whose mitigation has not progressed for two reviews escalates to the Human Operator.
4. A retired risk records the rationale, the date and the evidence.
5. Incidents create or update risks and produce a post-incident report within 48 h.

## 5. Incident severity

| Severity | Definition | Response |
|---|---|---|
| P0 | Data loss, security breach, total outage | Immediate; Human Operator notified; release blocked |
| P1 | Major feature unavailable, merge blocked, gate bypass | 4 h; Tech Lead + Human Operator |
| P2 | Degraded behavior with a workaround | Next working day |
| P3 | Cosmetic | Planned |

## 6. References

- `.docs/sow/SOW.md` §13
- `.docs/sow/WP-00-GOVERNANCE.md` §3.10, §8
- `.docs/CONSTITUTION.md` §5, §7
- `.docs/product/quality-gates.md`
- `.docs/product/change-control.md`
