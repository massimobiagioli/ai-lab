# CLAUDE.md — AI-Lab agent guardrails

This file is a compatibility entry point for Claude-based agents. The authoritative operating manual is [`AGENTS.md`](./AGENTS.md); read it first, then the constitution, principles and stack.

## Non-negotiable: Docker-only toolchain

AI-Lab is a Docker-first repository. The host provides **Docker, Docker Compose, Make, Git and basic shell utilities only**. Rust, Node/npm, Python, OpenSSL and the other application toolchains run inside the pinned images.

- Every Rust command goes through `scripts/run-rust.sh`.
- Every frontend command goes through `scripts/run-web.sh`.
- Every Node script goes through `scripts/run-node.sh`.
- Every Python script goes through `scripts/run-python.sh`.
- Key generation is `make generate-key` (Docker Node), never `scripts/doctor.sh --generate-key`, host `openssl` or a manual edit of `.env`.
- Tests, formatters, linters, generators, coverage, load and BDD commands are Make targets that use the Docker wrappers. Do not bypass them.
- Never run `cargo`, `rustc`, `npm`, `npx`, `node`, `python3` or `openssl` directly on the host, and never add a host-toolchain command to a Makefile, script, playbook, skill or prompt.
- Do not add a "host fallback" to a wrapper. A missing Docker daemon is an error, not a reason to run on the host.
- CI, local development, documentation and the playbooks follow the same rule.

## Source of truth

When instructions conflict, the order is:

1. `.docs/technical-specification/` — normative product contract.
2. `.docs/sow/SOW.md` and the target work package.
3. `.docs/CONSTITUTION.md`, `.docs/PRINCIPLES.md`, `.docs/STACK.md`.
4. Accepted ADRs (later supersedes earlier).
5. `.opencode/skills/`.
6. `AGENTS.md` and this file.

A conflict must be recorded, not silently resolved.

## Working rules

- Use the Make targets and the registered tools in `AGENTS.md`.
- Keep the repository English-only.
- Do not commit secrets, `.env`, keys or tokens.
- Run `scripts/harness-lint.sh`, `.tools/check-doc-links.sh` and the relevant gates before declaring done.
- A new reusable script belongs in `.tools/` (or `scripts/` when it is a harness contract) and must be registered in `AGENTS.md`.
- Architectural or irreversible decisions require a new ADR in `.docs/adr/` and a registry row.

## Quick start

```bash
make setup
make generate-key
make doctor
make up
make seed-demo
```

Open `http://127.0.0.1:8080`. For tests and BDD, see `make help`, `make bdd-list` and the [operations playbook](./.docs/playbooks/operation-playbook.pdf).
