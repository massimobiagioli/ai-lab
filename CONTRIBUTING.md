# Contributing to AI-Lab

## 1. Prerequisites

- Docker Engine ≥ 27 and Docker Compose ≥ 2.30.
- Rust 1.99.1 (declared in `rust-toolchain.toml`; `rustup` installs it automatically).
- Node.js 24 LTS and npm 11 (declared in `.tool-versions`). If Node is not installed on the host, `scripts/run-web.sh` falls back to the pinned `node:24.22.0-trixie-slim` container.
- Python 3.12 for tooling.

## 2. Setup

```bash
git clone <repository> ai-lab
cd ai-lab
make setup          # cargo fetch + npm ci
cp .env.example .env
scripts/doctor.sh --generate-key
make doctor
make check
```

`make check` is the fast local gate: formatting, linting, unit tests, SOW lint, documentation links and architecture fitness. It must be green before a pull request.

## 3. Branches and commits

- Branch from `main`: `ailab/{CARD-KEY}-{slug}`.
- Conventional Commits: `type(scope): imperative summary` (≤ 72 characters).
- Types: `feat`, `fix`, `chore`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`.
- Agent commits carry the trace trailers:

```
AI-Lab-Run: <run_uuid or "n/a">
AI-Lab-Card: <CARD-KEY>
AI-Lab-Agent: <agent_uuid>
```

- No direct pushes to `main`; a pull request is mandatory.
- Reviewer ≠ author and merger ≠ author (ADR-0011).

## 4. Hooks

```bash
make hooks
```

Installs lefthook when available, otherwise native git hooks. Hooks are a convenience; CI re-runs every check and is the real gate.

## 5. Pull request

Use `.github/pull_request_template.md`. Every PR links a card and a WP, lists evidence for each acceptance criterion, and records the commands run. Required status contexts are in `.github/required-checks.txt`.

## 6. Quality gates

See `docs/quality-gates.md`. A PR may merge only when every applicable gate is green. Coverage thresholds: backend line ≥ 80%, domain ≥ 90%, frontend ≥ 80%, new code ≥ 80%.

## 7. Security

Never commit a secret. Use `.env` (gitignored) for local values. `gitleaks` runs in the pre-commit hook and in CI. Report a vulnerability through a private security advisory; see `SECURITY.md`.

## 8. Documentation

Documentation is part of the Definition of Done. Product docs live in `docs/`; the normative specification, SOW and ADRs live in `.docs/`. Update the relevant index in the same change.

## 9. Toolchain notes

- TypeScript is pinned to 6.0.x because Angular 22.2 supports `>=6.0 <6.1` (ADR-0024).
- Rust is pinned to 1.99.1. Do not use a newer or older toolchain for a release build.
- Docker images and dependencies are digest- or version-pinned; Renovate owns updates.
