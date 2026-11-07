# AI-Lab Governance and Delivery Model

| Field | Value |
|---|---|
| Document | `docs/governance.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Tech Lead |
| Source of truth | `.docs/sow/SOW.md`, `.docs/sow/WP-00-GOVERNANCE.md`, `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` |
| Related | `docs/status-model.md`, `docs/quality-gates.md`, `docs/metrics.md`, `docs/change-control.md`, `docs/risk-register.md` |

This document describes how AI-Lab work is selected, decomposed, executed, verified, measured and accepted. It is the product-repository companion to the program SOW: rules live here, the authoritative WP index and per-WP contracts live in `.docs/sow/`. Where this document and a higher source disagree, the higher source wins (`.docs/CONSTITUTION.md` §9).

---

## 1. Delivery authority

- The **Tech Lead** is the single delivery authority for ordering, decomposition, architecture and merge.
- The **Human Operator** is the final acceptor and the only actor who may change budget, autonomy or exposure mode.
- Agents act only inside an assigned card and role. No agent performs project work outside a card (`FR-KAN-003`).

## 2. Roles and RACI

| Activity | Tech Lead | Analyst | Dev | QA | Human Operator |
|---|---|---|---|---|---|
| SOW / WP decomposition | A | R | C | C | I |
| Architecture decisions (ADR) | A | C | R | C | I |
| Card refinement and DoR | A/R | C | C | C | I |
| Implementation | C | I | R | I | I |
| Independent verification | C | I | I | R/A | I |
| Merge | R/A | I | C | C | C |
| Budget / autonomy / exposure | C | I | I | I | A |
| Milestone acceptance | R | C | C | R | A |
| Release | R | C | R | R | A |

`R` responsible, `A` accountable, `C` consulted, `I` informed.

**Hard rule — separation of duties.** An actor MUST NOT review or merge its own work. This holds for humans and agents alike and is enforced server-side with `422 SEPARATION_OF_DUTIES` (ADR-0011).

## 3. Decision rights

| Decision | Authority | Consultation | Record |
|---|---|---|---|
| WP scope and decomposition | Tech Lead | Analyst, QA | WP file + SOW index |
| Architecture / technology | Tech Lead | Analyst, Dev, QA | ADR |
| Card refinement and assignment | Tech Lead | Dev, QA | `card_events` |
| Merge | Tech Lead or delegated merger | QA | PR + merge event |
| Budget / autonomy / exposure | Human Operator | Tech Lead | Audit log + CR |
| Release | Tech Lead | QA, Human Operator | Release record |
| Security exception | QA + Human Operator | Tech Lead | ADR with expiry date |

## 4. Lean operating rules (measurable)

Every rule has a control and a measurement. A rule without a control is removed rather than tolerated.

| Rule | Normative control | Measurement | Owner |
|---|---|---|---|
| L-01 Eliminate waste | One canonical location per artifact; scope diff in every PR; duplicate detection at DoR | Duplicate review; PR scope diff | Tech Lead |
| L-02 Limit WIP | Agent WIP = 1; program WIP ≤ 2 WPs in `progress`; `DOING` ≤ `max_concurrent_cards`, `REVIEW` ≤ 3 per QA | `analytics.wip_current`; `WIP_LIMIT_EXCEEDED` events | Tech Lead |
| L-03 Pull, don't push | Work starts only when dependencies are `done` and a WIP slot exists | Scheduler eligibility; WP status check | Tech Lead |
| L-04 Small batches | Cards ≤ 8 points; PR ≤ 1,500 changed lines and ≤ 40 files unless the Human Operator confirms | Card estimate; PR diff stats | Dev |
| L-05 Fast feedback | CI ≤ 15 min; unit ≤ 5 min; local `make check` ≤ 3 min | CI duration histogram | Dev |
| L-06 Built-in quality | Tests, security scans and contract checks on every PR; no hardening sprint | Required checks | QA |
| L-07 Optimize the whole | Weekly review of lead time, cycle time, WIP, blocked time, flow efficiency | `analytics.*` views | Tech Lead |
| L-08 Defer commitment | Estimates are relative; dates are forecasts; scope protected by explicit out-of-scope lists | Milestone variance review | Tech Lead |

## 5. Agile / Kanban flow

AI-Lab uses continuous flow, not sprints.

- The board has exactly six columns, ordered: `TODO`, `READY`, `DOING`, `REVIEW`, `DONE`, `ICEBOX` (`FR-KAN-001`).
- An agent may claim a `READY` card only if it is the assignee, has capacity, and the card's WP is `progress`.
- Column invariants, guards and the allowed transition matrix are defined in `.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.3 and enforced by the Kanban engine, not by the UI.
- Every transition writes an immutable `card_event`; the card's `column` is a projection of the latest event.
- WIP limits are enforced by the engine. The UI never grants an exception.

## 6. Cadence and ceremonies

All times are UTC.

| Ceremony | Trigger | Inputs | Outputs | SLA | Owner |
|---|---|---|---|---|---|
| Async daily | 09:00 on a working day | Board, blockers | Status note ≤ 5 lines | Same day | Each agent |
| Weekly flow review | Monday 10:00 | Metrics dashboard | Pull order, blocker actions | 30 min | Tech Lead |
| Milestone demo | Milestone gate | `done` WPs | Recording + acceptance evidence | 30 min | Tech Lead + QA |
| Retrospective | After a milestone | Metrics and incidents | Exactly three owned actions | 30 min | Tech Lead |
| Change control | CR raised | CR form | Decision and record | 1 working day | Tech Lead or Human Operator |
| Incident review | P0/P1 or security event | Timeline, logs | Post-incident report | 48 h | QA |

A retrospective produces exactly three actions. Each action has an owner, a due date and a `TODO` card. Actions older than one milestone without progress escalate to the Human Operator.

## 7. Escalation

| Trigger | First responder | Escalation | Maximum response |
|---|---|---|---|
| Card blocked | Assignee | Tech Lead | Same working day |
| WP blocked | Tech Lead | Human Operator | 1 working day |
| P1 defect or failed gate | QA | Tech Lead + Human Operator | 4 h |
| P0, security incident, data loss | QA | Human Operator | Immediate |
| Budget exhausted | Worker | Tech Lead + Human Operator | Immediate hard stop |

A blocker is recorded with category (`dependency`, `clarification`, `external`, `risk`, `budget`, `policy`), owner, impact, next action and ETA. `blocked` is a flag, never a status.

## 8. Status, DoR, DoD and evidence

`docs/status-model.md` is the canonical definition of `ready`, `progress`, `done`, the blocked flag, card/WP Definition of Ready and Definition of Done, the transition matrix and evidence rules. No status change is valid without an append-only history row and linked evidence.

## 9. Quality and security governance

`docs/quality-gates.md` maps every required CI check to an owner, threshold and failure action. Security gates are not optional and never bypassed for speed. Gate bypass is a P1 incident.

## 10. Change control

`docs/change-control.md` defines the CR form, approval matrix, emergency path and record location. A change request never deletes history; it supersedes.

## 11. Documentation and ADR standards

- ADRs follow `Context → Decision → Consequences → Alternatives → Status`.
- ADR identifiers are `adr-NNNN.md`; the registry is `.docs/adr/registry.md`.
- Accepted ADRs are immutable; a superseding ADR is created instead.
- Product documentation lives under `docs/`; the harness, SOW and normative specification live under `.docs/` (see CR-0001 in `docs/change-requests/CR-0001.md`).
- Diagrams are Mermaid or Excalidraw sources with committed exports.
- Screenshots and GIFs come from the deterministic seed and are refreshed with the UI.
- Broken links and a failing docs build block release.

## 12. Branch and commit policy

AI-Lab uses trunk-based development. `main` is the only long-lived branch and is protected by the required checks in `docs/quality-gates.md`.

### 12.1 Branches

- Branch from `main`: `ailab/{card_key}-{slug}` (for example `ailab/AIL-42-fix-retry-backoff`).
- Keep branches short-lived: aim for one card and one review cycle.
- No direct pushes to `main`; a pull request is mandatory.
- No force pushes or deletions on `main`. Feature branches may be rebased before review.
- Release cuts, when needed, are documented tags, not long-running branches.

### 12.2 Merge

- Squash merge only; the default branch keeps a linear history.
- Reviewer ≠ author and merger ≠ author (ADR-0011).
- The PR template in `.github/pull_request_template.md` must be complete, including evidence.
- A merged PR records the merge SHA on the card before the card moves to `DONE`.

### 12.3 Commits

Conventional Commits with an imperative summary:

```
type(scope): imperative summary
```

Allowed types: `feat`, `fix`, `chore`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`. The body explains why; the footer references the card.

Every agent commit carries the trace trailers (ADR-0019):

```
AI-Lab-Run: <run_uuid or "n/a">
AI-Lab-Card: <CARD-KEY>
AI-Lab-Agent: <agent_uuid>
```

Example:

```
feat(kanban): reject REVIEW to DONE without a merge SHA

The guard now returns a machine-readable violation and keeps the card in
REVIEW.

AI-Lab-Run: 0192f0a1-7c3e-7d4a-9b2f-1a2b3c4d5e6f
AI-Lab-Card: AIL-42
AI-Lab-Agent: 0b7c2f6e-3d1a-4c8e-9f21-5a6b7c8d9e0f
```

### 12.4 Tags and releases

- Pre-release tags follow `v0.x.0` per milestone; the release is `v1.0.0`.
- A tag is created only from a green `main` commit with an SBOM, signed images and an acceptance report.
- A release note links the milestone gate record and the changelog.

## 13. References

- `.docs/sow/SOW.md` §4–§13
- `.docs/sow/WP-00-GOVERNANCE.md`
- `.docs/CONSTITUTION.md`
- `.docs/PRINCIPLES.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.4, §21–§23
- `.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.3–§C.4
- `.docs/technical-specification/APPENDIX-D-DELIVERY.md` §D.1–§D.3
