# AI-Lab Data Ownership

| Field | Value |
|---|---|
| Document | `.docs/product/data-ownership.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/domain-model.md`, `.docs/product/architecture.md`, `.docs/DATA-MODEL.md` |
| Enforcement | `.tools/check-architecture-fitness.sh`, CI `rust-integration` (when crates land) |

Every table has exactly one owning context. Other contexts may hold a reference ID but MUST NOT join or mutate the owned table. This document is the normative ownership map; the fitness check enforces it.

---

## 1. Ownership matrix

| Table | Owner | Readers via port/view | Writers |
|---|---|---|---|
| `workspaces` | Platform | Lab, Inventory, Kanban, Delivery, API | Platform |
| `labs` | Lab | Inventory, Kanban, Delivery, API | Lab |
| `providers`, `models` | Inventory | LLM, API | Inventory |
| `agents`, `agent_cv_versions`, `characters` | Inventory | Lab, Kanban, Delivery, Memory, Office | Inventory |
| `projects`, `scm_credentials` | Inventory | Kanban, Delivery, SCM | Inventory |
| `boards`, `cards`, `card_events`, `card_comments` | Kanban | Delivery, API, Office | Kanban |
| `sows`, `sow_versions`, `plans`, `plan_versions`, `plan_tasks` | Delivery | Kanban (materialization), API | Delivery |
| `runs`, `run_steps`, `decisions` | Delivery | Kanban (projection), Memory (reflection), API | Delivery |
| `memory_documents`, `memory_chunks`, `memory_episodes` | Memory | Delivery (retrieval), API | Memory |
| `outbox`, `domain_events` | Platform | Relay, API | Any context (same transaction) |
| `audit_log`, `app_settings`, `cost_ledger`, `llm_requests`, `maintenance_jobs` | Platform | API, Ops | Platform |
| `github_webhook_deliveries` | SCM integration | Platform | SCM |

Read-only analytics views in the `analytics` schema are owned by Platform and may join across contexts; they never write.

## 2. Forbidden access rules

1. A repository in context A MUST NOT issue SQL against a table owned by context B.
2. A context MUST NOT import another context's repository or persistence model.
3. Cross-context data is obtained through an application query port, a domain event or an approved read-only analytics view.
4. Write side effects across contexts use the transactional outbox, never a direct write.
5. Foreign-key references are allowed; cascading deletes across context-owned tables require an explicit Platform operation.
6. Reporting joins belong to `analytics.*` only.

## 3. Approved read paths

| Consumer | Needs | Approved path |
|---|---|---|
| Kanban | agent name/role/character | Inventory query port `AgentSummary` |
| Delivery | project key/default branch | Inventory query port `ProjectSummary` |
| Delivery | card key/column | Kanban query port `CardSummary` |
| Memory reflection | run outcome, card evidence | Delivery event `delivery.run.completed` |
| API projections | board, dashboard, inspector | read-only `analytics.*` or context query ports |
| Office | agent presence and card state | API projection of Kanban + Lab events |

## 4. Code convention (for the fitness check)

- Context-owned code lives in a path segment equal to one of: `inventory`, `lab`, `memory`, `kanban`, `delivery`, `scm`, `platform`.
- The recommended repository layout is `crates/infrastructure/src/<context>/...` and `crates/domain/src/<context>/...`. `.tools/architecture/paths.tsv` maps additional prefixes to contexts; the longest prefix wins, then the path-segment rule applies.
- Shared crates (`domain`, `application`, `infrastructure`, `llm`, `scm`, `memory` engine, `telemetry`) may reference tables only through the owning context's repository or an approved port.
- SQL table references are detected in Rust source (including schema-qualified, quoted and multi-line SQL) by matching `FROM`, `JOIN`, `INTO`, `UPDATE` and `DELETE FROM` targets against the table names in `.tools/architecture/contexts.tsv`.
- A file attributed to a context that references another context's table is a violation.
- SQL table references under `crates/infrastructure/` that cannot be attributed to a context are a violation; add a `paths.tsv` prefix or move the file under a context module. This closes the shared-adapter false negative.

## 5. Manifest

`.tools/architecture/contexts.tsv` is the machine-readable ownership map:

```
context<TAB>owner<TAB>table1,table2,...
```

`.tools/architecture/dependencies.tsv` is the allowed compile-time context dependency graph (event subscriptions are asynchronous and excluded):

```
from<TAB>to
```

`.tools/architecture/paths.tsv` maps path prefixes to contexts for the fitness scan:

```
path-prefix<TAB>context
```

The fitness check validates:

1. Every context has exactly one owner.
2. No table is owned by two contexts.
3. Every dependency and path prefix references a known context.
4. The dependency graph is acyclic.
5. No context-attributed Rust file references another context's table.
6. No unattributed SQL exists under `crates/infrastructure/`.
7. When a Compose file exists (including `infra/compose/docker-compose*.yml`), every Compose service appears in `.docs/product/service-catalog.md`.

## 6. Enforcement and exceptions

- Run `.tools/check-architecture-fitness.sh` in CI. It runs in `docs-build` today and must also run in `rust-integration` once crates land.
- A legitimate exception requires an ADR and an entry in this document; the test fails until the exception is encoded and reviewed.
- A violation is a P1 review defect; it is not waived silently.

## 7. References

- `.docs/technical-specification/APPENDIX-A-DATABASE.sql`
- `.docs/DATA-MODEL.md`
- `.docs/sow/WP-01-ARCHITECTURE.md` §3.5–§3.6
- `.docs/sow/WP-06-DOMAIN-KERNEL.md`
- `.docs/product/domain-model.md`
