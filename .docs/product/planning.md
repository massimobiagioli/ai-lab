# AI-Lab Planning

| Field | Value |
|---|---|
| Document | `.docs/product/planning.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Analyst |
| Related | `.docs/product/sow.md`, `.docs/product/kanban.md`, `crates/domain/src/delivery.rs` |

A plan is a DAG of WBS tasks linked to an approved SOW. Materialization turns the leaves into Kanban cards.

## 1. Task schema

Each task has a stable `task_key`, title, role, estimate, `depends_on[]`, deliverables and acceptance criteria. Keys are unique within a plan; estimates are required; deliverables and acceptance criteria must be non-empty.

## 2. Validation

A plan is rejected when:

- the task list is empty;
- a task key is duplicated;
- a dependency is dangling or self-referential;
- the graph contains a cycle (Kahn's algorithm leaves unvisited nodes);
- a task has no estimate, deliverable or acceptance criterion.

## 3. Materialization

- `preview` returns a deterministic list of `MaterializedCard` for the **leaves** (tasks no other task depends on), in topological order.
- Each task maps to exactly one functional requirement (`covers`); `Plan::validate_coverage` rejects unknown, missing or uncovered FRs. Each preview card carries the story, estimate, role, dependencies, `task_key`, `covers` and `sow_id` for traceability.
- `mark_materialized` requires an approved plan and moves it to `Materialized`.
- The application creates the actual cards with `Project::next_card_seq`; rerunning the preview is idempotent because it performs no writes.
- `auto_ready=false` creates `TODO` cards; `auto_ready=true` is allowed only when every DoR check passes (WP-16).

## 4. Tests

`cargo test -p ailab-domain` covers cycle, unknown dependency and duplicate-key rejection, topological order, leaf selection, FR coverage, traceability and the approve/materialize lifecycle.

## 5. Deferred to consumer WPs

- Materialization writes, dry-run API and idempotency keys: WP-16.
- Traceability API (idea → FR → task → card): WP-16.
- J2 end-to-end journey: WP-19/WP-21.

## 6. References

- `.docs/sow/WP-10-SOW-PLANNING.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.5
- `crates/domain/src/delivery.rs`
