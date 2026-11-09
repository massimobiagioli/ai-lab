# AGENTS.md — AI-Lab Agent Operating Manual

This file is the entry point for every AI agent working on AI-Lab. It contains **instructions only**. The human quickstart lives in [`README.md`](./README.md); do not duplicate it here.

---

## Read order

Read these in order before doing any work:

1. `AGENTS.md` (this file).
2. [`.docs/CONSTITUTION.md`](./.docs/CONSTITUTION.md).
3. [`.docs/PRINCIPLES.md`](./.docs/PRINCIPLES.md).
4. [`.docs/STACK.md`](./.docs/STACK.md).
5. [`.docs/BACKEND-INSTRUCTIONS.md`](./.docs/BACKEND-INSTRUCTIONS.md) for backend work, or [`.docs/FRONTEND-INSTRUCTIONS.md`](./.docs/FRONTEND-INSTRUCTIONS.md) for frontend work.
6. [`.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md`](./.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md) and its appendices.
7. [`.docs/sow/SOW.md`](./.docs/sow/SOW.md) and the target work package.
8. [`.docs/adr/registry.md`](./.docs/adr/registry.md) and any referenced ADR.
9. The specialist skill for the task (Skills index below).

Before changing any entity, attribute, relationship, logical index, retention rule or migration, read [`.docs/DATA-MODEL.md`](./.docs/DATA-MODEL.md).

Do not start a task if any of these files is missing or unreadable. Record the gap in the work package progress log and stop.

---

## Source of truth and conflict resolution

When sources disagree, the higher source wins and the lower one must be corrected:

1. `.docs/technical-specification/` — normative product contract.
2. `.docs/sow/SOW.md` and the target `WP-XX-*.md` — delivery contract.
3. `.docs/CONSTITUTION.md`, `.docs/PRINCIPLES.md`, `.docs/STACK.md` — project-wide rules.
4. `.docs/adr/` — accepted decisions; a later accepted ADR supersedes an earlier one.
5. `.opencode/skills/` — task workflows.
6. `AGENTS.md` — navigation and operating instructions.

If a conflict is unresolved, stop implementation, append the conflict to the work package progress log, and create an ADR or change request. Never silently choose.

---

## Operating loop

1. Select the next task with `/impl-task` or the next work package with `/impl-wp`.
2. Read the task, its acceptance criteria, its dependencies and the relevant spec sections.
3. Follow the TDD protocol: **Red → Green → Refactor**.
4. Update only the verified task row and deliverables in the work package.
5. Append a progress line: `YYYY-MM-DD | Tnn | files | tests | decision/ADR | next`.
6. Create an ADR for every architectural or irreversible decision.
7. Update the documentation and skill indexes in this file when they change.
8. Run `scripts/harness-lint.sh` and the quality gates.
9. Report the task, status, files, tests, evidence, ADRs and next step.

Do not commit or push unless the command is invoked with `--commit`. If `--commit` is present, use Conventional Commits and the AI-Lab trailers.

---

## Skills index

Load the exact skill with the `skill` tool by its ID. Skills are project-specific; do not substitute generic advice.

| Skill ID | When to use | Path |
|---|---|---|
| `wp-delivery` | Selecting, executing, updating or closing a work package or task. | [`.opencode/skills/wp-delivery/SKILL.md`](./.opencode/skills/wp-delivery/SKILL.md) |
| `tdd-rust` | Any Rust backend change: domain, application, adapters, worker, API, CLI. | [`.opencode/skills/tdd-rust/SKILL.md`](./.opencode/skills/tdd-rust/SKILL.md) |
| `tdd-angular` | Any Angular change: components, stores, services, office host, Playwright. | [`.opencode/skills/tdd-angular/SKILL.md`](./.opencode/skills/tdd-angular/SKILL.md) |
| `security-first` | Auth, secrets, sandbox, prompts, SCM, webhooks, exposure, dependencies. | [`.opencode/skills/security-first/SKILL.md`](./.opencode/skills/security-first/SKILL.md) |
| `kanban-governance` | Creating, refining, moving, reviewing or closing cards; DoR/DoD/WIP. | [`.opencode/skills/kanban-governance/SKILL.md`](./.opencode/skills/kanban-governance/SKILL.md) |
| `adr-authoring` | Architectural or irreversible decisions; superseding an ADR. | [`.opencode/skills/adr-authoring/SKILL.md`](./.opencode/skills/adr-authoring/SKILL.md) |
| `api-contracts` | REST, DTO, OpenAPI, WebSocket, AsyncAPI and compatibility changes. | [`.opencode/skills/api-contracts/SKILL.md`](./.opencode/skills/api-contracts/SKILL.md) |
| `data-migrations` | PostgreSQL schema, queries, pgvector, Redis keys, retention, integrity. | [`.opencode/skills/data-migrations/SKILL.md`](./.opencode/skills/data-migrations/SKILL.md) |
| `agent-prompts` | Prompt assembly, model routing, structured outputs, tool schemas, budgets. | [`.opencode/skills/agent-prompts/SKILL.md`](./.opencode/skills/agent-prompts/SKILL.md) |
| `docker-operations` | Dockerfiles, Compose, networks, healthchecks, runner, sandbox, proxies. | [`.opencode/skills/docker-operations/SKILL.md`](./.opencode/skills/docker-operations/SKILL.md) |
| `observability-sre` | Metrics, traces, logs, SLOs, alerts, dashboards, backup, runbooks. | [`.opencode/skills/observability-sre/SKILL.md`](./.opencode/skills/observability-sre/SKILL.md) |
| `quality-gates` | Before declaring done; CI, tests, coverage, load, chaos, fuzz. | [`.opencode/skills/quality-gates/SKILL.md`](./.opencode/skills/quality-gates/SKILL.md) |
| `bdd-e2e` | Gherkin BDD E2E suite, feature generation, steps, preflight gate and `make bdd`. | [`.opencode/skills/bdd-e2e/SKILL.md`](./.opencode/skills/bdd-e2e/SKILL.md) |
| `bdd-scenario-design` | Writing, refining and reviewing Gherkin scenarios, requirement mapping and gap closure. | [`.opencode/skills/bdd-scenario-design/SKILL.md`](./.opencode/skills/bdd-scenario-design/SKILL.md) |
| `bdd-step-engineering` | Step definitions, fixtures, selectors, API client, determinism and redaction. | [`.opencode/skills/bdd-step-engineering/SKILL.md`](./.opencode/skills/bdd-step-engineering/SKILL.md) |
| `bdd-preflight-gate` | Manual prerequisites checklist, make-only guard, preflight parser and fail-fast messages. | [`.opencode/skills/bdd-preflight-gate/SKILL.md`](./.opencode/skills/bdd-preflight-gate/SKILL.md) |
| `bdd-coverage-traceability` | 480-id coverage, feature generation, drift gates, manual/live mapping and evidence. | [`.opencode/skills/bdd-coverage-traceability/SKILL.md`](./.opencode/skills/bdd-coverage-traceability/SKILL.md) |
| `bdd-execution-triage` | Running `make bdd`, interpreting results, failure triage, flaky quarantine and evidence. | [`.opencode/skills/bdd-execution-triage/SKILL.md`](./.opencode/skills/bdd-execution-triage/SKILL.md) |
| `release-management` | Versioning, changelog, release candidate, SBOM, signing, rollback. | [`.opencode/skills/release-management/SKILL.md`](./.opencode/skills/release-management/SKILL.md) |
| `docs-sync` | Docs, skills, commands, ADRs, diagrams, screenshots and index updates. | [`.opencode/skills/docs-sync/SKILL.md`](./.opencode/skills/docs-sync/SKILL.md) |

---

## Documentation index

Every document under `.docs` is listed here. When a document is created, renamed or removed, update this table in the same change.

| Document | Purpose | Path |
|---|---|---|
| Constitution | Project non-negotiables, decision rights, change control, source-of-truth order. | [`.docs/CONSTITUTION.md`](./.docs/CONSTITUTION.md) |
| Principles | TDD, Security First, Clean Code, Agile/Lean, DDD, UX and their enforcement. | [`.docs/PRINCIPLES.md`](./.docs/PRINCIPLES.md) |
| Stack | Pinned versions, architecture, repository layout, commands, constraints. | [`.docs/STACK.md`](./.docs/STACK.md) |
| Backend instructions | Backend rules: architecture, Rust, data, API, security, testing, observability. | [`.docs/BACKEND-INSTRUCTIONS.md`](./.docs/BACKEND-INSTRUCTIONS.md) |
| Frontend instructions | Frontend rules: Angular, state, API/WS, design system, office, accessibility, testing. | [`.docs/FRONTEND-INSTRUCTIONS.md`](./.docs/FRONTEND-INSTRUCTIONS.md) |
| Data model | Logical entities, attributes, relationships, access paths and cross-context interactions. | [`.docs/DATA-MODEL.md`](./.docs/DATA-MODEL.md) |
| ADR registry | Index of all architecture decisions and their status. | [`.docs/adr/registry.md`](./.docs/adr/registry.md) |
| Technical specification | Normative product specification. | [`.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md`](./.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md) |
| Database schema | Canonical PostgreSQL 18 + pgvector DDL. | [`.docs/technical-specification/APPENDIX-A-DATABASE.sql`](./.docs/technical-specification/APPENDIX-A-DATABASE.sql) |
| API and event contracts | OpenAPI, AsyncAPI, JSON Schemas and error codes. | [`.docs/technical-specification/APPENDIX-B-CONTRACTS.md`](./.docs/technical-specification/APPENDIX-B-CONTRACTS.md) |
| Agents and Kanban | Roles, characters, prompts, transition matrix, assets. | [`.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md`](./.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md) |
| Delivery appendix | WBS, acceptance tests, ADRs, environment variables, docs manifest. | [`.docs/technical-specification/APPENDIX-D-DELIVERY.md`](./.docs/technical-specification/APPENDIX-D-DELIVERY.md) |
| E2E BDD checklist | 480 Gherkin end-to-end scenarios, including 120 Amiga-style UI and 120 Inventory/Lab management certification scenarios, with execution checkboxes. | [`.docs/e2e/BDD.md`](./.docs/e2e/BDD.md) |
| E2E manual pre-test requirements | Manual-only pre-test checklist for credentials, external services, policy approvals, environment and sign-off. | [`.docs/e2e/req.md`](./.docs/e2e/req.md) |
| SOW index | Delivery index, status model, dependency graph, milestones. | [`.docs/sow/SOW.md`](./.docs/sow/SOW.md) |
| Review plan checklist | Ordered execution plan to finish the full WP review (2026-11-09). | [`.docs/sow/review-plan-chacklist-20261109.md`](./.docs/sow/review-plan-chacklist-20261109.md) |
| WP-00 Governance | Program governance, Lean/Agile delivery, quality system. | [`.docs/sow/WP-00-GOVERNANCE.md`](./.docs/sow/WP-00-GOVERNANCE.md) |
| WP-00 review plan | Governance review checklist and evidence. | [`.docs/sow/WP-00-GOVERNANCE-review-plan.md`](./.docs/sow/WP-00-GOVERNANCE-review-plan.md) |
| WP-01 review plan | Architecture review checklist and evidence. | [`.docs/sow/WP-01-ARCHITECTURE-review-plan.md`](./.docs/sow/WP-01-ARCHITECTURE-review-plan.md) |
| WP-01 Architecture | Architecture and service topology. | [`.docs/sow/WP-01-ARCHITECTURE.md`](./.docs/sow/WP-01-ARCHITECTURE.md) |
| WP-02 Repository/DevEx | Repository, toolchain and developer experience. | [`.docs/sow/WP-02-REPOSITORY-DEVEX.md`](./.docs/sow/WP-02-REPOSITORY-DEVEX.md) |
| WP-02 review plan | Repository and developer-experience review checklist and evidence. | [`.docs/sow/WP-02-REPOSITORY-DEVEX-review-plan.md`](./.docs/sow/WP-02-REPOSITORY-DEVEX-review-plan.md) |
| WP-03 Infrastructure | Docker and runtime platform. | [`.docs/sow/WP-03-INFRASTRUCTURE.md`](./.docs/sow/WP-03-INFRASTRUCTURE.md) |
| WP-03 review plan | Docker and runtime-platform review checklist and evidence. | [`.docs/sow/WP-03-INFRASTRUCTURE-review-plan.md`](./.docs/sow/WP-03-INFRASTRUCTURE-review-plan.md) |
| WP-04 Data platform | Schema, migrations, persistence, Redis, integrity. | [`.docs/sow/WP-04-DATA-PLATFORM.md`](./.docs/sow/WP-04-DATA-PLATFORM.md) |
| WP-04 review plan | Schema, migration and persistence review checklist and evidence. | [`.docs/sow/WP-04-DATA-PLATFORM-review-plan.md`](./.docs/sow/WP-04-DATA-PLATFORM-review-plan.md) |
| WP-05 Security | Security architecture and secrets management. | [`.docs/sow/WP-05-SECURITY.md`](./.docs/sow/WP-05-SECURITY.md) |
| WP-05 review plan | Security review checklist and evidence. | [`.docs/sow/WP-05-SECURITY-review-plan.md`](./.docs/sow/WP-05-SECURITY-review-plan.md) |
| WP-06 Domain kernel | Domain kernel and eventing. | [`.docs/sow/WP-06-DOMAIN-KERNEL.md`](./.docs/sow/WP-06-DOMAIN-KERNEL.md) |
| WP-06 review plan | Domain kernel and eventing review checklist and evidence. | [`.docs/sow/WP-06-DOMAIN-KERNEL-review-plan.md`](./.docs/sow/WP-06-DOMAIN-KERNEL-review-plan.md) |
| WP-07 Inventory | Providers, models, agents, CV, projects, credentials. | [`.docs/sow/WP-07-INVENTORY.md`](./.docs/sow/WP-07-INVENTORY.md) |
| WP-07 review plan | Inventory review checklist and evidence. | [`.docs/sow/WP-07-INVENTORY-review-plan.md`](./.docs/sow/WP-07-INVENTORY-review-plan.md) |
| WP-08 Lab and memory | Lab aggregate and three-tier memory. | [`.docs/sow/WP-08-LAB-MEMORY.md`](./.docs/sow/WP-08-LAB-MEMORY.md) |
| WP-08 review plan | Lab and memory review checklist and evidence. | [`.docs/sow/WP-08-LAB-MEMORY-review-plan.md`](./.docs/sow/WP-08-LAB-MEMORY-review-plan.md) |
| WP-09 Kanban engine | Kanban state machine, DoR/DoD, invariants. | [`.docs/sow/WP-09-KANBAN-ENGINE.md`](./.docs/sow/WP-09-KANBAN-ENGINE.md) |
| WP-09 review plan | Kanban engine review checklist and evidence. | [`.docs/sow/WP-09-KANBAN-ENGINE-review-plan.md`](./.docs/sow/WP-09-KANBAN-ENGINE-review-plan.md) |
| WP-10 SOW and planning | Idea intake, SOW, WBS, materialization. | [`.docs/sow/WP-10-SOW-PLANNING.md`](./.docs/sow/WP-10-SOW-PLANNING.md) |
| WP-10 review plan | SOW and planning review checklist and evidence. | [`.docs/sow/WP-10-SOW-PLANNING-review-plan.md`](./.docs/sow/WP-10-SOW-PLANNING-review-plan.md) |
| WP-11 LLM platform | DeepSeek adapter, prompts, routing, budgets. | [`.docs/sow/WP-11-LLM-PLATFORM.md`](./.docs/sow/WP-11-LLM-PLATFORM.md) |
| WP-11 review plan | LLM platform review checklist and evidence. | [`.docs/sow/WP-11-LLM-PLATFORM-review-plan.md`](./.docs/sow/WP-11-LLM-PLATFORM-review-plan.md) |
| WP-12 SCM and GitHub | SCM port, GitHub adapter, webhooks, merge. | [`.docs/sow/WP-12-SCM-GITHUB.md`](./.docs/sow/WP-12-SCM-GITHUB.md) |
| WP-12 review plan | SCM and GitHub review checklist and evidence. | [`.docs/sow/WP-12-SCM-GITHUB-review-plan.md`](./.docs/sow/WP-12-SCM-GITHUB-review-plan.md) |
| WP-13 Orchestration | Scheduler, leases, run pipeline, recovery. | [`.docs/sow/WP-13-ORCHESTRATION.md`](./.docs/sow/WP-13-ORCHESTRATION.md) |
| WP-13 review plan | Orchestration review checklist and evidence. | [`.docs/sow/WP-13-ORCHESTRATION-review-plan.md`](./.docs/sow/WP-13-ORCHESTRATION-review-plan.md) |
| WP-13B Backend hardening follow-up | WP-13/WP-16 residual hardening (resume, context, OTel, DoD, adapters, Redis, REST/WS, OpenAPI). | [`.docs/sow/WP-13B-FOLLOW-UP.md`](./.docs/sow/WP-13B-FOLLOW-UP.md) |
| WP-13B review plan | Backend hardening review checklist and evidence. | [`.docs/sow/WP-13B-FOLLOW-UP-review-plan.md`](./.docs/sow/WP-13B-FOLLOW-UP-review-plan.md) |
| WP-24 Frontend & QA hardening | WP-19/WP-21 residual UI, drawers and suites. | [`.docs/sow/WP-24-FRONTEND-HARDENING.md`](./.docs/sow/WP-24-FRONTEND-HARDENING.md) |
| WP-24 review plan | Frontend/QA hardening review checklist and evidence. | [`.docs/sow/WP-24-FRONTEND-HARDENING-review-plan.md`](./.docs/sow/WP-24-FRONTEND-HARDENING-review-plan.md) |
| WP-14 Sandbox runner | Sandbox isolation and tool policy. | [`.docs/sow/WP-14-SANDBOX-RUNNER.md`](./.docs/sow/WP-14-SANDBOX-RUNNER.md) |
| WP-14 review plan | Sandbox runner review checklist and evidence. | [`.docs/sow/WP-14-SANDBOX-RUNNER-review-plan.md`](./.docs/sow/WP-14-SANDBOX-RUNNER-review-plan.md) |
| WP-15 QA and merge | QA review, merge policy, autonomy. | [`.docs/sow/WP-15-QA-MERGE.md`](./.docs/sow/WP-15-QA-MERGE.md) |
| WP-15 review plan | QA, review, merge and autonomy review checklist and evidence. | [`.docs/sow/WP-15-QA-MERGE-review-plan.md`](./.docs/sow/WP-15-QA-MERGE-review-plan.md) |
| WP-16 API and realtime | REST, WebSocket, contracts, rate limits. | [`.docs/sow/WP-16-API-REALTIME.md`](./.docs/sow/WP-16-API-REALTIME.md) |
| WP-16 review plan | API and realtime review checklist and evidence. | [`.docs/sow/WP-16-API-REALTIME-review-plan.md`](./.docs/sow/WP-16-API-REALTIME-review-plan.md) |
| WP-17 Frontend platform | Angular platform, design system, accessibility. | [`.docs/sow/WP-17-FRONTEND-PLATFORM.md`](./.docs/sow/WP-17-FRONTEND-PLATFORM.md) |
| WP-17 review plan | Frontend platform review checklist and evidence. | [`.docs/sow/WP-17-FRONTEND-PLATFORM-review-plan.md`](./.docs/sow/WP-17-FRONTEND-PLATFORM-review-plan.md) |
| WP-18 Virtual office | Canvas engine, sprites, pathfinding, interactions. | [`.docs/sow/WP-18-VIRTUAL-OFFICE.md`](./.docs/sow/WP-18-VIRTUAL-OFFICE.md) |
| WP-18 review plan | Virtual office review checklist and evidence. | [`.docs/sow/WP-18-VIRTUAL-OFFICE-review-plan.md`](./.docs/sow/WP-18-VIRTUAL-OFFICE-review-plan.md) |
| WP-19 Product UI | Dashboard, inventory, Kanban, SOW, memory, settings. | [`.docs/sow/WP-19-PRODUCT-UI.md`](./.docs/sow/WP-19-PRODUCT-UI.md) |
| WP-19 review plan | Product UI review checklist and evidence. | [`.docs/sow/WP-19-PRODUCT-UI-review-plan.md`](./.docs/sow/WP-19-PRODUCT-UI-review-plan.md) |
| WP-20 Observability and SRE | Logs, metrics, traces, SLOs, backup, runbooks. | [`.docs/sow/WP-20-OBSERVABILITY-SRE.md`](./.docs/sow/WP-20-OBSERVABILITY-SRE.md) |
| WP-20 review plan | Observability and SRE review checklist and evidence. | [`.docs/sow/WP-20-OBSERVABILITY-SRE-review-plan.md`](./.docs/sow/WP-20-OBSERVABILITY-SRE-review-plan.md) |
| WP-21 Testing | Test strategy, suites, fixtures, gates. | [`.docs/sow/WP-21-TESTING.md`](./.docs/sow/WP-21-TESTING.md) |
| WP-22 Docs and release | Documentation, release, acceptance, handover. | [`.docs/sow/WP-22-DOCS-RELEASE.md`](./.docs/sow/WP-22-DOCS-RELEASE.md) |
| WP-22 review plan | Documentation, release and handover review checklist and evidence. | [`.docs/sow/WP-22-DOCS-RELEASE-review-plan.md`](./.docs/sow/WP-22-DOCS-RELEASE-review-plan.md) |
| WP-23 Maintenance | Maintenance jobs, dependency updates, memory hygiene. | [`.docs/sow/WP-23-MAINTENANCE.md`](./.docs/sow/WP-23-MAINTENANCE.md) |
| WP-23 review plan | Maintenance scheduler and handler review checklist and evidence. | [`.docs/sow/WP-23-MAINTENANCE-review-plan.md`](./.docs/sow/WP-23-MAINTENANCE-review-plan.md) |
| Product documentation index | Gateway to product documentation, governance pages, templates, CRs and backlog cards. | [`.docs/product/README.md`](./.docs/product/README.md) |

Product documentation under `.docs/product/` is indexed by `.docs/product/README.md`; individual product files are not listed here. The ADR registry is the only index for ADR files. Every accepted ADR is linked from it.

---

## ADR registry

- Registry: [`.docs/adr/registry.md`](./.docs/adr/registry.md).
- Every architectural or irreversible decision creates a new `adr-nnnn.md` and a registry row.
- Accepted ADRs are immutable; superseding requires a new ADR and status updates in both files.
- Load the `adr-authoring` skill before writing one.
- A decision recorded only in code, a comment or a work package is a review defect.

---

## Commands

| Command | Input | Behavior |
|---|---|---|
| `/impl-task` | `WP-XX`, `WP-XX/Tnn`, empty, optional `--commit` | Execute one task with Red → Green → Refactor, update status, gates and evidence. |
| `/impl-wp` | `WP-XX`, empty, optional `--commit` | Execute an entire work package task by task, update status, gates and evidence. |

Both commands load `wp-delivery` and the relevant specialist skill. Both default to no commit. Both stop on a blocker instead of guessing.

---

## Quality gates

A task or work package is not done until all applicable gates pass:

1. `scripts/harness-lint.sh`.
2. `scripts/sow-lint.sh`.
3. `make check`.
4. `cargo fmt --check`.
5. `cargo clippy --workspace --all-targets -- -D warnings`.
6. `cargo nextest run --workspace`.
7. `npm --prefix web run lint`.
8. `npm --prefix web test`.
9. `npm --prefix web run build`.
10. `npx --prefix web playwright test`.
11. `make openapi` and `make asyncapi` with no drift.
12. `schemathesis run .docs/product/api/openapi.json`.
13. `gitleaks detect --source . --no-git`.
14. `cargo deny check`.
15. `npm --prefix web audit --audit-level=high`.
16. Trivy on changed images, plus SBOM and signature at release.
17. Coverage thresholds: backend line at least 80 percent, domain at least 90 percent, frontend at least 80 percent, new code at least 80 percent.
18. No P0 or P1 open; reviewer is not the author; merger is not the author.

Use the `quality-gates` skill for the exact commands and evidence rules.

---

## Language and tooling

- **English only.** Every repository artifact MUST be written in English: code, comments, commit messages, documentation, skills, commands, ADRs, tests, logs, UI copy and generated reports. No other language is permitted in files under version control.
- **Reusable tools.** Every time a task needs a reusable script, checker, generator, formatter, migration helper or analysis utility, you MUST:
  1. create it under `.tools/` with a stable kebab-case filename;
  2. make it executable and give it a `--help` output or a header usage comment;
  3. register it in the Tool registry below with name, path, purpose and invocation;
  4. reuse the registered tool instead of reimplementing the same logic;
  5. update the registry in the same change when the tool changes or is removed.
- **Existing tools.** `scripts/harness-lint.sh` predates this rule and remains at its current path; it is registered below and MUST be reused. `scripts/sow-lint.sh` is a WP-00/STACK contract deliverable and stays at `scripts/`; its testers and fixtures live under `.tools/` and are registered below.
- **No inline one-offs.** Do not leave a reusable check only inside a prompt, a skill or shell history. Promote it to `.tools/` and register it.

### Tool registry

| Tool | Path | Purpose | Invocation |
|---|---|---|---|
| `harness-lint` | [`scripts/harness-lint.sh`](./scripts/harness-lint.sh) | Validate harness files, indexes, ADR registry, commands, placeholder absence and relative links. | `scripts/harness-lint.sh` |
| `sow-lint` | [`scripts/sow-lint.sh`](./scripts/sow-lint.sh) | Validate WP statuses, index agreement, template structure, dependencies, progress logs, SOW links, CRs and required CI checks. | `scripts/sow-lint.sh [--root DIR]` |
| `sow-lint-self-test` | [`.tools/sow-lint-self-test.sh`](./.tools/sow-lint-self-test.sh) | Exercise `sow-lint` against disposable valid and invalid SOW trees. | `.tools/sow-lint-self-test.sh` |
| `test-analytics-views` | [`.tools/test-analytics-views.sh`](./.tools/test-analytics-views.sh) | Apply the schema and analytics views on a disposable PostgreSQL + pgvector container and assert 25 fixture values. | `.tools/test-analytics-views.sh` |
| `check-doc-links` | [`.tools/check-doc-links.sh`](./.tools/check-doc-links.sh) | Resolve relative Markdown links under `.docs/product/` and `.docs/`. | `.tools/check-doc-links.sh [--root DIR]` |
| `check-openapi-drift` | [`.tools/check-openapi-drift.py`](./.tools/check-openapi-drift.py) | Fail when an OpenAPI path is not served by the edge router. | `.tools/check-openapi-drift.py [--root DIR]` |
| `check-observability` | [`.tools/check-observability.sh`](./.tools/check-observability.sh) | Validate the SLO, alert and Grafana dashboard artifacts. | `.tools/check-observability.sh` |
| `run-e2e` | [`.tools/run-e2e.sh`](./.tools/run-e2e.sh) | Run Playwright in the official image (browser deps included). | `.tools/run-e2e.sh [args]` |
| `bdd-preflight` | [`scripts/bdd-preflight.mjs`](./scripts/bdd-preflight.mjs) | Parse `.docs/e2e/req.md` and block the BDD suite when a manual item is missing. | `node scripts/bdd-preflight.mjs [--json] [--file PATH]` |
| `bdd-preflight-test` | [`scripts/bdd-preflight.test.mjs`](./scripts/bdd-preflight.test.mjs) | Unit tests for the manual preflight parser. | `node scripts/bdd-preflight.test.mjs` |
| `bdd-set-api-key` | [`scripts/bdd-set-api-key.sh`](./scripts/bdd-set-api-key.sh) | Load the live DeepSeek key through a hidden prompt and create or rotate the provider. | `make bdd-set-api-key` |
| `bdd-set-api-key-test` | [`scripts/bdd-set-api-key.test.sh`](./scripts/bdd-set-api-key.test.sh) | Fake-provider tests for the hidden-prompt key loader. | `bash scripts/bdd-set-api-key.test.sh` |
| `gen-bdd-features` | [`.tools/gen-bdd-features.mjs`](./.tools/gen-bdd-features.mjs) | Generate deterministic `.feature` files from `.docs/e2e/BDD.md`. | `.tools/gen-bdd-features.mjs --check\|--write` |
| `gen-bdd-steps` | [`.tools/gen-bdd-steps.mjs`](./.tools/gen-bdd-steps.mjs) | Generate exact-match step definitions for narrative scenarios that lack a curated step. | `.tools/gen-bdd-steps.mjs --check\|--write` |
| `check-bdd-coverage` | [`.tools/check-bdd-coverage.mjs`](./.tools/check-bdd-coverage.mjs) | Enforce the 480-id coverage, drift and zero-pending-step gate. | `.tools/check-bdd-coverage.mjs [--report-only]` |
| `bdd-self-test` | [`.tools/bdd-self-test.sh`](./.tools/bdd-self-test.sh) | Fixture-based negative tests for the generator and coverage checker. | `.tools/bdd-self-test.sh` |
| `run-bdd` | [`scripts/run-bdd.sh`](./scripts/run-bdd.sh) | Guarded make-only BDD runner: preflight, drift, coverage, readiness, bddgen, Playwright, full log. | `make bdd` |
| `run-load` | [`.tools/run-load.sh`](./.tools/run-load.sh) | Run the k6 edge load scenario against the API namespace. | `.tools/run-load.sh [args]` |
| `chaos-worker-kill` | [`.tools/chaos-worker-kill.sh`](./.tools/chaos-worker-kill.sh) | Kill/restart the worker and verify recovery. | `.tools/chaos-worker-kill.sh` |
| `check-coverage` | [`.tools/check-coverage.sh`](./.tools/check-coverage.sh) | Enforce backend 80, domain 90 and web 80 line coverage. | `.tools/check-coverage.sh` |
| `check-architecture-fitness` | [`.tools/check-architecture-fitness.sh`](./.tools/check-architecture-fitness.sh) | Validate context ownership, dependency acyclicity, cross-context table access and Compose catalog coverage. | `.tools/check-architecture-fitness.sh [--root DIR]` |
| `architecture-fitness-self-test` | [`.tools/architecture-fitness-self-test.sh`](./.tools/architecture-fitness-self-test.sh) | Exercise the architecture fitness check against disposable roots. | `.tools/architecture-fitness-self-test.sh` |
| `check-crate-dependencies` | [`.tools/check-crate-dependencies.sh`](./.tools/check-crate-dependencies.sh) | Validate crate direction, acyclicity, `#![forbid(unsafe_code)]` and the no-unwrap gate. | `.tools/check-crate-dependencies.sh` |
| `doctor` | [`scripts/doctor.sh`](./scripts/doctor.sh) | Validate Docker, ports, disk, tools, keys and connectivity. | `scripts/doctor.sh [--generate-key] [--connectivity]` |
| `run-web` | [`scripts/run-web.sh`](./scripts/run-web.sh) | Run a command in `web/` with host npm or the pinned Node container. | `scripts/run-web.sh npm run lint` |
| `wait-for` | [`scripts/wait-for.sh`](./scripts/wait-for.sh) | Wait for a TCP host and port. | `scripts/wait-for.sh HOST PORT [TIMEOUT]` |
| `install-hooks` | [`scripts/hooks/install.sh`](./scripts/hooks/install.sh) | Install lefthook or native git hooks. | `scripts/hooks/install.sh` |
| `gen-openapi` | [`scripts/gen-openapi.sh`](./scripts/gen-openapi.sh) | Generate `.docs/product/api/openapi.json` (WP-16). | `scripts/gen-openapi.sh` |
| `gen-asyncapi` | [`scripts/gen-asyncapi.sh`](./scripts/gen-asyncapi.sh) | Generate `.docs/product/api/asyncapi.json` from the event catalog (WP-16). | `scripts/gen-asyncapi.sh` |
| `gen-asyncapi-catalog` | [`.tools/gen-asyncapi.py`](./.tools/gen-asyncapi.py) | Generate AsyncAPI channels from `EVENT_CATALOG`. | `.tools/gen-asyncapi.py [--root DIR]` |
| `seed-demo` | [`scripts/seed-demo.sh`](./scripts/seed-demo.sh) | Seed the deterministic demo dataset (WP-04). | `scripts/seed-demo.sh` |
| `test-supply-chain` | [`.tools/test-supply-chain.sh`](./.tools/test-supply-chain.sh) | Negative fixtures: vulnerable npm dependency and disallowed Cargo license must be rejected. | `.tools/test-supply-chain.sh` |
| `test-migrations` | [`.tools/test-migrations.sh`](./.tools/test-migrations.sh) | Diff the versioned migrations against Appendix A on a disposable PostgreSQL container. | `.tools/test-migrations.sh` |
| `test-constraints` | [`.tools/test-constraints.sh`](./.tools/test-constraints.sh) | Run CHECK/UNIQUE negatives and the optimistic-concurrency case. | `.tools/test-constraints.sh` |
| `test-compose-isolation` | [`.tools/test-compose-isolation.sh`](./.tools/test-compose-isolation.sh) | Assert Compose network isolation, hardening and resource limits across services. | `.tools/test-compose-isolation.sh` |
| `integrity-check` | [`scripts/integrity-check.sh`](./scripts/integrity-check.sh) | Run the report-only integrity checks against the live database. | `scripts/integrity-check.sh` |
| `backup` | [`scripts/backup.sh`](./scripts/backup.sh) | Dump the database, archive artifacts and write a manifest. | `scripts/backup.sh` |
| `restore-drill` | [`scripts/restore-drill.sh`](./scripts/restore-drill.sh) | Restore a dump into a scratch database and compare row counts. | `scripts/restore-drill.sh <dump>` |

New tools are added to this table before their first use. A tool that is not registered MUST NOT be used by an agent.

---

## Status model

Only three statuses exist for work packages and cards:

- `ready` — scoped, estimated, dependencies defined, not started.
- `progress` — active, owner assigned, at least one deliverable in progress.
- `done` — all deliverables checked, acceptance evidenced, gates green, signed off.

`blocked` is a flag, not a status. A blocked item stays in its current status and records category (`dependency`, `clarification`, `external`, `risk`, `budget`, `policy`), owner, impact, next action and ETA. Status changes are append-only in the work package history. `done` requires evidence; a status change without evidence is a defect.

---

## Update rules

1. After creating, renaming or removing any document under `.docs`, update the Documentation index in this file in the same change.
2. After creating, renaming or removing a skill, update the Skills index in this file in the same change.
3. After creating or superseding an ADR, update `.docs/adr/registry.md` and both ADR files in the same change.
4. After changing a command, update the Commands section in this file if its input or behavior changed.
5. After changing a quality gate, update the Quality gates section and `.docs/sow/WP-00-GOVERNANCE.md` in the same change.
6. Run `scripts/harness-lint.sh` after every update; fix all failures before reporting completion.
7. Never duplicate content from `README.md` or the technical specification in this file. Link instead.
8. Never add product narrative, marketing copy, screenshots or tutorials to this file.
9. After creating, changing or removing a tool, update the Tool registry in this file and `.docs/STACK.md` if its invocation changed.
10. Keep every generated or updated artifact in English.
11. After any entity, attribute, relationship, logical index, retention rule or migration change, update [`.docs/DATA-MODEL.md`](./.docs/DATA-MODEL.md) in the same change.
