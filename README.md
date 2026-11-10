# AI-Lab

AI-Lab turns a raw idea into a maintained software product through a managed team of AI agents, and makes running that team feel like a 16-bit office sim. The product owns the full lifecycle: idea, statement of work, plan, Kanban tasks, execution, review, merge, release, maintenance and learning.

> **Repository status.** Governance (WP-00), architecture (WP-01) and the repository/toolchain foundation (WP-02) are delivered. The Rust workspace, Angular 22 workspace, Make interface, hooks, CI workflow and developer scripts are in place and verified. Runtime services (`make up`) land with WP-03/WP-04.

## Documentation

| Document | Path |
|---|---|
| Agent operating manual | [`AGENTS.md`](./AGENTS.md) |
| Claude agent guardrails | [`CLAUDE.md`](./CLAUDE.md) |
| Constitution | [`.docs/CONSTITUTION.md`](./.docs/CONSTITUTION.md) |
| Development principles | [`.docs/PRINCIPLES.md`](./.docs/PRINCIPLES.md) |
| Stack | [`.docs/STACK.md`](./.docs/STACK.md) |
| Technical specification | [`.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md`](./.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md) |
| SOW index | [`.docs/sow/SOW.md`](./.docs/sow/SOW.md) |
| ADR registry | [`.docs/adr/registry.md`](./.docs/adr/registry.md) |
| Harness generation prompt | [`.prompts/harness-prompt.md`](./.prompts/harness-prompt.md) |
| User playbook | [`.docs/playbooks/final-user-playbook.pdf`](./.docs/playbooks/final-user-playbook.pdf) |
| Operations playbook | [`.docs/playbooks/operation-playbook.pdf`](./.docs/playbooks/operation-playbook.pdf) |
| BDD checklist | [`.docs/e2e/BDD.md`](./.docs/e2e/BDD.md) |

## Repository map

```text
Cargo.toml         Rust workspace (12 crates, edition 2024, lints, profiles)
crates/            domain, application, infrastructure, llm, scm, memory,
                   orchestrator, api, worker, telemetry, testkit, cli
web/               Angular 22.2 workspace (zoneless, signals, strict TS 6.0)
migrations/        schema and analytics views
infra/             Docker, Compose, proxies and observability (WP-03/WP-20)
scripts/           doctor, wait-for, sow-lint, hooks, generators
.tools/            reusable checkers and fixtures (registered in AGENTS.md)
.docs/product/              product documentation (architecture, governance, templates)
.docs/             specification, SOW, work packages, ADRs, constitution, stack
.opencode/         skills and commands used by implementing agents
.github/           CI workflows, required checks and PR template
```

## Quickstart

```bash
git clone <repository> ai-lab
cd ai-lab
make setup                 # fetch dependencies in Docker
make generate-key          # create .env with the master key and runner token
make doctor
make check                 # fast local gate in Docker
```

`make check` runs formatting, clippy, ESLint/Prettier, Rust and frontend unit tests, `sow-lint`, documentation links and architecture fitness. **Every application command runs in Docker** (ADR-0027): the host needs Docker, Docker Compose, Make and Git only. Rust, Node, Python and OpenSSL are not installed or used on the host.

The runtime stack starts when WP-03 lands:

```bash
make up            # core profile
make down          # stop, preserving volumes
make logs          # follow logs
make sow-lint      # validate SOW statuses, links and dependencies
make docs-check    # harness and documentation link checks
make arch-fitness  # context ownership and dependency checks
make help          # list every target
```

## Getting started with delivery

1. Read [`AGENTS.md`](./AGENTS.md) for the operating rules.
2. Read [`.docs/sow/SOW.md`](./.docs/sow/SOW.md) for the delivery index.
3. Continue with the next work package using `/impl-wp WP-03` or a task with `/impl-task WP-03/T01`.
4. Run `make check` before every pull request.

## Principles

- Security first: no secret exposure, least privilege, sandboxed execution.
- Test-driven: Red, Green, Refactor.
- Evidence over claims: every done item links to a test, a benchmark or a merged change.
- Single source of truth: the specification wins, the SOW decomposes it, the harness executes it.
- Accessible and truthful user experience, including the pixel office.

## License

Apache-2.0. See [`LICENSE`](./LICENSE).
