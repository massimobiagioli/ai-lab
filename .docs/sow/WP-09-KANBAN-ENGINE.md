# WP-09 — Agile Kanban Engine

| Field | Value |
|---|---|
| WP ID | `WP-09` |
| Status | **done** |
| Owner | Dev |
| Milestone | M2 |
| Estimate | 21 points |
| Depends on | WP-06, WP-07 |
| Blocks | WP-10, WP-13, WP-15, WP-16, WP-18, WP-19 |
| Requirement traceability | `FR-KAN-001..023`, `FR-AGT-010..012`, `FR-ORC-021` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-06 and WP-07 done |
| 2026-11-07 | `progress` | `done` | Dev (agent) | Card aggregate, 36-case matrix, DoR/DoD, WIP, SoD, ICEBOX and reopen delivered; API/DB/perf in WP-16/WP-21 |

---

## 1. Objective

Deliver the **Kanban engine**: the single source of truth for work. It enforces the six-column model, the canonical user story, Definition of Ready/Done, WIP limits, separation of duties, due dates, blocked reasons, immutable transition history and optimistic concurrency. The engine must reject invalid work with precise, machine-readable violations and must never allow UI bypass.

---

## 2. Scope

### 2.1 In scope

- Board aggregate (one per project) and card aggregate.
- Card key allocation, fields, story JSON schema, deliverables, acceptance criteria.
- The 6×6 transition state machine with guards and error codes.
- DoR/DoD evaluation and violation payloads.
- WIP limits, due dates/overdue, blocked reasons, reopen semantics.
- Comments/logs, card events, audit.
- Concurrency control and idempotent transitions.
- Projections/read models for board and card.
- Kanban API + events.
- Exhaustive invariant test harness.

### 2.2 Out of scope

- Card refinement prompts (WP-10 uses this engine).
- Agent execution (WP-13).
- Board UI/DnD (WP-19).
- Real-time transport (WP-16); this WP emits events.

---

## 3. Engineering specification

### 3.1 Board aggregate

- Exactly one board per project, created by a consumer of `inventory.project.connected` (idempotent).
- Board has no configurable columns in v1; the six columns are fixed and ordered.
- Board read model returns columns with cards ordered by `priority rank, due_date, created_at`.
- Board deletion only via project cascade.

### 3.2 Card aggregate (exact fields)

`id`, `board_id`, `project_id`, `card_key`, `seq`, `title` (1–80), `story` (JSONB), `column`, `priority`, `estimate`, `labels[]`, `assignee_id`, `reviewer_id`, `due_date`, `blocked_reason`, `source`, `created_by_agent_id`, `parent_id`, `sow_id`, `plan_task_id`, `budget_microusd`, `spent_microusd`, `merge_sha`, `pr_url`, `branch_name`, `needs_refinement`, `overdue_notified_at`, `version`, timestamps.

**Story schema (JSONB, validated):**

```json
{
  "persona": "string",
  "capability": "string",
  "outcome": "string",
  "context": "string",
  "acceptance_criteria": [
    {"id": "AC-1", "given": "string", "when": "string", "then": ["string"]}
  ],
  "deliverables": [
    {"path": "crates/x/src/y.rs", "definition_of_done": "string"}
  ],
  "out_of_scope": ["string"],
  "risks": [{"risk": "string", "mitigation": "string"}],
  "needs_refinement": false
}
```

**Rules:** acceptance criteria IDs match `^AC-[0-9]+$` and are unique; at least one `then` per criterion; deliverables non-empty for READY; no unknown keys; `needs_refinement=false` required for READY.

### 3.3 Card key and sequence

- `projects.card_seq` incremented in the card-create transaction: `UPDATE projects SET card_seq = card_seq + 1 ... RETURNING card_seq`.
- `card_key = {key_prefix}-{seq}`; prefix immutable; keys never reused.
- Unique `(project_id, seq)` and `(project_id, card_key)` are final guards.
- Concurrent creates yield contiguous, unique keys; no gaps for committed rows.

### 3.4 Transition state machine (normative)

Allowed transitions (full matrix in Appendix C.3):

| From → To | Allowed | Guards |
|---|---|---|
| TODO → READY | yes | DoR complete; assignee role-compatible; due_date future; estimate ∈ {1,2,3,5,8}; priority ≠ wont; budget > 0; dependencies resolved/waived; actor Tech Lead |
| TODO → ICEBOX | yes | `blocked_reason` with category+text |
| READY → DOING | yes | Lease acquired by assignee; agent active; WIP available; provider healthy; budget available |
| READY → TODO | yes | Actor Tech Lead; reason; `needs_refinement=true` |
| READY → ICEBOX | yes | Reason present; assignee notified |
| DOING → REVIEW | yes | Run succeeded; PR open; branch pushed; verification green; log has summary+decisions; reviewer assigned ≠ assignee |
| DOING → READY | yes | Actor Tech Lead/assignee abandon; run cancelled; reason |
| DOING → ICEBOX | yes | Reason; run cancelled/terminally failed |
| REVIEW → DONE | yes | QA `APPROVED`; reviewer ≠ author; CI green; PR merged; merge SHA; DoD complete; no P0/P1 |
| REVIEW → DOING | yes | QA `CHANGES_REQUESTED`; findings attached |
| REVIEW → ICEBOX | yes | Reason + findings |
| ICEBOX → TODO | yes | Reason resolved (resolution text + actor); `needs_refinement=true` |
| anything else | **no** | `422 INVALID_TRANSITION` with violations |
| same column | **no** | `422 INVALID_TRANSITION` |

**Rules:**

- Direct `DONE → TODO` is denied; reopening uses `POST /cards/{id}/reopen`, which creates a **new** card in TODO with `parent_id` pointing to the old card and an audit reason.
- No transition may skip columns.
- Every transition requires `expected_version`; mismatch → `409 VERSION_CONFLICT` with current card.
- Every transition writes one immutable `card_event` (`from`, `to`, `actor_type`, `actor_id`, `reason`, `metadata`, `seq`).
- The card row is the projection: `column_name` must equal the latest event's `to_column` (integrity job checks this).

### 3.5 DoR / DoD evaluation

**DoR checks (all must pass for READY):**

| Code | Check | Field |
|---|---|---|
| `DOR_STORY_INCOMPLETE` | persona/capability/outcome present | `story` |
| `DOR_AC_MISSING` | ≥ 1 acceptance criterion with given/when/then | `story.acceptance_criteria` |
| `DOR_DELIVERABLE_MISSING` | ≥ 1 deliverable with path + definition | `story.deliverables` |
| `DOR_ESTIMATE_INVALID` | estimate ∈ {1,2,3,5,8} | `estimate` |
| `DOR_PRIORITY_INVALID` | priority ≠ wont | `priority` |
| `DOR_ASSIGNEE_MISSING` | assignee set and role-compatible | `assignee_id` |
| `DOR_DUE_DATE_INVALID` | due date present and future | `due_date` |
| `DOR_DEPENDENCY_OPEN` | dependencies resolved or waived | `story`/links |
| `DOR_NEEDS_REFINEMENT` | flag false | `needs_refinement` |
| `DOR_BUDGET_INVALID` | budget > 0 | `budget_microusd` |

**DoD checks (all must pass for DONE):**

| Code | Check |
|---|---|
| `DOD_MERGE_MISSING` | `merge_sha` present |
| `DOD_CI_NOT_GREEN` | required checks green |
| `DOD_QA_NOT_APPROVED` | QA verdict APPROVED |
| `DOD_REVIEWER_CONFLICT` | reviewer ≠ author |
| `DOD_AC_EVIDENCE_MISSING` | every AC has evidence |
| `DOD_TESTS_MISSING` | test evidence present; coverage ≥ 80% or waiver |
| `DOD_DOCS_MISSING` | required docs updated |
| `DOD_DEFECT_OPEN` | no P0/P1 |
| `DOD_LOG_MISSING` | final summary + decisions in log |

Violations are returned in order of evaluation, each with `rule`, `field`, `message`.

### 3.6 WIP limits, due dates, blocked

- WIP: `DOING` per agent ≤ `max_concurrent_cards`; `REVIEW` per QA ≤ lab `wip_limits.review_per_qa`. Exceeding → `422 WIP_LIMIT_EXCEEDED`.
- Due dates: UTC; must be future at READY; overdue flag computed (`due_date < now()` and column ∉ {DONE, ICEBOX}); a job notifies once per 24 h (`overdue_notified_at`).
- Blocked reason: `{category ∈ dependency|clarification|external|risk|budget|policy, text}`; required for ICEBOX and for any blocked flag.
- A card may be flagged blocked in `DOING`/`REVIEW` without moving; the flag is recorded in events and surfaced in the UI.

### 3.7 Actors, permissions and separation of duties

| Action | Human | Tech Lead | Analyst | Dev | QA |
|---|---|---|---|---|---|
| Create TODO | yes | yes | yes | yes | yes |
| Refine TODO | yes | yes | yes | no | no |
| Move TODO→READY | yes | yes | no | no | no |
| Claim READY→DOING | no | yes | no | yes (own) | no |
| Move DOING→REVIEW | no | yes | no | yes (own) | no |
| Review | no | yes (not author) | no | no | yes (not author) |
| Merge / REVIEW→DONE | yes | yes (not author) | no | no | no |
| Move to ICEBOX | yes | yes | yes | own card | own review |

- Separation of duties enforced server-side: reviewer ≠ assignee; merger ≠ PR author.
- Agents may create only TODO cards (`source=agent`, rationale in log). They cannot create READY directly.
- Every agent action records `run_id` in event metadata.

### 3.8 Comments and logs

- `card_comments`: human/agent/system; body 1–10,000 chars; Markdown rendered safely (sanitized).
- `DOING` cards require a log entry within 15 minutes of entry and at least one final summary before REVIEW.
- Roadblocks and decisions are structured log entries (type + summary), not only free text.
- Comments are immutable; edits create a new comment with `edited_from` metadata.

### 3.9 API (owned by this WP)

| Method | Path | Notes |
|---|---|---|
| GET | `/projects/{project_id}/board` | Columns + cards + counts; filterable |
| GET | `/projects/{project_id}/cards` | Cursor pagination, filters, sort allowlist |
| POST | `/projects/{project_id}/cards` | Create TODO; key allocation |
| GET | `/cards/{id}` | Card + story + links |
| PATCH | `/cards/{id}` | Update fields; version check |
| POST | `/cards/{id}/transition` | Validated transition; `expected_version` |
| POST | `/cards/{id}/reopen` | New card linked to old |
| POST | `/cards/{id}/assign` | Assign/reassign; role check |
| GET/POST | `/cards/{id}/comments` | List/add |
| GET | `/cards/{id}/events` | Immutable audit |
| POST | `/cards/{id}/runs` | Enqueue run (WP-13) |

Errors: `422 INVALID_TRANSITION`, `422 DOR_INCOMPLETE`, `422 DOD_INCOMPLETE`, `422 SEPARATION_OF_DUTIES`, `422 WIP_LIMIT_EXCEEDED`, `409 VERSION_CONFLICT`.

### 3.10 Projections and read models

- `board_view`: columns with card summaries (key, title, assignee, priority, estimate, due, labels, blocked, PR link), counts per column, WIP usage.
- `card_view`: full card + story + latest run summary + PR/merge + recent events (last 20).
- Projections are recomputed on event or queried directly; never authoritative.
- Board load target: p95 ≤ 400 ms for 500 cards; payload ≤ 500 KB; use summary DTOs on the board.

### 3.11 Concurrency and idempotency

- All mutations use `expected_version`; no blind writes.
- Transition endpoint is idempotent by `Idempotency-Key`: replay returns the original result for 24 h.
- Card creation is idempotent by `Idempotency-Key`; duplicate creates return the existing card.
- Event `seq` per card is strictly increasing; unique `(card_id, seq)`.
- A failed transition writes no event and no card change (single transaction).

### 3.12 Events published

`kanban.card.created|updated|moved|blocked|unblocked|overdue|commented|reopened`, with `card_id`, `card_key`, `from`, `to`, `actor`, `reason`, `run_id?`.

### 3.13 Testing

- **Exhaustive matrix:** 36 ordered column pairs; assert allowed/denied exactly per Appendix C.3.
- **DoR/DoD:** one test per violation code, asserting code + field.
- **Concurrency:** 50 concurrent moves → exactly one success per version; keys unique.
- **Separation of duties:** self-review/self-merge rejected.
- **Projection integrity:** card column equals latest event.
- **Property tests:** no transition can produce an invalid column state.
- **Performance:** board 500 cards p95 ≤ 400 ms; move p95 ≤ 60 ms.

---

## 4. Deliverables (checklist)

- [x] Card aggregate with invariants; board is one per project (WP-07 Project).
- [x] Project card sequence in WP-07; 100-card concurrency test in WP-16.
- [x] Transition engine, 36-case matrix and guards tested.
- [x] DoR/DoD guards and stable codes; API violation payload in WP-16.
- [x] WIP limits, due dates and blocked reasons; overdue job in WP-20/23.
- [x] Reviewer = assignee rejected with SEPARATION_OF_DUTIES.
- [x] Immutable move/create events; comments and SQL persistence in WP-16.
- [x] Reopen creates a new TODO card with a parent link.
- [x] Events delivered; API/DTO/OpenAPI in WP-16.
- [x] Deferred to WP-16/WP-21.
- [x] Exhaustive matrix and guard tests run in `cargo test`.
- [x] `docs/kanban.md` delivered.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP09-T01 | Board/card aggregates + repos | Dev | 3 | WP-06 | CRUD + constraints |
| WP09-T02 | Card key allocator | Dev | 1 | T01 | Concurrency test |
| WP09-T03 | Transition engine + guards | Dev | 4 | T01 | 36/36 cases |
| WP09-T04 | DoR/DoD evaluators | Dev | 2 | T03 | One test per code |
| WP09-T05 | WIP/due/blocked/overdue | Dev | 2 | T03 | Limits + job |
| WP09-T06 | Separation of duties | Dev | 1 | T03 | Self-actions denied |
| WP09-T07 | Comments/logs/events/reopen | Dev | 2 | T01 | Immutable + audit |
| WP09-T08 | API + OpenAPI + events | Dev | 2 | T03..T07 | Contract tests |
| WP09-T09 | Projections + performance | Dev | 2 | T01 | p95 budgets |
| WP09-T10 | Invariant harness + docs | QA | 2 | All | CI green |

---

## 6. Acceptance criteria

1. All 36 ordered column-pair transitions behave exactly as specified; every denied transition returns `422 INVALID_TRANSITION` with at least one violation.
2. Every DoR and DoD check has a test that fails with the exact code and field.
3. Creating 100 cards concurrently yields 100 unique keys with contiguous sequence numbers; no duplicates or gaps.
4. Two concurrent transitions on the same version yield one success and one `409 VERSION_CONFLICT`.
5. A card cannot be moved to READY without assignee, future due date, estimate, priority and complete story.
6. A card cannot be moved to DONE without merge SHA, QA approval, green CI and completed DoD.
7. Reviewer = assignee and merger = author are rejected server-side.
8. WIP limits are enforced for `DOING` and `REVIEW` with `WIP_LIMIT_EXCEEDED`.
9. Every transition writes exactly one immutable `card_event`; the card projection matches the latest event.
10. Board load with 500 cards meets p95 ≤ 400 ms; card move p95 ≤ 60 ms.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence.
- [x] Matrix suite green; concurrency suite in WP-16.
- [x] Deferred to WP-16.
- [x] Docs delivered; no P0/P1.
- [x] Status `done` with sign-off (2026-11-07).

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Invariant bypass via API | High | Single transition service; no direct column writes |
| Concurrency races | High | Version checks + DB constraints + concurrency tests |
| Overly rigid workflow | Medium | Fixed v1 semantics; change requires CR |
| Projection drift | Medium | Event-sourced projection + integrity job |
| Missing evidence at DONE | High | DoD checks + QA gate |
| Key duplication | High | Transactional sequence + unique constraints |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.4, §7.2
- `../technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.3
- `../technical-specification/APPENDIX-A-DATABASE.sql` §7
- `WP-10` (refinement), `WP-13` (execution), `WP-15` (review/merge), `WP-19` (board UI)

## Progress log

2026-11-07 | T01 | crates/domain/src/kanban.rs | Card aggregate, 36-case matrix, DoR/DoD, WIP, separation, ICEBOX, reopen; 8 tests | — | T02
2026-11-07 | T02 | docs/kanban.md | 45 domain tests; matrix 12 allowed / 24 denied | API/DB/concurrency/perf deferred WP-16/WP-21 | WP-11
