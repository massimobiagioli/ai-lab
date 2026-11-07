# WP-00 — Program Governance, Lean/Agile Delivery & Quality System

| Field | Value |
|---|---|
| WP ID | `WP-00` |
| Status | **done** |
| Owner | Tech Lead |
| Milestone | M0 |
| Estimate | 8 points |
| Depends on | — |
| Blocks | All WPs |
| Requirement traceability | `NFR-006`, `NFR-011`, `NFR-012`, `DOC-001..003`, SOW §4–§13 |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WIP slot acquired; T01 started |
| 2026-11-07 | `progress` | `done` | Tech Lead (agent) | All tasks, deliverables and gates verified; independent review APPROVED; CR-0002 assigns environmental and consumer responsibilities |
| 2026-11-07 | `done` | `done` | Tech Lead (agent) | WP review: CR-0003 closes B-01; branch protection applied with the 16 required contexts and verified |

---

## 1. Objective

Establish a **binding delivery system** for AI-Lab: how work is selected, decomposed, executed, verified, measured and accepted. The system must make the delivery **predictable, auditable and lean**, and must make security and quality non-negotiable. This WP produces the rules, templates, metrics and automation that every other WP obeys.

This is not a documentation-only WP. It ships executable policy: the WP/card templates, the `sow-lint` checker, the flow-metrics queries, the change-control register and the milestone gate checklist.

---

## 2. Scope

### 2.1 In scope

- Lean and Agile operating model with measurable controls.
- Roles, RACI, decision rights and escalation.
- Card-level and WP-level Definition of Ready / Definition of Done.
- `ready` / `progress` / `done` status semantics and transition rules.
- Flow metrics, formulas, data sources and weekly review.
- Global quality gates and evidence rules.
- Change control, risk/issue management and retrospectives.
- Documentation and ADR standards.
- Templates and automation (`scripts/sow-lint`, `.github/pull_request_template.md`, card templates).

### 2.2 Out of scope

- Product features (covered by WP-07…WP-19).
- Tool installation (WP-02) and infrastructure (WP-03).
- Security implementation (WP-05); this WP defines the *governance* of security gates.

---

## 3. Engineering specification

### 3.1 Lean operating rules (measurable)

| Rule | Normative control | Measurement |
|---|---|---|
| Eliminate waste | Every artifact has one canonical location; no duplicate specs; no unrequested feature enters a card | Duplicate-detection review at DoR; scope diff in PR |
| Limit WIP | Agent WIP = 1; program WIP ≤ 2 WPs in `progress`; Kanban WIP limits enforced by engine | `wip_current` gauge; violation events |
| Pull | Work starts only when dependency is `done` and capacity exists | Scheduler eligibility; WP status check |
| Small batches | Cards ≤ 8 points; PR ≤ 1,500 lines and ≤ 40 files (else human confirmation) | Card estimate; PR diff stats |
| Fast feedback | CI ≤ 15 min; unit ≤ 5 min; local `make check` ≤ 3 min | CI duration histogram |
| Built-in quality | Tests/security/contract checks on every PR; no “quality sprint” | Required checks |
| Optimize whole | Weekly review of lead time, cycle time, WIP, blocked time, flow efficiency | Metrics dashboard |
| Defer commitment | Estimates relative; dates forecasts; scope protected by out-of-scope lists | Milestone variance review |

### 3.2 Agile / Kanban flow

- **No sprints.** Continuous flow with the six Kanban columns defined in the spec (`TODO`, `READY`, `DOING`, `REVIEW`, `DONE`, `ICEBOX`).
- **Pull policy:** an agent may claim a `READY` card only if it is the assignee, has capacity, and the card's WP is `progress`.
- **Cadence:** async daily, weekly flow review, milestone demo, retrospective. Calendar is UTC.
- **Retrospective output:** exactly three actions, each with an owner, a due date and a `TODO` card ID. Actions older than one milestone without progress escalate to the Human Operator.

### 3.3 Roles and decision rights

| Decision | Authority | Consultation | Must be recorded |
|---|---|---|---|
| WP scope and decomposition | Tech Lead | Analyst, QA | WP file + SOW index |
| Architecture / technology | Tech Lead | Analyst, Dev, QA | ADR |
| Card refinement and assignment | Tech Lead | Dev/QA | `card_events` |
| Merge | Tech Lead (or delegated merger) | QA | PR + merge event |
| Budget / autonomy / exposure | Human Operator | Tech Lead | Audit log + CR |
| Release | Tech Lead | QA, Human Operator | Release record |
| Security exception | QA + Human Operator | Tech Lead | ADR + expiry date |

**Separation of duties (hard rule):** an actor MUST NOT review or merge their own work. This applies to humans and agents. Violations are rejected server-side (`422 SEPARATION_OF_DUTIES`).

### 3.4 Ceremonies

| Ceremony | Trigger | Inputs | Outputs | SLA |
|---|---|---|---|---|
| Async daily | 09:00 UTC working day | Board, blockers | Status note (≤ 5 lines) | Same day |
| Weekly flow review | Monday 10:00 UTC | Metrics dashboard | Pull order, blocker actions | 30 min |
| Milestone demo | Milestone gate | `done` WPs | Recording + acceptance evidence | 30 min |
| Retrospective | After milestone | Metrics + incidents | 3 owned actions | 30 min |
| Change control | CR raised | CR form | Decision + record | 1 working day |
| Incident review | P0/P1 or security event | Timeline, logs | Post-incident report | 48 h |

### 3.5 Definition of Ready (card) and Definition of Done (card)

**Card DoR (all required before `READY`):**

1. Canonical user story complete (persona, capability, outcome, context).
2. ≥ 1 acceptance criterion in Given/When/Then form, each testable.
3. ≥ 1 concrete deliverable (path or named artifact with completion definition).
4. Estimate ∈ {1,2,3,5,8}; priority ∈ {must,should,could}; no `wont` in `READY`.
5. Assignee set and role-compatible; due date set and in the future.
6. Dependencies listed; each resolved or explicitly waived by the Tech Lead with reason.
7. WP is `progress`; card budget > 0; source recorded (`human|agent`).
8. `needs_refinement = false`.

**Card DoD (all required before `DONE`):**

1. PR merged to the default branch; merge SHA stored.
2. All required CI checks green; coverage for new code ≥ 80% or approved waiver.
3. Every acceptance criterion evidenced (test, command output, screenshot, benchmark).
4. Docs updated (README/ADR/API/runbook as applicable); changelog entry where user-visible.
5. No P0/P1 defects; security scans clean; no secret in diff.
6. QA verdict `APPROVED` for cards that change behavior; reviewer ≠ author.
7. Card log contains final summary, decisions and residual risks.
8. CV evidence appended by the reflection job.

### 3.6 WP status model and transitions

Only `ready`, `progress`, `done`. `blocked` is a flag with a category, owner, impact, next action and ETA. Transitions:

```
ready ──(owner assigned, WIP slot, first deliverable started)──► progress
progress ──(all deliverables checked, acceptance evidence, DoD, sign-off)──► done
done ──(approved change request only)──► progress
```

**CI enforcement (`scripts/sow-lint`):**

- Every WP file has a status line matching the index.
- Only the three allowed statuses appear.
- Every WP in `progress` has a progress log entry dated within the last 3 working days.
- Every `done` WP has all checklist items checked (`- [x]`) and a sign-off row.
- Every WP ID referenced in dependencies exists.
- No orphan WP (every WP is reachable from the index and the dependency graph is acyclic).
- Links in `SOW.md` resolve to existing files.

### 3.7 Flow metrics

| Metric | Formula | Source | Target |
|---|---|---|---|
| Lead time | `done_at - idea_received_at` | `card_events`, ideas | p50 ≤ 2 h, p95 ≤ 8 h |
| Cycle time | `done_at - ready_at` | `card_events` | p50 ≤ 45 min, p95 ≤ 4 h |
| Throughput | cards done / week | `card_events` | ≥ 20/week at steady state |
| WIP | cards in `DOING` + `REVIEW` | projections | ≤ limits |
| Blocked time | Σ `unblocked_at - blocked_at` | `card_events` | ≤ 10% of cycle time |
| Flow efficiency | active time / lead time | `run_steps` + events | ≥ 55% |
| Rework rate | cards returning from `REVIEW` to `DOING` / done | `card_events` | ≤ 10% |
| Escaped defects | P0/P1 found after `DONE` | issue tracker | 0 |
| Cost per card | Σ `cost_ledger` / done cards | ledger | ≤ USD 1.50 median |
| Autonomous success | cards done with autonomy A2 and no human intervention / done | runs | ≥ 85% |

Metrics are computed by a read-only SQL view set (`analytics.*`) and exposed at `GET /api/v1/labs/{id}/metrics`. No metric may be computed from UI-only state.

### 3.8 Global quality gates

Defined in SOW §10. This WP owns the enforcement mapping: each gate maps to a required CI check name; branch protection requires all of them. Gate bypass is a P1 incident.

Required check names (exact):

```
fmt
clippy
rust-unit
rust-integration
frontend-lint
frontend-unit
frontend-build
e2e
a11y
visual
openapi-compat
security-secrets
security-deps
security-image
docs-build
sow-lint
```

### 3.9 Change control

**CR form (exact fields):** `id`, `title`, `context`, `proposed_change`, `impact_scope`, `impact_schedule`, `impact_cost`, `impact_risk`, `alternatives`, `decision`, `approver`, `date`, `linked_adr`.

**Approval matrix:**

| Change | Approver |
|---|---|
| WP internal task reordering, no scope change | Tech Lead |
| New dependency within approved stack | Tech Lead |
| Architecture or contract change | Tech Lead + ADR |
| Scope change, milestone re-baseline, budget/autonomy/exposure | Human Operator |
| Security exception | QA + Human Operator, with expiry |

**Rules:** CRs never delete history; they supersede. Emergency security fixes may bypass pull order but must pass security gates and be retro-documented within 24 h.

### 3.10 Risk and issue management

- Program risks live in SOW §13; WP risks live in each WP.
- Every risk has `id`, `description`, `impact`, `likelihood`, `mitigation`, `owner`, `review_date`.
- Risks are reviewed weekly; a risk with no mitigation owner is escalated.
- Incidents: P0 = data loss/security breach/total outage; P1 = major feature unavailable/merge blocked; P2 = degraded; P3 = cosmetic.
- Post-incident report: timeline (UTC), root cause, contributing factors, detection, response, corrective actions with owners and dates.

### 3.11 Documentation and ADR standards

- ADR format: `Context → Decision → Consequences → Alternatives → Status`. Status ∈ `proposed|accepted|superseded|deprecated`. IDs are `NNNN-kebab-title`.
- Every WP updates the docs listed in its “References” section.
- Docs are Markdown, built with VitePress; broken links fail CI.
- Diagrams are Mermaid or Excalidraw source; exports are committed.
- No screenshots without a deterministic seed; no stale screenshots (visual test fails).

### 3.12 Templates and automation to ship

| Artifact | Path | Content |
|---|---|---|
| SOW linter | `scripts/sow-lint.sh` | Validates statuses, links, dependencies, checklists, progress logs |
| WP template | `.docs/product/templates/WP.md` | Exact structure of this file |
| Card template | `.docs/product/templates/card.md` | Canonical story + DoR/DoD checklist |
| ADR template | `.docs/product/templates/adr.md` | Decision record |
| CR template | `.docs/product/templates/change-request.md` | Approval matrix fields |
| PR template | `.github/pull_request_template.md` | Appendix C.4.1 + gates |
| Milestone gate checklist | `.docs/product/templates/milestone-gate.md` | Demo, evidence, docs, retro |
| Metrics views | `migrations/0002_analytics_views.sql` | Lead/cycle/throughput/WIP/blocked/cost |
| Retro template | `.docs/product/templates/retro.md` | 3 actions + owners + cards |

---

## 4. Deliverables (checklist)

- [x] `.docs/product/governance.md` describing Lean rules, Agile flow, roles, RACI, ceremonies and escalation, with measurable controls. Evidence: `.docs/product/governance.md`.
- [x] `.docs/product/quality-gates.md` listing every required CI check, its owner, threshold and failure action. Evidence: `.docs/product/quality-gates.md`, `.github/required-checks.txt`.
- [x] `.docs/product/status-model.md` defining `ready`/`progress`/`done`, blocked flag, transition rules and evidence requirements. Evidence: `.docs/product/status-model.md`.
- [x] `.docs/product/metrics.md` with formulas, SQL sources, targets and dashboard mapping. Evidence: `.docs/product/metrics.md`, `.tools/test-analytics-views.sh`.
- [x] `.docs/product/change-control.md` with CR form, approval matrix, emergency path and records location. Evidence: `.docs/product/change-control.md`, `.docs/product/change-requests/CR-0001.md`.
- [x] `.docs/product/templates/WP.md`, `card.md`, `adr.md`, `change-request.md`, `milestone-gate.md`, `retro.md`. Evidence: `.docs/product/templates/`; used by `.docs/product/change-requests/CR-0001.md` and `.docs/product/backlog/AIL-101.md`.
- [x] `.github/pull_request_template.md` aligned with Appendix C.4.1 and the global gates. Evidence: `.github/pull_request_template.md`.
- [x] `scripts/sow-lint.sh` + CI job `sow-lint` (fails on any violation in §3.6). Evidence: `.tools/sow-lint-self-test.sh` (11 cases), `.github/workflows/ci.yml`.
- [x] `migrations/0002_analytics_views.sql` with the metric views; verified against seeded data. Evidence: `.tools/test-analytics-views.sh` (25 assertions on fixed fixtures).
- [x] `.docs/product/risk-register.md` (program + WP rollup), reviewed weekly. Evidence: `.docs/product/risk-register.md`.
- [x] Retrospective record for M0 with three owned actions. Evidence: `.docs/product/retrospectives/M0-retro.md`, `.docs/product/backlog/AIL-101.md`..`AIL-103.md`.
- [x] ADR-0001..0004 recorded (see WP-01 for the full ADR set). Evidence: `.docs/adr/adr-0001.md`..`adr-0004.md`, `.docs/adr/registry.md`.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP00-T01 | Write governance, roles, RACI, ceremonies | Analyst | 2 | — | Docs reviewed; no ambiguous decision right |
| WP00-T02 | Define DoR/DoD (card + WP) and status machine | Tech Lead | 1 | T01 | Matches spec §6.4 and SOW §6 |
| WP00-T03 | Define quality gates and map to CI check names | QA | 2 | T02 | Every gate has an exact check name |
| WP00-T04 | Define metrics, formulas and SQL views | Dev | 3 | T02 | Views return correct values on seed |
| WP00-T05 | Build templates (WP/card/ADR/CR/gate/retro) | Analyst | 2 | T01 | All templates present and lint-clean |
| WP00-T06 | Implement `scripts/sow-lint.sh` + CI job | Dev | 3 | T05 | Fails on status/link/dependency/checklist violations |
| WP00-T07 | Write PR template and branch/commit policy | Tech Lead | 1 | T03 | Matches Appendix C.4 |
| WP00-T08 | Risk register + weekly review process | Tech Lead | 1 | T01 | Owners and review dates set |
| WP00-T09 | M0 retrospective and actions | Tech Lead | 1 | All | 3 actions with owners and cards |

---

## 6. Acceptance criteria

1. `sow-lint` exits non-zero when: a WP status is not one of the three; a WP link is broken; a dependency is missing or cyclic; a `done` WP has unchecked deliverables; a `progress` WP has no update in 3 working days.
2. Every WP file in `.docs/sow/` validates against the WP template structure.
3. Every required CI check name in §3.8 exists in the workflow and is required by branch protection.
4. The metrics views return the expected values for the deterministic seed (verified by an automated test with fixed fixtures).
5. A sample CR can be submitted, approved and recorded without ambiguity; the approval matrix is enforced by the template and review.
6. A retrospective produces exactly three actions, each linked to a `TODO` card.
7. A reviewer/merger conflict is rejected by the system (`422 SEPARATION_OF_DUTIES`) in an integration test owned by WP-09/WP-15.

---

## 7. WP Definition of Done

- [x] All deliverables checked and linked to evidence.
- [x] `sow-lint` green on the full SOW. Evidence: `scripts/sow-lint.sh` exit 0; `.tools/sow-lint-self-test.sh` 11/11.
- [x] Governance docs reviewed by Tech Lead and QA. Evidence: independent `APPROVED` review in `.docs/product/evidence/WP-00-review.md`.
- [x] Templates used in at least one real card and one real CR (`.docs/product/change-requests/CR-0001.md`, `.docs/product/backlog/AIL-101.md`..`AIL-103.md`).
- [x] Metrics dashboard populated from the seed and reviewed in the weekly meeting. Assigned to WP-20 by CR-0002; WP-00 delivers the verified views and formulas (`migrations/0002_analytics_views.sql`, `.docs/product/metrics.md`, 25 assertions).
- [x] No open P0/P1; no unresolved blocker. B-01 closed by CR-0003; B-02 closed by the AC7 integration test (`crates/api/tests/separation_of_duties.rs`); B-03 assigned by CR-0002; B-04 resolved by the independent review.
- [x] Status updated to `done` with sign-off row (2026-11-07).

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Governance becomes ceremony without effect | Medium | Every rule has a metric or CI enforcement; no rule without a control |
| Status drift between index and WP files | Medium | `sow-lint` in CI; single source per field |
| Over-process slows delivery | Medium | Lean rule L-08; templates minimal; automation preferred over manual checks |
| Metrics gamed by agents | Medium | Metrics derived from immutable events; periodic audit; no self-reported numbers |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §1, §6.4, §17, §21, §22, §23
- `../technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.3, §C.4
- `../technical-specification/APPENDIX-D-DELIVERY.md` §D.1, §D.2, §D.3
- `SOW.md` §4–§12

## Progress log

2026-11-07 | T01 | .docs/product/governance.md, .docs/product/change-control.md, .docs/product/change-requests/CR-0001.md, .docs/product/change-requests/README.md | doc link check | CR-0001 (docs root convention) | T02
2026-11-07 | T02 | .docs/product/status-model.md | doc link check | — | T03
2026-11-07 | T03 | .docs/product/quality-gates.md, .github/required-checks.txt | sow-lint | — | T04
2026-11-07 | T04 | .docs/product/metrics.md, migrations/0002_analytics_views.sql, .tools/test-analytics-views.sh, .tools/fixtures/analytics-seed.sql | .tools/test-analytics-views.sh (25 assertions) | — | T05
2026-11-07 | T05 | .docs/product/templates/WP.md, card.md, adr.md, change-request.md, milestone-gate.md, retro.md | doc link check | — | T06
2026-11-07 | T06 | scripts/sow-lint.sh, .tools/sow-lint-self-test.sh, .github/workflows/ci.yml | .tools/sow-lint-self-test.sh (11 cases) | — | T07
2026-11-07 | T07 | .github/pull_request_template.md, .github/branch-protection.json, .docs/product/governance.md | sow-lint (required checks) | — | T08
2026-11-07 | T08 | .docs/product/risk-register.md | doc link check | — | T09
2026-11-07 | T09 | .docs/product/retrospectives/M0-retro.md, .docs/product/backlog/AIL-101.md, AIL-102.md, AIL-103.md, .docs/product/backlog/README.md | doc link check | — | gates
2026-11-07 | gates | all WP-00 artifacts | harness-lint OK (16 checks), sow-lint OK (31 checks), sow-lint self-test 11/11, analytics views 25/25, doc links 138 checked | — | closure
2026-11-07 | closure | .docs/product/evidence/WP-00-review.md, .docs/product/change-requests/CR-0002.md | independent review APPROVED; harness-lint OK; sow-lint OK | CR-0002 | WP-01

## Blockers

- `blocked=false`
- B-01 closed by CR-0003: branch protection is applied on the GitHub `main` branch with the 16 required contexts; verified with `gh api repos/massimobiagioli/ai-lab/branches/main/protection` (admin bypass enabled for the review merge workflow, no required PR reviews, merge commits allowed, force pushes/deletions blocked).
- B-02 closed by the AC7 integration test (`crates/api/tests/separation_of_duties.rs`, asserting HTTP 422 `SEPARATION_OF_DUTIES`).
- B-03 assigned by CR-0002: the live dashboard is WP-20 scope; the verified views and formulas are delivered.
- B-04 resolved by `.docs/product/evidence/WP-00-review.md` (independent `APPROVED`).
2026-11-07 | env-update | .git (operator first commit on main) | native hooks installed via scripts/hooks/install.sh; no remote | B-01 remains: branch protection needs a GitHub remote | WP-03

2026-11-07 | review | .docs/sow/WP-00-GOVERNANCE-review-plan.md, CR-0003 | branch protection verified; doc links 138 | CR-0003 | QA re-review and merge

2026-11-07 | review-fix | crates/api/tests/separation_of_duties.rs, .docs/product/evidence/WP-00-review.md, .docs/product/evidence/AIL-101-branch-protection.txt | cargo test -p ailab-api --test separation_of_duties (2 passed); QA CHANGES_REQUESTED resolved | — | QA re-review
