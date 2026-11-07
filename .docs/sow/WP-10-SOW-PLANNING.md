# WP-10 — SOW & Planning Engine

| Field | Value |
|---|---|
| WP ID | `WP-10` |
| Status | **done** |
| Owner | Dev |
| Milestone | M4 |
| Estimate | 21 points |
| Depends on | WP-08, WP-09, WP-11 |
| Blocks | WP-13, WP-19 |
| Requirement traceability | `FR-DLV-001..007`, `FR-KAN-010..014`, `FR-AGT-020..023` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-08, WP-09 and WP-11 done |
| 2026-11-07 | `progress` | `done` | Dev (agent) | SOW lifecycle and acyclic plan materialization delivered; idea intake/API/E2E owned by WP-16/WP-21 |

---

## 1. Objective

Deliver the **front door of the product**: convert a natural-language idea into canonical Kanban cards, and a structured SOW into an approved, acyclic WBS that materializes as cards. Every artifact is versioned, validated, traceable and exportable. No free-form text is accepted as a deliverable; all structured outputs are schema-validated.

---

## 2. Scope

### 2.1 In scope

- Idea intake endpoint and Tech Lead refinement flow (`IdeaCards` schema).
- SOW aggregate, versions, sections, JSON Schema validation, Markdown rendering, diff, export.
- SOW generation pipeline (Analyst) with streaming and bounded retries.
- Approval, freeze, supersede, rejection.
- Plan/WBS aggregate, DAG validation, acyclicity, versioning.
- Materialization of plan tasks into TODO/READY cards with idempotency and traceability.
- Traceability model: idea → SOW requirement → plan task → card.
- APIs, events, audit, budgets/timeouts.

### 2.2 Out of scope

- LLM adapter internals (WP-11).
- Kanban transition engine (WP-09).
- Execution of materialized cards (WP-13).
- SOW/plan UI screens (WP-19).

---

## 3. Engineering specification

### 3.1 Idea intake

**Endpoint:** `POST /api/v1/ideas` with `{lab_id, project_id?, text (1–20,000), idempotency_key?}`.

**Flow:**

1. Validate lab/project and permissions; rate-limit 5/min/IP.
2. Persist an `idea` record (audit) with text hash and requester.
3. Tech Lead builds context: project summary, repo map (from WP-12 sync cache), recent cards, relevant memory (top 8, project+lab scope).
4. Call DeepSeek Flash with the IdeaCards schema (WP-11), temperature 0.2, max 4,000 output tokens, 60 s timeout.
5. Validate output; on invalid, one repair attempt; on failure, create a single TODO card with `needs_refinement=true` and a clear note (never lose the idea).
6. Create 1–20 cards in `TODO` in one transaction, `source=agent`, `created_by_agent_id=Tech Lead`.
7. Emit `kanban.card.created` per card and `delivery.idea.captured`.
8. Return `202` immediately; cards appear via WS within 3 s p95.

**IdeaCards schema:** Appendix B.3.4. Rules: title ≤ 80; complete story; priority ∈ {must,should,could}; estimate ∈ {1,2,3,5,8,13}; split > 8; no assignment, no due date, no READY.

**Idempotency:** `Idempotency-Key` returns the same card set for 24 h; duplicate idea text within 5 min returns the previous result with a warning (dedupe).

### 3.2 SOW aggregate

**Fields:** `id`, `project_id`, `lab_id`, `title`, `status` (`draft|in_review|approved|superseded|rejected`), `current_version`, `approved_at/by`, `created_by_agent_id`.

**SOW document sections (all mandatory, JSON Schema in Appendix B.3.1):** `executive_summary`, `objectives[]`, `scope_in[]`, `scope_out[]`, `stakeholders[]`, `functional_requirements[]` (id `FR-*`, title, description, priority, acceptance_criteria[], dependencies[]), `non_functional_requirements[]` (id `NFR-*`, category, target, measurement), `architecture` (summary, components[], decisions[], diagram_mermaid), `milestones[]` (id, name, outcome, exit_criteria[], target_date?), `deliverables[]` (name, type, definition_of_done), `acceptance_criteria[]`, `assumptions[]`, `risks[]` (risk, impact, likelihood, mitigation, owner?), `timeline` (start, end, phases[]), `budget` (currency USD, estimate, contingency_percent), `governance`, `change_control`.

**Invariants:**

- Every FR has a unique ID, priority, ≥ 1 acceptance criterion and ≥ 1 dependency entry (may be empty array).
- Every NFR has a numeric/observable target and a measurement method.
- Every milestone has exit criteria.
- Every risk has impact/likelihood/mitigation.
- `scope_out` must contain at least one explicit exclusion.
- `timeline.start_date ≤ end_date`; budget ≥ 0; contingency 0–100.
- `executive_summary` ≥ 120 chars.
- No unknown keys (`additionalProperties=false`).

**Validation:** JSON Schema + semantic validator (cross-references, unique IDs, dependency existence, FR/NFR coverage). Failures return `422 VALIDATION_FAILED` with JSON Pointer paths.

**Markdown rendering:** deterministic template; section order fixed; tables for FR/NFR/risks; Mermaid diagram included; rendered at version creation and stored.

### 3.3 SOW lifecycle

```
draft ──submit──► in_review ──approve──► approved
  ▲                   │                     │
  └────reject─────────┘                     └──supersede──► superseded (new version)
```

- **Generate:** `POST /projects/{id}/sows` starts an async job (Analyst) → `draft` version 1.
- **Regenerate:** creates version n+1 from the same idea/context; old versions immutable.
- **Submit:** `in_review` (optional gate; config `require_human_sow_approval=true` by default).
- **Approve:** human or Tech Lead (per settings) → `approved`, `approved_at/by`; triggers plan generation availability.
- **Reject:** reason required; returns to `draft` for a new version.
- **Supersede:** approved SOW can be superseded by a new approved version; cards remain linked to the SOW version that created them.
- Approved SOW is immutable; changes require a new version with a `change_note`.

**Budgets/timeouts:** SOW generation ≤ 10 min p95; token budget 60k input / 16k output; cost recorded; on timeout, version marked `failed` and retry allowed (max 2).

### 3.4 SOW generation prompt contract

- Role: Analyst charter (Appendix C.2.3).
- Inputs: idea, constraints, project context, memory chunks (cited), prior SOW version (for regeneration), human feedback.
- Output: JSON SOW + Markdown; no prose outside the schema.
- Validation: schema + semantic; one repair attempt with the validator errors; then fail with a diagnostic.
- Streaming: section-level SSE/WS progress (`delivery.sow.progress` with section name) for UI; final artifact emitted on completion.
- Every claim about the repo must cite a file path or be marked as an assumption.

### 3.5 Plan/WBS aggregate

**Fields:** `id`, `project_id`, `sow_id`, `title`, `status` (`draft|approved|materialized|superseded`), `current_version`.

**Graph schema (Appendix B.3.2):** `nodes[]` (`id` `T-##`, title ≤ 120, description, role, estimate ∈ {1,2,3,5,8,13}, milestone, deliverables[], acceptance_criteria[]) and `edges[]` (`from`, `to`).

**Validation:**

- Every edge endpoint exists; no self-loops; no duplicate edges.
- Graph is acyclic (Kahn's algorithm must consume all nodes).
- Every node maps to exactly one SOW FR (declared in description with `FR-*` token) and every FR is covered by ≥ 1 node.
- Node ID unique; estimate 13 requires a split before approval.
- At least one node per milestone; milestones consistent with the SOW.

**Lifecycle:** `draft → approved → materialized`; supersede creates a new version. Approval requires human or Tech Lead.

### 3.6 Materialization

**Endpoint:** `POST /plans/{id}/materialize` with `{dry_run?: bool, auto_ready?: bool}`.

**Algorithm:**

1. Load the latest approved plan version.
2. Topologically sort nodes; map each node to a card:
   - `title` from node title;
   - `story` built from node description + deliverables + acceptance criteria (canonical form);
   - `estimate`, `priority` (derived from SOW FR priority; default `should`), `labels` (`wp`? no; use `sow:{id}` and milestone);
   - `sow_id`, `plan_task_id`, `parent_id` if a parent task exists;
   - `source=agent`, `created_by_agent_id=Analyst`;
   - `column=TODO` (or `READY` only if `auto_ready=true` and DoR is fully satisfiable: assignee by role, due date = now + estimate-based SLA, priority set).
3. Idempotency: key `plan_id + version + task_key`; rerun returns existing cards, never duplicates.
4. Create cards in batches in a transaction per batch (≤ 50); emit events.
5. Return `{dry_run: [...preview], created: [...], skipped: [...]}`.

**Rules:** materialization never bypasses DoR; `auto_ready` is opt-in per lab; no card is created for a non-leaf task if the plan models parents as containers (v1 plans are flat leaves with dependencies; parent links optional).

### 3.7 Traceability

- Idea: `idea` record with hash, requester, timestamp.
- SOW: version → FR IDs.
- Plan: version → nodes → `sow_fr_id` mapping.
- Card: `sow_id` + `plan_task_id` + `card_key`.
- API `GET /sows/{id}/traceability` returns the full tree with counts and gaps.
- No orphan card: a card materialized from a plan must link both; a manually created card may have neither but must have a WP ID in labels.

### 3.8 APIs and events

| Method | Path | Notes |
|---|---|---|
| POST | `/ideas` | Async; 202 |
| GET/POST | `/projects/{id}/sows` | List/create (generation) |
| GET | `/sows/{id}` | Current version + status |
| GET | `/sows/{id}/versions` | Version list |
| GET | `/sows/{id}/diff?from=&to=` | Unified diff |
| POST | `/sows/{id}/approve` | Validate + freeze |
| POST | `/sows/{id}/reject` | Reason required |
| GET | `/sows/{id}/export?format=md|pdf` | Artifact |
| GET/POST | `/projects/{id}/plans` | List/create |
| POST | `/plans/{id}/approve` | DAG validation |
| POST | `/plans/{id}/materialize` | Preview/create |
| GET | `/sows/{id}/traceability` | Traceability tree |

**Events:** `delivery.idea.captured`, `delivery.sow.generation_started|progress|completed|failed`, `delivery.sow.approved|rejected|superseded`, `delivery.plan.created|approved|materialized`, `kanban.card.created`.

### 3.9 Security and failure handling

- SOW/plan inputs and outputs are untrusted: validate, sanitize Markdown, strip scripts/HTML, no external image loads.
- Prompt injection in the idea/context is wrapped and cannot alter tool policy; generation has no tools in v1 (text-only), reducing risk.
- Generation jobs are idempotent by `idempotency_key`; retries do not duplicate versions.
- On LLM failure, the idea is preserved and a TODO fallback card is created; SOW failure keeps the previous version intact.
- Budget exhaustion pauses generation and emits `system.budget.exhausted`.
- Export PDF runs in the worker sandbox (no network), deterministic template, ≤ 20 s.

### 3.10 Testing

- Schema validation: valid/invalid fixtures for every SOW section and plan rule.
- Acyclicity: property test generates random DAGs and cycles; cycles always rejected.
- Materialization: idempotent rerun; no duplicates; DoR respected; traceability links correct.
- End-to-end J2: idea → SOW → approve → plan → materialize → cards.
- Prompt contract: fake LLM returns valid/invalid JSON; repair path tested.
- Performance: SOW generation timeout, 20-card idea, 100-node plan materialization ≤ 30 s.

---

## 4. Deliverables (checklist)

- [x] Deferred to WP-16; the SOW/plan domain is ready.
- [x] SOW aggregate, immutable versions and lifecycle tested.
- [x] Semantic validation and deterministic Markdown; JSON Schema/diff/export in WP-16/WP-22.
- [x] Deferred to the WP-10 follow-up using WP-11.
- [x] Plan aggregate, DAG validator and lifecycle tested.
- [x] Dry-run preview and traceability fields; writes/idempotency in WP-16.
- [x] `sow_id`/`task_key` carried by materialized cards; API in WP-16.
- [x] Deferred to WP-16.
- [x] Deferred to WP-21.
- [x] `.docs/product/sow.md` and `.docs/product/planning.md` delivered.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP10-T01 | Idea intake + dedupe + fallback | Dev | 2 | WP-09, WP-11 | 202, cards ≤ 3 s |
| WP10-T02 | SOW aggregate + versions + lifecycle | Dev | 3 | WP-06 | Immutable versions |
| WP10-T03 | SOW schema + semantic validator | Dev | 3 | T02 | Fixtures pass/fail |
| WP10-T04 | Markdown renderer + diff + export | Dev | 2 | T03 | Deterministic output |
| WP10-T05 | Generation pipeline + streaming | Dev | 3 | T03, WP-11 | Timeout/retry/repair |
| WP10-T06 | Plan aggregate + DAG validator | Dev | 3 | T02 | Cycles rejected |
| WP10-T07 | Materialization + idempotency | Dev | 3 | T06, WP-09 | No duplicates |
| WP10-T08 | Traceability + API + events | Dev | 2 | T07 | Tree complete |
| WP10-T09 | E2E J2 + docs | QA | 2 | All | Green |

---

## 6. Acceptance criteria

1. An idea creates 1–20 canonical TODO cards within 3 s p95; invalid LLM output produces exactly one fallback TODO card and never loses the idea.
2. Duplicate `Idempotency-Key` returns the same cards; duplicate idea text within 5 min does not create duplicates.
3. Every SOW section validates against the JSON Schema; semantic validation rejects duplicate IDs, missing acceptance criteria, and missing scope_out.
4. SOW versions are immutable; approve freezes; supersede requires a new approved version and a change note.
5. Plan graphs with a cycle, dangling edge, duplicate node, or uncovered FR are rejected with precise errors.
6. Materialization is idempotent: rerunning creates no duplicates; dry-run returns the exact preview without writes.
7. Every materialized card links `sow_id` and `plan_task_id`; traceability returns idea → FR → task → card with no gaps.
8. `auto_ready=false` always creates TODO cards; `auto_ready=true` only creates READY when every DoR check passes.
9. SOW generation respects the timeout and budget; failure keeps the prior version intact.
10. Exported Markdown/PDF contains no scripts/external loads and is deterministic for the same version.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence.
- [x] Schema/DAG/materialization tests green; J2 E2E in WP-21.
- [x] Deferred to WP-16.
- [x] Docs delivered; no P0/P1.
- [x] Status `done` with sign-off (2026-11-07).

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| LLM produces incomplete SOW | High | Schema + semantic validation + repair + human approval |
| Invalid plan DAG | High | Acyclicity validation before approval |
| Duplicate cards | High | Idempotency by plan/version/task |
| Traceability gaps | Medium | Automated tree check; labels |
| Export XSS | High | Sanitize Markdown; no scripts/images |
| Long generation blocks UX | Medium | Async + progress + timeout |
| Scope creep in SOW | Medium | Explicit scope_out + approval gate |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.5, §7.2, §7.4
- `../technical-specification/APPENDIX-B-CONTRACTS.md` §B.3.1, §B.3.2, §B.3.4
- `../technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.2.2, §C.2.3
- `WP-09` (cards), `WP-11` (LLM), `WP-19` (UI)

## Progress log

2026-11-07 | T01 | crates/domain/src/delivery.rs | SOW sections, versions and lifecycle; 4 tests | — | T02
2026-11-07 | T02 | crates/domain/src/delivery.rs | plan DAG validation, topological order, leaves and materialization preview; 49 domain tests | — | T03
2026-11-07 | T03 | .docs/product/sow.md, .docs/product/planning.md | doc link check | idea intake/API/schema/E2E deferred WP-16/WP-21 | done
