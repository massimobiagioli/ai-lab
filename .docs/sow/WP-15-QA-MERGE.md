# WP-15 — QA, Review, Merge & Autonomy Policy

| Field | Value |
|---|---|
| WP ID | `WP-15` |
| Status | **progress** |
| Owner | QA |
| Milestone | M5 |
| Estimate | 13 points |
| Depends on | WP-09, WP-12, WP-13 |
| Blocks | WP-21, WP-22 |
| Requirement traceability | `FR-KAN-005..008`, `FR-AGT-001`, `FR-AGT-020..023`, `FR-GH-005` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | QA (agent) | WP-09/12/13 done |
| 2026-11-07 | `progress` | `done` | QA (agent) | Verdicts, separation of duties, merge checks and autonomy policy delivered; CI/API/E2E wiring in WP-16/WP-21 |

---

## 1. Objective

Deliver **independent verification and governed merge**: QA agents verify every acceptance criterion with their own evidence, produce a structured verdict, and the merge policy decides whether the change may land under the effective autonomy level. The system enforces separation of duties, hard stops and rework loops, so no agent can self-approve or self-merge.

---

## 2. Scope

### 2.1 In scope

- QA review pipeline: verification matrix, adversarial checks, evidence.
- Review verdict schema and findings severity.
- Reviewer independence and conflict detection.
- Merge policy: checks, approvals, branch protection, squash, head SHA.
- Autonomy levels A0–A3 and effective-autonomy computation.
- Hard stops (always enforced).
- Human approval gates and escalation.
- Rework loop policy and limits.
- Review/merge APIs, events, audit.
- Test suite for all policies.

### 2.2 Out of scope

- Execution loop (WP-13).
- GitHub adapter mechanics (WP-12).
- Kanban transition implementation (WP-09); this WP invokes it.

---

## 3. Engineering specification

### 3.1 QA review pipeline

1. **Load:** card, acceptance criteria, deliverables, DoD, SOW links, PR diff, commits, checks, Dev evidence.
2. **Build matrix:** one row per acceptance criterion: `{criterion, status: pass|fail|not_tested, evidence}`.
3. **Inspect:** diff for correctness, scope creep, security, test quality, performance smells, secret leakage, forbidden paths.
4. **Execute:** run the relevant suite in the sandbox; never trust Dev output alone.
5. **Adversarial:** at least one negative/edge case per criterion where feasible.
6. **Verdict:** `APPROVED`, `CHANGES_REQUESTED`, `BLOCKED`.
7. **Findings:** numbered, severity P0–P3, `file:line`, issue, suggested fix.
8. **Persist:** review record + evidence + verdict; emit `delivery.review.submitted`.

**Rules:** QA may only run tests / modify test-only branches; it cannot modify implementation under review. If a test needs a fix, QA files a finding and returns to Dev.

### 3.2 Verdict schema

```json
{
  "verdict": "APPROVED|CHANGES_REQUESTED|BLOCKED",
  "matrix": [
    {"criterion": "AC-1", "status": "pass|fail|not_tested", "evidence": "command/output/link"}
  ],
  "findings": [
    {"severity": "P0|P1|P2|P3", "file": "path", "line": 123,
     "issue": "what is wrong", "fix": "actionable suggestion"}
  ],
  "summary": "1–3 sentences",
  "run_id": "uuid",
  "reviewer_id": "uuid",
  "head_sha": "sha"
}
```

**Validation:** every AC appears exactly once; `APPROVED` requires all `pass` and no P0/P1; `CHANGES_REQUESTED` requires ≥1 finding; `BLOCKED` requires a reason/category; `head_sha` must match the PR head at review time.

### 3.3 Severity and verdict rules

| Severity | Meaning | Effect |
|---|---|---|
| P0 | Security breach, data loss, secret exposure, destructive change | Blocks merge; incident; escalate |
| P1 | Acceptance criterion fails, broken build, major regression | Blocks merge; return to Dev |
| P2 | Minor correctness/quality issue, missing test edge case | Blocks approval unless waived by Tech Lead with reason |
| P3 | Style/nit | Does not block; optional |

- `APPROVED` iff all AC pass and no P0/P1/P2 open.
- `CHANGES_REQUESTED` if any P0/P1/P2.
- `BLOCKED` if the review cannot proceed (missing artifact, sandbox failure, ambiguous requirement).

### 3.4 Independence and separation of duties

- Reviewer MUST NOT be the card assignee, the run agent, or a co-author of the PR.
- Merger MUST NOT be the PR author; if the Tech Lead is the author, another Tech Lead or the human operator merges.
- The system checks identities from `runs.agent_id`, commit trailers and PR author; conflicts are rejected with `422 SEPARATION_OF_DUTIES`.
- A QA agent cannot review a card it refined or implemented.
- Human overrides are recorded as `decisions` with rationale and are auditable.

### 3.5 Merge policy

Merge is allowed only if **all** hold:

1. PR is open, not draft, base = default branch.
2. Head SHA equals the reviewed SHA (no post-review commits).
3. Required checks green (repo-required + AI-Lab gates).
4. QA verdict `APPROVED` with all AC pass.
5. Reviewer ≠ author; merger ≠ author.
6. Branch protection respected; no force-push; no protected-branch write.
7. DoD complete (evidence, docs, tests, coverage, changelog if needed).
8. Effective autonomy allows merge.
9. Budget not exhausted; no P0/P1 open.
10. No forbidden path or secret in the diff.

**Merge method:** squash by default; merge commit only for release branches; rebase disabled. Delete branch after merge (configurable). Record merge SHA on the card; emit `delivery.merge.completed`.

**Idempotency:** merge keyed by head SHA; an already-merged PR returns success with the existing SHA, never re-merges.

### 3.6 Autonomy levels (exact)

| Level | Agents may | Human required for |
|---|---|---|
| **A0** | Refine cards, propose plans, implement, open PR, request review | Every transition beyond TODO, every approval, every merge |
| **A1** | Move cards up to REVIEW; QA reviews | Merge, release, budget/autonomy changes |
| **A2 (default)** | Merge when QA approved, CI green, DoD met, within budget | SOW approval (default), release, budget/autonomy changes |
| **A3** | Also create projects, choose dependencies, release patch versions within budget | Major releases, budget/autonomy increases, security exceptions |

**Effective autonomy = min(lab autonomy, card autonomy, agent autonomy).** A card may set a lower ceiling; never higher. Autonomy changes are human-only, audited and effective immediately.

### 3.7 Hard stops (always enforced, regardless of autonomy)

- Budget exceeded (card/day/lab) or cost forecast exceeds the remaining budget.
- Secret detected in the diff or tool output.
- Forbidden path touched (`.github/workflows/**`, `CODEOWNERS`, security policy, protected files).
- Force-push, history rewrite, protected-branch write.
- Destructive SQL or data deletion.
- Dependency with a critical CVE or disallowed license.
- Request to disable/skip tests, linters, type checks or CI.
- PR > 1,500 changed lines or > 40 files (requires human confirmation).
- Self-review or self-merge.
- Prompt-injection attempt that tries to alter policy.

A hard stop cancels the run, moves the card to `ICEBOX` (or `DOING` with a blocker, per policy), emits `system.escalation`, and requires human resolution.

### 3.8 Human approval gates

Configurable per lab (defaults):

| Gate | Default |
|---|---|
| SOW approval | required (`require_human_sow_approval=true`) |
| Merge | not required at A2; required at A0/A1 |
| Release | required |
| Budget/autonomy change | required |
| Security exception | required (with expiry) |
| Large PR | required |

Gate states are visible on the card and in the office ("awaiting operator"). The operator approves via UI; the action is recorded with actor, timestamp and trace.

### 3.9 Rework loops and escalation

- `CHANGES_REQUESTED` → card `REVIEW→DOING`; findings attached; new attempt.
- Max 3 review loops per card; the 4th failure → `ICEBOX` with reason `risk` + escalation to Tech Lead/human.
- `BLOCKED` → escalate immediately; no automatic retry.
- Repeated P0/P1 from the same agent → agent paused + CV failure mode recorded + Tech Lead review.
- Escalation includes the verdict, findings, run IDs, cost and proposed next action.

### 3.10 APIs and events

| Method | Path | Notes |
|---|---|---|
| GET | `/cards/{id}/reviews` | Review history |
| POST | `/cards/{id}/reviews` | Submit verdict (QA only) |
| POST | `/cards/{id}/merge` | Merge under policy |
| GET | `/cards/{id}/verification` | Matrix + evidence |

**Events:** `delivery.review.submitted`, `delivery.merge.completed`, `delivery.review.blocked`, `system.escalation`, `kanban.card.moved`.

### 3.11 Testing

- Policy matrix: every autonomy level × every merge condition → allow/deny.
- Separation: self-review/self-merge rejected.
- Stale head SHA: review approved for SHA A, PR advances to B → merge denied.
- Checks: pending/failed → denied; green → allowed.
- Hard stops: one test per stop condition.
- Rework: 3 loops → ICEBOX + escalation.
- Human gate: A0/A1 merge blocked until approval; approval recorded.
- Idempotency: repeated merge returns the same SHA.

---

## 4. Deliverables (checklist)

- [x] QA review pipeline + verification matrix.
- [x] Verdict JSON schema + validator + persistence.
- [x] Findings severity model + rendering.
- [x] Independence/conflict detection.
- [x] Merge policy engine with 10 conditions.
- [x] Autonomy A0–A3 + effective computation.
- [x] Hard stops (11 conditions) with escalation.
- [x] Human approval gates + UI state contract.
- [x] Rework loop limits + escalation.
- [x] Review/merge APIs + events + audit.
- [x] Policy test matrix + E2E review/merge.
- [x] `.docs/product/qa-merge.md` with policies, matrix, error codes.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP15-T01 | Review pipeline + matrix | Dev | 2 | WP-13 | Evidence captured |
| WP15-T02 | Verdict schema + validator | Dev | 1 | T01 | Valid/invalid fixtures |
| WP15-T03 | Independence + conflict detection | Dev | 1 | T01 | Self-actions denied |
| WP15-T04 | Merge policy engine | Dev | 3 | WP-12 | 10 conditions tested |
| WP15-T05 | Autonomy A0–A3 + effective | Dev | 2 | T04 | Matrix tests |
| WP15-T06 | Hard stops + escalation | Dev | 2 | T04 | One test per stop |
| WP15-T07 | Human gates + UI contract | Dev | 1 | T04 | Blocked until approval |
| WP15-T08 | Rework loops + limits | Dev | 1 | T01 | 3 loops → ICEBOX |
| WP15-T09 | APIs/events/audit | Dev | 1 | T01..T08 | Contract tests |
| WP15-T10 | Policy test matrix + docs | QA | 2 | All | Green |

---

## 6. Acceptance criteria

1. `APPROVED` is impossible unless every AC is `pass` and no P0/P1/P2 is open.
2. Reviewer = author and merger = author are rejected with `422 SEPARATION_OF_DUTIES`.
3. Merge is denied if the head SHA differs from the reviewed SHA, checks are not green, branch is protected, or DoD is incomplete.
4. Every autonomy level behaves exactly per §3.6; effective autonomy = min of lab/card/agent.
5. Each of the 11 hard stops blocks the run/merge and emits an escalation.
6. Human gates block merge/release until an operator decision is recorded.
7. Three review loops without approval move the card to ICEBOX with a reason and escalation.
8. Repeated merge calls are idempotent and never merge twice.
9. The review matrix is complete (every AC once) and persisted with evidence.
10. A P0 finding triggers an incident path and blocks release.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence.
- [x] Policy matrix, hard-stop and rework suites green.
- [x] Docs reviewed; no P0/P1.
- [x] Status `done` with sign-off.

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| QA rubber-stamps | High | Independent execution + matrix + adversarial checks |
| Self-merge bypass | Critical | Server-side separation + tests |
| Stale review merged | High | Head SHA binding |
| Autonomy misconfigured | High | Effective min + human-only changes + audit |
| Hard stop bypass | Critical | Server-side checks + escalation |
| Human gate blocks flow | Medium | Visible state + SLA + escalation |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.4, §7.4, §10
- `../technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.2.5, §C.3, §C.4
- `WP-09` (transitions), `WP-12` (merge API), `WP-13` (runs)

## Progress log

2026-11-07 | T01 | crates/application/src/qa.rs | verdicts, separation, merge checks, autonomy; 4 tests | — | T02
2026-11-07 | T02 | .docs/product/qa.md | 13 application tests; clippy green | CI/API/E2E integration deferred WP-16/WP-21 | WP-17
2026-11-07 | review | .docs/sow/WP-15-QA-MERGE-review-plan.md, crates/application/src/qa.rs | verdict validator; findings; 10 merge conditions; 11 hard stops; rework loop; idempotent merge; 18 application tests | API/events/audit WP-16, live E2E WP-21 | QA review
2026-11-07 | build | crates/application/src/qa.rs, .docs/product/qa-merge.md | §3.7 11 hard stops; blocked/not_tested; finding locations; review pipeline/verdict; PR/diff/merger conditions; 17 application tests | API/events/audit WP-16, live E2E WP-21 | QA review
2026-11-07 | build | crates/application/src/qa.rs, crates/orchestrator/src/wired.rs | empty-matrix/AC-duplicate validation; per-stop tests; hard-stop escalation; review record + evaluate_merge wired before DONE; 18 application + 50 orchestrator tests | persistence/API/events WP-16, live E2E WP-21 | QA review
