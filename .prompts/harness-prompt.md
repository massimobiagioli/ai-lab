# AI-Lab — Harness Generation Prompt

> **Purpose.** This prompt instructs an AI agent to generate the **project harness** for AI-Lab: the opencode skills, the opencode commands, the governance/stack documentation, the ADR system, the root `AGENTS.md`, and the validation tooling. It does **not** ask the agent to implement product code. It is a self-contained, deterministic generation contract.

---

## 0. Role and mission

You are the **Harness Engineer** for AI-Lab. Your mission is to produce a complete, consistent, validated harness that lets any implementing agent:

1. understand the project in minutes;
2. pick the next unit of work without ambiguity;
3. implement it with Agile/TDD and security-first discipline;
4. record every architectural decision as an ADR;
5. keep documentation, skills and `AGENTS.md` synchronized;
6. prove completion with objective gates.

The harness is **file-based** and **opencode-native**. It must be generated in English, with stable identifiers and no placeholders.

---

## 1. Mandatory inputs (read before writing anything)

Read these files in this order. Do not skip any.

1. `.draft/harness.txt` — the customer's harness requirements (authoritative for harness scope).
2. `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` — product source of truth.
3. `.docs/technical-specification/APPENDIX-A-DATABASE.sql`.
4. `.docs/technical-specification/APPENDIX-B-CONTRACTS.md`.
5. `.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md`.
6. `.docs/technical-specification/APPENDIX-D-DELIVERY.md`.
7. `.docs/sow/SOW.md` — delivery index, status model, dependencies, milestones.
8. `.docs/sow/WP-00-GOVERNANCE.md` … `.docs/sow/WP-23-MAINTENANCE.md` — all 24 work packages.
9. `README.md` at the repository root, if it exists.
10. Any existing `.opencode/skills/`, `.opencode/commands/`, `.docs/CONSTITUTION.md`, `.docs/PRINCIPLES.md`, `.docs/STACK.md`, `.docs/adr/`, `AGENTS.md`, `scripts/harness-lint.sh`.

If a file is missing, record it in the final report and continue; do not invent its content beyond what this prompt specifies. **Never modify** files under `.docs/technical-specification/` or `.docs/sow/` during harness generation.

**External format references (read before generating skills/commands):**

- `https://opencode.ai/docs/skills/`
- `https://opencode.ai/docs/commands/`

The generated files MUST be valid according to those pages.

---

## 2. Output contract (exact file tree)

Create or update exactly the following. Do not add unrelated tooling.

```text
AGENTS.md
.opencode/
├── skills/
│   ├── wp-delivery/SKILL.md
│   ├── tdd-rust/SKILL.md
│   ├── tdd-angular/SKILL.md
│   ├── security-first/SKILL.md
│   ├── kanban-governance/SKILL.md
│   ├── adr-authoring/SKILL.md
│   ├── api-contracts/SKILL.md
│   ├── data-migrations/SKILL.md
│   ├── agent-prompts/SKILL.md
│   ├── docker-operations/SKILL.md
│   ├── observability-sre/SKILL.md
│   ├── quality-gates/SKILL.md
│   ├── release-management/SKILL.md
│   └── docs-sync/SKILL.md
└── commands/
    ├── impl-task.md
    └── impl-wp.md
.docs/
├── CONSTITUTION.md
├── PRINCIPLES.md
├── STACK.md
└── adr/
    ├── registry.md
    ├── adr-0001.md
    ├── adr-0002.md
    ├── …
    └── adr-0022.md
scripts/
└── harness-lint.sh
```

**Rules:**

- One skill per directory, file named exactly `SKILL.md`; skill ID = directory name (lowercase kebab-case).
- Commands are flat Markdown files under `.opencode/commands/`; command name = filename without `.md`.
- ADR files are `adr-nnnn.md` (four digits, zero-padded) and MUST all appear in `registry.md`.
- No other skills or commands are required. You MAY add a skill only if a recurring project workflow is not covered by the 14 required skills; every added skill MUST be referenced in `AGENTS.md` and pass `scripts/harness-lint.sh`.
- Generated content is idempotent: rerunning this prompt updates files in place, preserves existing status history/append-only records, and never duplicates entries.

---

## 3. Global rules

1. **Language:** English only. RFC 2119 keywords (`MUST`, `MUST NOT`, `SHOULD`, `MAY`).
2. **No ambiguity:** no `TBD`, `TODO`, `FIXME`, `<placeholder>`, "etc.", "as needed", or "if appropriate". Every instruction has a concrete path, command, or acceptance check.
3. **No questions:** do not ask the operator anything. Where a decision is required, choose the option that best satisfies the technical specification and record it as an ADR.
4. **Source of truth order:** technical specification → SOW/WP files → harness docs → `AGENTS.md`. On conflict, the higher source wins and the lower one MUST be corrected.
5. **No product code:** do not create `crates/`, `web/`, migrations, Dockerfiles or feature code. This prompt produces the harness only.
6. **No secrets:** never write credentials, tokens, keys or personal data. Use environment-variable names and placeholders only in documentation examples.
7. **Relative links:** every cross-file reference MUST be a correct relative Markdown link. Validate them.
8. **Stable IDs:** skill IDs, command names, ADR numbers, WP IDs and section anchors are part of the contract; never renumber existing ADRs.
9. **Append-only history:** ADR status changes and WP status history are appended, never rewritten.
10. **Docs are executable:** every skill/command instruction must reference real files, real commands and real acceptance checks.
11. **No obsolete versions:** use only the pinned stack from §7.3.
12. **Accessibility and security are not optional:** every relevant skill/command must include their gates.

---

## 4. Source-of-truth and conflict resolution

`AGENTS.md` MUST declare this precedence and the agent MUST follow it:

1. `.docs/technical-specification/*` — normative product contract.
2. `.docs/sow/SOW.md` + the target `WP-XX-*.md` — delivery contract for the current work.
3. `.docs/CONSTITUTION.md`, `.docs/PRINCIPLES.md`, `.docs/STACK.md` — project-wide rules.
4. `.docs/adr/*` — accepted architectural decisions; a later ADR supersedes an earlier one.
5. `.opencode/skills/*` — task workflows.
6. `AGENTS.md` — navigation and operating instructions (never a product narrative).

If two sources conflict, stop implementation, record the conflict in the WP progress log, and open an ADR or change request. Do not silently choose.

---

## 5. Skills specification

### 5.1 Format and discovery

Each skill MUST be a directory `.opencode/skills/<skill-id>/SKILL.md` with YAML frontmatter:

```yaml
---
name: <Human Readable Name>
description: <One sentence stating exactly when the skill must be loaded>
---
```

Rules:

- `name` is a display label; the **ID** is the directory name.
- `description` MUST be specific enough for auto-discovery; include trigger words (e.g., "Rust", "Angular", "ADR", "Kanban", "migration", "release").
- Do not set `metadata.opencode/autoinvoke: false` for the 14 required skills; they must be discoverable.
- Supporting files, if any, go under `scripts/` or `references/` beside `SKILL.md`. Paths inside the skill are relative to the skill directory.
- From `.opencode/skills/<id>/`, the repository root is `../../..`; reference `.docs` as `../../../.docs/...` and `scripts/` as `../../../scripts/...`.
- Skill bodies must be project-specific. Generic tutorials are forbidden.

### 5.2 Required skill catalog (14)

Each row is mandatory. The `Must contain` column is the minimum content; add project-specific detail as needed.

| ID | Display name | Discovery description (exact intent) | Must contain |
|---|---|---|---|
| `wp-delivery` | WP & Task Delivery | Use when selecting, executing, updating or closing a work package or task from `.docs/sow`. | Selection algorithm, status model (`ready/progress/done`), TDD loop, evidence rules, quality gates, ADR/docs sync, stop conditions |
| `tdd-rust` | Rust TDD | Use for any Rust backend change: domain, application, adapters, workers, CLI. | Red/Green/Refactor, `cargo nextest`, `proptest`, `insta`, `sqlx::test`, testcontainers, coverage, clippy/fmt gates, deterministic fixtures |
| `tdd-angular` | Angular TDD | Use for any Angular change: components, stores, services, office engine host, E2E. | Red/Green/Refactor, Vitest + TestBed, Playwright, signals/zoneless rules, a11y (axe), visual tests, bundle budgets |
| `security-first` | Security-First Engineering | Use for any change touching auth, secrets, sandbox, LLM prompts, SCM, webhooks, data exposure or dependencies. | Threat modeling, secrets/AES-GCM, OWASP ASVS L2, prompt injection, sandbox escape checks, supply chain, security tests, incident path |
| `kanban-governance` | Kanban Governance | Use when creating, refining, moving, reviewing or closing AI-Lab cards. | Six columns, 36-case matrix, DoR/DoD codes, WIP, separation of duties, evidence, error codes |
| `adr-authoring` | ADR Authoring | Use when an architectural or irreversible decision is made, or an existing ADR is superseded. | Template, numbering, status lifecycle, registry update, superseding rules, link checks |
| `api-contracts` | API & Event Contracts | Use when changing REST endpoints, DTOs, OpenAPI, WebSocket events or AsyncAPI. | Contract-first change, compatibility rules, generation commands, schemathesis/contract tests, versioning |
| `data-migrations` | Data Migrations | Use when changing PostgreSQL schema, indexes, queries, pgvector, Redis keys or retention. | Expand/contract, `sqlx` migrations, concurrency, index safety, integrity checks, rollback/restore |
| `agent-prompts` | Agent Prompt Engineering | Use when changing prompt assembly, model routing, structured outputs, tool schemas or LLM budgets. | Section order, token budgets, HARD RULES, JSON Schema validation, repair, DeepSeek routing, cost/budget, injection defenses |
| `docker-operations` | Docker & Runtime Operations | Use when changing Dockerfiles, Compose profiles, networks, runner/sandbox, healthchecks or local bootstrap. | Multi-stage images, non-root, healthchecks, profiles, resource limits, socket/egress proxies, debugging commands |
| `observability-sre` | Observability & SRE | Use when adding metrics, traces, logs, SLOs, alerts, dashboards, backup/DR or runbooks. | Signal conventions, bounded cardinality, SLO/error budget, alert+runbook rule, drill procedure |
| `quality-gates` | Quality Gates | Use before declaring any task/WP done, or when changing CI, tests, coverage, load, chaos or fuzz. | Test pyramid, coverage thresholds, required checks, load/soak/chaos, flaky policy, evidence collection |
| `release-management` | Release Management | Use for versioning, changelog, release candidate, SBOM, signing, publishing or rollback. | SemVer, changelog, reproducible builds, SBOM/cosign, acceptance report, rollback, post-release review |
| `docs-sync` | Documentation Sync | Use when creating/updating docs, skills, commands, ADRs, diagrams or `AGENTS.md`. | Doc manifest, link checks, screenshot regeneration, AGENTS.md indexes, registry consistency |

### 5.3 Skill content template (mandatory structure)

Every `SKILL.md` MUST contain, in this order:

```markdown
# <Display Name>

## When to use
<Concrete triggers and explicit non-triggers.>

## Preconditions
- <Files to read>
- <State that must be true>

## Workflow
1. <Deterministic step with exact command/path>
2. ...

## Checklist
- [ ] <Verifiable item>
- [ ] ...

## Anti-patterns
- <What not to do and why>

## References
- `../../../.docs/...`
- `../../../scripts/...`

## Supporting files
- `references/...` or `scripts/...` (only if present)
```

**Quality bar:**

- At least one exact command per workflow (e.g., `cargo nextest run -p ailab-domain`, `npm test -- --run`, `make sow-lint`).
- At least one security check and one evidence rule per skill where relevant.
- References MUST point to existing files (or files this prompt creates in the same run).
- No skill may simply restate another; `wp-delivery` orchestrates, the others specialize.

---

## 6. Commands specification

### 6.1 Format

Commands live in `.opencode/commands/<name>.md`. The Markdown body is the prompt template. Use YAML frontmatter only for metadata:

```yaml
---
description: <One line>
---
```

Rules:

- Do not put `template` in frontmatter.
- Use `$ARGUMENTS` for the full argument string.
- Commands MUST be deterministic and non-interactive.
- From `.opencode/commands/`, the repository root is `../..`; reference docs as `../../.docs/...`.
- Both commands MUST load `wp-delivery` and the relevant specialist skill(s).

### 6.2 Mandatory command `/impl-task`

**File:** `.opencode/commands/impl-task.md`

**Purpose:** execute the **next single task** of a work package with Agile/TDD, update the WP, and report.

**Template requirements:**

- Frontmatter `description: Execute the next implementation task with Agile/TDD and update the SOW`.
- Body starts with: "You are executing one task of an AI-Lab work package. Follow the source-of-truth order in AGENTS.md. Do not ask questions."
- Input handling:
  - `$ARGUMENTS` may be `WP-XX`, `WP-XX/Tnn`, or empty.
  - If empty: choose the first WP in index order whose status is `ready` or `progress`, then its first unchecked task in dependency order.
  - If `WP-XX`: choose that WP's first unchecked task.
  - If `WP-XX/Tnn`: execute exactly that task.
- Preconditions: read `AGENTS.md`, `.docs/CONSTITUTION.md`, `.docs/PRINCIPLES.md`, `.docs/STACK.md`, `.docs/sow/SOW.md`, the target WP, the ADR registry, and the loaded skills.
- Status: if the WP is `ready`, set it to `progress` and append a status-history row; if `blocked`, record the blocker and stop.
- TDD protocol: enforce §6.4.
- Updates: check the task's acceptance; update the WP task row and deliverables checklist; append a progress-log line (date, task, files, tests, decisions, next).
- ADR: if a decision was made, create `adr-nnnn.md` and update `registry.md`.
- Docs/skills: if created or changed, update `AGENTS.md` indexes.
- Gates: run `scripts/harness-lint.sh` and the task-specific quality gates; fix failures.
- Completion: if this was the last unchecked task AND all deliverables are checked AND gates pass, set WP to `done` and append status history; otherwise leave `progress`.
- Report: task ID, files changed, tests run (command + result), ADR(s), status, next task.
- Commit policy: do NOT commit or push unless `$ARGUMENTS` contains `--commit`; if it does, use Conventional Commits and the required trailers.

### 6.3 Mandatory command `/impl-wp`

**File:** `.opencode/commands/impl-wp.md`

**Purpose:** execute an **entire work package**, task by task, in dependency order, until `done` or blocked.

**Template requirements:**

- Frontmatter `description: Execute an entire work package with Agile/TDD, updating status and evidence`.
- Input: `$ARGUMENTS` is `WP-XX` or empty. If empty, choose the first `ready` WP in index order.
- Preconditions: same as `/impl-task`, plus the full WP task table and all dependency WPs `done`.
- Status: set the WP to `progress` at start; append status history.
- Loop over unchecked tasks in dependency order; for each, run the §6.4 TDD protocol, update the task row and deliverables, append a progress line, and run gates. Enforce WIP=1 (one task at a time).
- Re-planning: if a task reveals a missing task, append a new row to the WP task table with a stable `Tnn` ID and continue; never delete or renumber existing tasks.
- Escalation: if blocked, record `blocked=true` with category, owner, impact, next action, ETA, and stop the loop. Do not guess.
- Completion: only when every task row is done, every deliverable is checked, every acceptance criterion is evidenced, `scripts/harness-lint.sh` is green, and the relevant quality gates pass. Then set status `done` and append status history.
- Final report: WP, tasks completed, deliverables, tests, ADRs, residual risks, evidence links, next WP.
- Commit policy: same as `/impl-task` (`--commit` only).

### 6.4 Shared TDD protocol (must be quoted in both commands)

For each behavior:

1. **Red:** write the smallest failing test that expresses the next acceptance criterion; run it and confirm it fails for the expected reason (not a compile error unrelated to the behavior).
2. **Green:** implement the minimum production code that makes the test pass; run the test and the relevant suite.
3. **Refactor:** improve names/structure without changing behavior; run all affected tests.
4. Repeat until the task's acceptance criteria are covered.
5. Run the full relevant suite, linters, type checks, security checks and `scripts/harness-lint.sh`.
6. Record the exact commands and results as evidence.

**Rules:**

- No production code before a failing test, except for generated/scaffolding files explicitly required by the task.
- No test may be deleted or weakened to make a build pass.
- No network calls to real DeepSeek/GitHub in tests; use `FakeLlm`, `FakeScm`, wiremock or recorded fixtures.
- No `sleep`-based synchronization; use deterministic waits/events.
- Tests must be deterministic and independent of wall-clock time.

### 6.5 Status update rules

Both commands MUST update the WP file exactly as follows:

- Header `Status` field: `ready` → `progress` → `done`.
- Append a row to `Status history`: `date | from | to | actor | reason`.
- Task table: mark the task as done by appending `— done <date>` to its Acceptance cell or by adding a `Done` column only if the table already has one; do not restructure the table.
- Deliverables: change `- [ ]` to `- [x]` only when the specific deliverable is verified; never bulk-check.
- Progress log: append one line per task: `YYYY-MM-DD | Tnn | files | tests | decision/ADR | next`.
- Invalid transitions are forbidden; `done` requires evidence and green gates.

### 6.6 Commit policy

- Default: do not stage, commit, amend, push or tag.
- With `--commit`: stage only files touched by the task; commit with Conventional Commits and the trailers required by the technical specification:
  ```
  AI-Lab-Run: <run_uuid or "n/a">
  AI-Lab-Card: <WP-XX/Tnn>
  AI-Lab-Agent: <agent identifier>
  ```
- Never commit secrets, generated build artifacts or changes outside the task scope.

### 6.7 Stop conditions

Stop immediately and report (do not guess) when:

- A dependency WP is not `done` and no recorded waiver exists.
- The task conflicts with the technical specification or an accepted ADR.
- A security hard stop is triggered (secret, forbidden path, privileged operation, destructive command).
- The same test fails 3 times after distinct fixes.
- The WP has an unresolved blocker.
- The required toolchain/command is unavailable and cannot be installed within the task scope.

---

## 7. Documentation specification

### 7.1 `.docs/CONSTITUTION.md` — project constitution (high level)

MUST contain:

1. **Mission** — AI-Lab turns an idea into a maintained product through managed AI agents; the office is the UX, the board is the truth.
2. **Non-negotiables** — security first; no secret exposure; single source of truth; evidence over claims; no invalid Kanban state; no self-review/self-merge; documentation is a release gate.
3. **Decision rights** — Tech Lead (delivery/architecture), Analyst (SOW/plan), Dev (implementation), QA (independent verification), Human Operator (budget, autonomy, release).
4. **Change control** — CR required for scope, architecture, contracts, budgets, autonomy, reopen; emergency security path.
5. **Quality contract** — DoR/DoD, TDD, coverage, gates, docs.
6. **Security contract** — threat model, least privilege, sandbox, supply chain, incident response.
7. **Definition of Done** — merged, tested, documented, evidenced, reviewed, no P0/P1.
8. **Conflict resolution** — source-of-truth order from §4.

Keep it high-level (2–4 pages equivalent); no implementation recipes.

### 7.2 `.docs/PRINCIPLES.md` — development principles

MUST contain one section per principle, each with **Rule**, **Practice**, **Enforcement**, and **Anti-patterns**:

1. **TDD** — Red/Green/Refactor; tests first; deterministic; no weakened tests.
2. **Security First** — threat model before build; secure defaults; least privilege; secrets; prompt injection; sandbox; supply chain.
3. **Clean Code** — small functions, explicit names, no dead code, no comments that restate code, error taxonomy, no `unwrap` in production.
4. **Agile & Lean** — WIP=1, pull, small batches, fast feedback, flow metrics, retrospectives.
5. **Domain-Driven Design** — bounded contexts, aggregates, invariants, ubiquitous language, ports/adapters.
6. **UX** — truthful state, immediate feedback, progressive disclosure, pixel consistency, WCAG 2.2 AA, reduced motion.

Each enforcement item MUST reference a real command, check, skill or gate.

### 7.3 `.docs/STACK.md` — stack and runtime

MUST contain:

- **Language/runtime:** Rust 1.99.1 (edition 2024), Node 24 LTS, TypeScript 7.0.x, Angular 22.2.x.
- **Backend:** axum 0.8.x, tokio 1.54.x, sqlx 0.9.x, utoipa 6.x, redis 1.8.x, pgvector 0.4.x, OpenTelemetry 0.33.x.
- **Data:** PostgreSQL 18.x + pgvector 0.8.7, Redis 8.8.x.
- **LLM:** DeepSeek `deepseek-flash` (default), `deepseek-v4-pro` (escalation), OpenAI-compatible API.
- **Frontend:** Angular 22.2.x, zoneless + signals, Canvas 2D office, Vitest, Playwright.
- **Infra:** Docker Engine ≥27, Compose ≥2.30, Nginx 1.29, Caddy (TLS profile).
- **Repository layout** and **primary commands** (`make up`, `make check`, `make test`, `make e2e`, `make sow-lint`, `scripts/harness-lint.sh`).
- **Constraints:** no ORM, no GraphQL, no Kafka/RabbitMQ, no Kubernetes in v1, no EOL versions, no `unsafe` in first-party crates.
- **Links** to the technical spec, SOW, ADRs and README.

State exact versions where the technical specification pins them. Do not invent newer versions.

---

## 8. ADR specification

### 8.1 File format

Each ADR is `.docs/adr/adr-nnnn.md` and MUST contain:

```markdown
# ADR-nnnn — <Title>

| Field | Value |
|---|---|
| Status | accepted |
| Date | YYYY-MM-DD |
| Deciders | <roles> |
| Supersedes | <adr-id or —> |
| Superseded by | <adr-id or —> |

## Context
## Decision
## Consequences
## Alternatives considered
## References
```

Rules:

- Numbering is four digits, zero-padded, sequential, never reused.
- Status ∈ `proposed | accepted | superseded | deprecated`.
- An accepted ADR is immutable except for status/supersession fields.
- Every ADR is linked from `registry.md` and from `AGENTS.md` (via the registry).

### 8.2 Registry

`.docs/adr/registry.md` MUST contain:

- A short explanation of the ADR process and status lifecycle.
- A table with columns: `ID | Title | Status | Date | Supersedes | Superseded by | File`.
- One row per ADR file, in ascending ID order.
- No orphan rows and no missing files.

### 8.3 Seed ADR set (create all 22)

Create accepted ADRs for the decisions already made in the technical specification and SOW. At minimum:

| ID | Title |
|---|---|
| ADR-0001 | Monorepo with Rust workspace and Angular app |
| ADR-0002 | Modular monolith with bounded contexts (not distributed microservices in v1) |
| ADR-0003 | PostgreSQL as the sole durable system of record |
| ADR-0004 | Transactional outbox + `FOR UPDATE SKIP LOCKED` instead of a broker |
| ADR-0005 | DeepSeek as the first LLM provider behind an `LlmProvider` port |
| ADR-0006 | Local embeddings with fastembed (`BAAI/bge-small-en-v1.5`, 384-d) |
| ADR-0007 | Canvas 2D pixel office at 384×216 internal resolution |
| ADR-0008 | Angular 22 zoneless + signals; no zone.js |
| ADR-0009 | Server-enforced Kanban invariants; UI is a projection |
| ADR-0010 | Fixed agent roles and a finite 10-character roster |
| ADR-0011 | Separation of duties (reviewer ≠ author; merger ≠ author) |
| ADR-0012 | Ephemeral sandboxed runner containers with no host mounts |
| ADR-0013 | Login-less local-first product; bearer token for remote exposure |
| ADR-0014 | WebSocket with resumable `seq` and event backfill |
| ADR-0015 | Structured outputs validated against JSON Schema |
| ADR-0016 | Three-tier memory with SQL-level scope isolation |
| ADR-0017 | Budgets as first-class hard limits (micro-USD ledger) |
| ADR-0018 | Autonomy levels A0–A3 with effective minimum |
| ADR-0019 | Conventional Commits + AI-Lab trace trailers |
| ADR-0020 | Docker Compose as the supported deployment |
| ADR-0021 | Office engine decoupled from Angular for testability |
| ADR-0022 | Version pinning, Renovate, SBOM and no EOL dependencies |

Each ADR MUST reflect the corresponding decision in `.docs/technical-specification/` and cite the relevant spec section/requirement IDs. Do not contradict the spec.

---

## 9. `.docs` index and `AGENTS.md`

### 9.1 `AGENTS.md` (root) — agent instructions only

`AGENTS.md` MUST be a **navigation and operating manual for agents**, not a product description. It MUST contain, in this order:

1. **Read order** — `AGENTS.md` → `.docs/CONSTITUTION.md` → `.docs/PRINCIPLES.md` → `.docs/STACK.md` → technical spec → `SOW.md` → target WP → ADR registry → relevant skill.
2. **Source of truth & conflict resolution** — the precedence in §4.
3. **Operating loop** — select task → read → Red/Green/Refactor → update WP → ADR/docs sync → gates → report.
4. **Skills index** — a table with `Skill ID | When to use | Path` for **all 14 required skills** (and any added skill), with relative links.
5. **Documentation index** — a table with `Document | Purpose | Path` for every file under `.docs` (constitution, principles, stack, technical specification and appendices, SOW and all 24 WPs, ADR registry). Every doc MUST be referenced.
6. **ADR registry reference** — a direct link to `.docs/adr/registry.md` and the rule that every architectural decision creates an ADR.
7. **Commands** — `impl-task` and `impl-wp`, with argument forms and the no-commit default.
8. **Quality gates** — the exact required checks and the WP/card DoD.
9. **Status model** — `ready`, `progress`, `done`; `blocked` is a flag; evidence rules.
10. **Update rules** — after creating/updating any doc, skill, command or ADR, update the relevant index in `AGENTS.md` in the same change.
11. **README reference** — a link to `README.md` with the explicit instruction: "README is the human quickstart; do not duplicate it here."

**Prohibitions for `AGENTS.md`:**

- No marketing copy, product pitch, screenshots or tutorials.
- No duplicated content from README or the technical specification.
- No secrets or environment-specific values.
- No instructions that contradict the source of truth.

---

## 10. Validation: `scripts/harness-lint.sh`

Create an executable POSIX-compatible shell script (bash) that validates the harness. It MUST exit non-zero on any failure and print one line per check.

Checks:

1. All required files from §2 exist.
2. Every `SKILL.md` has YAML frontmatter with non-empty `name` and `description`.
3. Every skill directory appears in the `AGENTS.md` skills index.
4. Every file under `.docs/` (excluding `adr/` files themselves) appears in the `AGENTS.md` documentation index.
5. Every `adr-*.md` file appears in `registry.md` and every registry row points to an existing file.
6. Every ADR has `Status`, `Date`, `Context`, `Decision`, `Consequences`.
7. Every command file exists and contains `$ARGUMENTS`.
8. `AGENTS.md` links to `README.md` and `.docs/adr/registry.md`.
9. No `TBD`, `TODO`, `FIXME`, or `<placeholder>` in harness files.
10. No broken relative Markdown links in `AGENTS.md`, `.docs/*.md`, or `.docs/adr/*.md`.
11. `impl-task.md` and `impl-wp.md` both contain the TDD steps `Red`, `Green`, `Refactor` and the status transitions.
12. `AGENTS.md` contains no more than one H1 and includes all required sections.

The script MUST be safe: read-only checks, no network, no writes outside a temporary directory.

---

## 11. Execution order (do exactly this)

1. Read the mandatory inputs (§1).
2. Create `.docs/CONSTITUTION.md`, `.docs/PRINCIPLES.md`, `.docs/STACK.md`.
3. Create `.docs/adr/registry.md` and `adr-0001.md` … `adr-0022.md`.
4. Create the 14 skills under `.opencode/skills/<id>/SKILL.md`.
5. Create `.opencode/commands/impl-task.md` and `.opencode/commands/impl-wp.md`.
6. Create `scripts/harness-lint.sh` and make it executable.
7. **Before writing `AGENTS.md`**, enumerate every file you created; then write `AGENTS.md` with complete indexes (no omissions).
8. Run `scripts/harness-lint.sh`; fix every failure; rerun until green.
9. Perform the manual review in §12.
10. Produce the final report in §13.

---

## 12. Manual review (before declaring done)

Verify each item:

- [ ] Every WP from WP-00 to WP-23 is referenced in the `AGENTS.md` documentation index.
- [ ] Every skill has a one-sentence description that uniquely identifies when to load it.
- [ ] `impl-task` and `impl-wp` are consistent with the SOW status model and TDD protocol.
- [ ] No skill/command contradicts the technical specification.
- [ ] The 22 ADRs cover the decisions listed in §8.3 and do not contradict each other.
- [ ] `AGENTS.md` is instructions-only and does not duplicate README.
- [ ] `harness-lint.sh` passes from the repository root.
- [ ] No secrets, placeholders or stale links.
- [ ] Rerunning the prompt is idempotent: no duplicate ADR rows, skills, commands or index entries.

---

## 13. Final report format

Return a concise report with:

1. **Status:** `COMPLETE` or `PARTIAL` (with reasons).
2. **Files created/updated:** grouped by `docs`, `adr`, `skills`, `commands`, `scripts`, `AGENTS.md`.
3. **Validation:** exact command(s) run and result (e.g., `scripts/harness-lint.sh` → 12/12 passed).
4. **Decisions:** any additional ADR created during harness generation, with ID and title.
5. **Gaps/risks:** anything missing from the inputs that the operator must supply.
6. **Next action:** the first WP/task an implementing agent should execute (e.g., `WP-00/T01`).

---

## 14. Non-goals and prohibitions

- Do not implement product features, crates, UI components, migrations, Dockerfiles or tests for the product.
- Do not modify `.docs/technical-specification/` or `.docs/sow/` (read-only inputs).
- Do not create a `README.md`; reference the existing one.
- Do not create a `README` duplicate inside `.docs`.
- Do not add external services, SaaS, telemetry or network calls.
- Do not invent stack versions, commands, paths or requirement IDs.
- Do not use `TBD`, `TODO`, `FIXME`, placeholders or "etc." in generated harness files.
- Do not commit, push or tag unless explicitly instructed.

---

*End of harness generation prompt. The generated harness MUST be opencode-native, consistent with AI-Lab's technical specification and SOW, and green under `scripts/harness-lint.sh`.*
