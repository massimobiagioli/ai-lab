# AI-Lab Product Documentation

This directory holds product documentation. Program control (technical specification, SOW, WP contracts, ADRs) lives in `.docs/`.

| Document | Purpose |
|---|---|
| [architecture.md](./architecture.md) | C4 context/container/component, trust boundaries, data flows, ADR index |
| [eventing.md](./eventing.md) | Event envelope, catalog, outbox, relay, sequencing and idempotency |
| [inventory.md](./inventory.md) | Provider, model, SCM credential, agent, CV and project invariants |
| [memory.md](./memory.md) | Lab settings, memory scopes, chunking, hybrid retrieval and compaction |
| [kanban.md](./kanban.md) | Transition matrix, guards, DoR/DoD and error codes |
| [llm.md](./llm.md) | Provider port, routing, prompt assembly, structured outputs, budgets |
| [agents.md](./agents.md) | Roles, prompt order, routing, guardrails and CV |
| [sow.md](./sow.md) | SOW sections, lifecycle, versions and rendering |
| [planning.md](./planning.md) | WBS DAG validation, topological order and materialization |
| [github.md](./github.md) | SCM port, GitHub conventions, checks, merge and webhooks |
| [sandbox.md](./sandbox.md) | Sandbox spec, isolation, egress and lifecycle |
| [tools.md](./tools.md) | Tool policy allowlists, denials and protected paths |
| [realtime.md](./realtime.md) | WebSocket envelope, resume, backfill and backpressure |
| [orchestration.md](./orchestration.md) | Scheduler eligibility, leases, run state, retries and recovery |
| [qa.md](./qa.md) | Review verdicts, separation of duties and autonomy merge policy |
| [qa/WP-24-FRONTEND-HARDENING-qa-report.md](./qa/WP-24-FRONTEND-HARDENING-qa-report.md) | WP-24 QA verdict (NOT APPROVED) and the coverage gaps |
| [docs-manifest.md](./docs-manifest.md) | Documentation manifest, artifact status and the DOC-GATE open items |
| [acceptance-report.md](./acceptance-report.md) | AC-001..015 evidence map and the pending sign-off |
| [handover.md](./handover.md) | Operator handover, ownership, limitations and support boundaries |
| [testing.md](./testing.md) | Test commands, fixtures, seed, flaky policy and coverage status |
| [qa/WP-22-DOCS-RELEASE-qa-report.md](./qa/WP-22-DOCS-RELEASE-qa-report.md) | WP-22 QA verdict (NOT APPROVED) and the release gaps |
| [qa/WP-23-MAINTENANCE-qa-report.md](./qa/WP-23-MAINTENANCE-qa-report.md) | WP-23 QA verdict (NOT APPROVED) and the open maintenance handlers |
| [qa-merge.md](./qa-merge.md) | Review pipeline, merge conditions, hard stops and error codes |
| [frontend.md](./frontend.md) | Angular shell, pixel tokens, status badge and accessibility |
| [design-system.md](./design-system.md) | Pixel design tokens, UI primitives and accessibility rules |
| [office.md](./office.md) | Canvas office, A* pathfinding, state mapping and accessibility |
| [product-ui.md](./product-ui.md) | Dashboard, surfaces, UX states and accessibility |
| [ui.md](./ui.md) | Product routes, states and interaction rules |
| [observability.md](./observability.md) | Logs, metrics, traces, SLOs, alerts and backup |
| [testing.md](./testing.md) | Test layers, commands, fixtures, coverage and flaky policy |
| [demo-script.md](./demo-script.md) | Golden demo with timings and expected output |
| [glossary.md](./glossary.md) | Domain vocabulary |
| [release-checklist.md](./release-checklist.md) | Release gates, artifacts, acceptance and rollback |
| [maintenance.md](./maintenance.md) | Maintenance jobs, dependency policy, CV reflection and memory hygiene |
| [domain-model.md](./domain-model.md) | Bounded contexts, aggregates, invariants, event catalog |
| [service-catalog.md](./service-catalog.md) | Runtime units, scaling, data ownership, SLOs |
| [contracts.md](./contracts.md) | Communication matrix, runner control-plane API, edge contracts |
| [data-ownership.md](./data-ownership.md) | Table ownership, forbidden access, fitness enforcement |
| [failure-modes.md](./failure-modes.md) | Failure matrix, recovery procedures, RTO/RPO |
| [security.md](./security.md) | Threat model, exposure modes, crypto, controls, incident response |
| [security/](./security/) | Abuse cases, ASVS L2 checklist, audit coverage matrix |
| [database.md](./database.md) | ERD, table dictionary, index rationale, migration policy, retention |
| [diagrams/](./diagrams/) | Mermaid sources and rendered SVG exports |
| [governance.md](./governance.md) | Lean/Agile delivery model, roles, RACI, ceremonies, escalation |
| [status-model.md](./status-model.md) | `ready`/`progress`/`done`, blocked flag, DoR/DoD, evidence |
| [quality-gates.md](./quality-gates.md) | Required CI checks, owners, thresholds, failure actions |
| [metrics.md](./metrics.md) | Flow metric formulas, sources, targets, dashboard mapping |
| [change-control.md](./change-control.md) | CR form, approval matrix, emergency path, records |
| [risk-register.md](./risk-register.md) | Program and WP risk rollup with owners and review dates |
| [review-checklist.md](./review-checklist.md) | Correctness, scope, tests, security, performance and docs review checklist |
| [api/](./api/) | OpenAPI and AsyncAPI contracts (WP-16) |
| [runbooks/](./runbooks/) | Operational runbooks (WP-20) |
| [templates/](./templates/) | WP, card, ADR, CR, milestone gate and retrospective templates |
| [change-requests/](./change-requests/) | Change request register and records |
| [retrospectives/](./retrospectives/) | Milestone retrospective records |
| [backlog/](./backlog/) | `TODO` cards created by retrospectives before the Kanban engine lands |
| [evidence/](./evidence/) | Independent review records and acceptance evidence |

Conventions:

1. One canonical location per fact.
2. English only.
3. Relative links must resolve; a broken link blocks release.
4. Product behavior described here must match the normative specification; when it does not, the specification wins and this documentation is corrected.
