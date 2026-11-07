# WP-02 — Repository, Toolchain & Developer Experience

| Field | Value |
|---|---|
| WP ID | `WP-02` |
| Status | **done** |
| Owner | Dev |
| Milestone | M0 |
| Estimate | 8 points |
| Depends on | WP-00, WP-01 |
| Blocks | WP-03..WP-22 |
| Requirement traceability | `VER-001..008`, `OPS-010..014`, `NFR-011`, `DOC-001..003` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-00 and WP-01 done; T01 started |
| 2026-11-07 | `progress` | `done` | Dev (agent) | All deliverables verified; independent review APPROVED; AC1 and AC8 downstream verification owned by WP-03 and WP-16 |
| 2026-11-07 | `done` | `done` | Tech Lead (agent) | WP review: review plan created, gates reproduced, CR-0004 adds AILAB_RUNNER_TOKEN; QA verdict recorded in the review plan |

---

## 1. Objective

Deliver a **reproducible, fast, secure developer environment** and repository standard: one monorepo, pinned toolchains, deterministic builds, a single `make` interface, automated formatting/linting, pre-commit hooks, CI workflows, dependency governance and contribution rules. A new developer (or agent) must go from clone to green `make check` in ≤ 10 minutes without tribal knowledge.

---

## 2. Scope

### 2.1 In scope

- Monorepo layout and ownership (`CODEOWNERS`).
- Rust workspace and Angular workspace configuration.
- Pinned toolchains and package managers.
- Make targets, scripts and local dev workflows.
- Formatting, linting, type checking, commit hooks and commit conventions.
- CI workflows, caching, artifacts and required checks.
- Dependency update and supply-chain policy.
- `.env` handling, secrets hygiene and local configuration.
- Contribution guide and review checklist.

### 2.2 Out of scope

- Docker image definitions (WP-03).
- Database schema content (WP-04).
- Application code (WP-06+).
- CI runners infrastructure (GitHub-hosted runners are assumed).

---

## 3. Engineering specification

### 3.1 Monorepo layout (exact)

```
ai-lab/
├── Cargo.toml                       # workspace, [workspace.package], lints
├── Cargo.lock                       # committed
├── rust-toolchain.toml              # channel = "1.99.1", edition 2024
├── .tool-versions                   # rust 1.99.1, nodejs 24.x, python 3.12
├── .editorconfig
├── .gitattributes                   # LF normalization; binary assets
├── .gitignore
├── .dockerignore
├── .env.example
├── Makefile
├── lefthook.yml
├── renovate.json
├── deny.toml
├── rustfmt.toml
├── clippy.toml
├── LICENSE                          # Apache-2.0
├── README.md
├── CHANGELOG.md
├── SECURITY.md
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
├── CODEOWNERS
├── crates/
│   ├── domain/
│   ├── application/
│   ├── infrastructure/
│   ├── llm/
│   ├── scm/
│   ├── memory/
│   ├── orchestrator/
│   ├── api/
│   ├── worker/
│   ├── telemetry/
│   ├── testkit/
│   └── cli/
├── web/
│   ├── package.json
│   ├── package-lock.json
│   ├── angular.json
│   ├── tsconfig.json
│   ├── tsconfig.app.json
│   ├── eslint.config.js
│   ├── src/
│   └── e2e/
├── migrations/
├── infra/
│   ├── docker/
│   ├── compose/
│   ├── nginx/
│   ├── prometheus/
│   ├── grafana/
│   └── otel/
├── scripts/
│   ├── doctor.sh
│   ├── sow-lint.sh
│   ├── wait-for.sh
│   ├── gen-openapi.sh
│   ├── gen-asyncapi.sh
│   ├── seed-demo.sh
│   └── hooks/
├── .docs/product/
│   ├── architecture.md
│   ├── domain-model.md
│   ├── api/
│   ├── diagrams/
│   ├── runbooks/
│   ├── templates/
│   └── ...
└── .github/
    ├── workflows/
    │   ├── ci.yml
    │   ├── images.yml
    │   ├── nightly.yml
    │   └── release.yml
    ├── pull_request_template.md
    └── ISSUE_TEMPLATE/
```

**Rules:** no source file outside the layout above; `scripts/` contains only executable tooling; generated artifacts never committed except documented contract snapshots (`.docs/product/api/*.json`) and lockfiles. ADRs live in `.docs/adr/` per CR-0001; `.docs/product/` holds product documentation.

### 3.2 Toolchain pinning

| Tool | Pin | Enforcement |
|---|---|---|
| Rust | `1.99.1` (channel + components `rustfmt`, `clippy`, `llvm-tools-preview`) | `rust-toolchain.toml`; CI fails on mismatch |
| Edition | 2024 | `Cargo.toml` |
| Node.js | `24.x` LTS (Krypton line) | `.tool-versions`, Docker image `node:24-trixie-slim`, `package.json engines` |
| npm | `11.x` (ships with Node 24; pin in `packageManager`) | `corepack` disabled; `package-lock.json` v3 |
| PostgreSQL | `18.x` | Compose image tag + digest; `psql` client version check in doctor |
| Redis | `8.8.x` | Compose tag + digest |
| Python | `3.12.x` (tooling only) | `.tool-versions` |
| Docker | `≥ 27` | `make doctor` |
| Compose | `≥ 2.30` | `make doctor` |

**No `latest` tags. No floating versions. No pre-releases.** Security patches within 48 h; majors require an ADR.

### 3.3 Rust workspace configuration

- Single workspace with `resolver = "3"` (edition 2024).
- `[workspace.package]` centralizes `version`, `edition`, `license`, `repository`, `rust-version`.
- `[workspace.lints.rust]` enables `unsafe_code = "forbid"`, `unused_must_use = "deny"`, `rust_2018_idioms = "deny"`.
- `[workspace.lints.clippy]` enables `all`, `pedantic` (with an explicit allowlist), `nursery` selective; CI runs `-D warnings`.
- Release profile: `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"`, `panic = "unwind"` (graceful errors; `panic=abort` forbidden).
- Dev profile: `opt-level = 1` for dependencies to speed test loops; `debug = 1`.
- `sccache` optional via `RUSTC_WRAPPER`; CI uses `Swatinem/rust-cache`.
- SQLx offline mode: `.sqlx/` committed; CI uses `SQLX_OFFLINE=true` after `cargo sqlx prepare --workspace`.

### 3.4 Angular workspace configuration

- Angular **22.2.x**, standalone components, zoneless (`provideZonelessChangeDetection()`), signals, new control flow (`@if/@for/@switch`).
- TypeScript **6.0.x** with `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `noImplicitOverride`, `noFallthroughCasesInSwitch` (ADR-0024; Angular 22.2 supports `>=6.0 <6.1`).
- ESLint flat config with Angular ESLint + `@typescript-eslint`; Prettier for formatting (printWidth 100, singleQuote, trailingComma all).
- Test runner: Vitest (unit), Playwright (E2E).
- Build: Angular CLI/esbuild; budgets enforced (`initial ≤ 350 KB gzip`, `office chunk ≤ 180 KB gzip`, CSS ≤ 60 KB gzip).
- No NgModules for new code; no `any` outside validated parse boundaries; no `eval`; CSP-safe.

### 3.5 Make interface (normative)

`make` is the only supported entry point. Targets:

| Target | Behavior | Max duration |
|---|---|---|
| `make help` | List targets with one-line descriptions | 1 s |
| `make up` | Build + start core profile (db, redis, api, worker, web, migrate) | 10 min cold |
| `make up-obs` | Add observability profile | 5 min |
| `make down` | Stop and remove containers (volumes preserved) | 60 s |
| `make reset` | Stop, remove volumes, re-up, migrate, seed demo | 10 min |
| `make logs` | Follow logs for core services | — |
| `make ps` | Service status + health | 2 s |
| `make migrate` | Run sqlx migrations | 60 s |
| `make seed` | Seed reference data (characters, models on provider) | 60 s |
| `make seed-demo` | Deterministic demo dataset | 2 min |
| `make fmt` | `cargo fmt` + Prettier | 60 s |
| `make lint` | `clippy -D warnings` + ESLint | 5 min |
| `make check` | fmt check + lint + typecheck + unit tests + sow-lint + docs links | 3 min |
| `make test` | All Rust + frontend unit/integration | 15 min |
| `make e2e` | Playwright against a running stack | 20 min |
| `make load` | k6 scenarios | 30 min |
| `make doctor` | Verify Docker, ports, disk, tools, keys, connectivity | 30 s |
| `make backup` | `pg_dump` + artifacts manifest | 5 min |
| `make restore-drill` | Restore into a scratch DB and verify checksums | 15 min |
| `make openapi` | Regenerate `.docs/product/api/openapi.json` and client types | 60 s |
| `make asyncapi` | Regenerate `.docs/product/api/asyncapi.json` | 60 s |
| `make sbom` | Generate SBOMs for all images | 5 min |
| `make sow-lint` | Validate SOW statuses/links/checklists | 5 s |
| `make clean` | Remove build artifacts (no source) | 60 s |

**Rules:** every target is idempotent; every target prints a single-line summary on success; failures exit non-zero with a remediation hint. No target requires interactive input.

### 3.6 Local development workflow

- `make up` starts the full stack; `make dev` (compose dev profile) mounts source and enables hot reload for Rust (`cargo-watch`) and Angular (`ng serve` with proxy to `api`).
- Ports (host bind): `8080` web, `3000` api (dev only), `5432` postgres (dev only), `6379` redis (dev only). Production binds only `8080` on loopback.
- `.env` is created from `.env.example`; `AILAB_MASTER_KEY` is generated by `scripts/doctor.sh --generate-key` if absent (dev only) and never committed.
- Database state: `make reset` is safe and reproducible; `make seed-demo` is deterministic (fixed UUIDs, fixed timestamps relative to a fixed epoch).
- Logs: structured JSON; `make logs` pretty-prints in dev.

### 3.7 Hooks and commit policy

- `lefthook` (pinned) installed via `make hooks`:
  - **pre-commit:** `cargo fmt --check` on staged Rust files; Prettier check on staged web files; `gitleaks protect --staged`; reject merge conflict markers.
  - **commit-msg:** Conventional Commits regex; reject empty scope where required; require card key in footer for card work.
  - **pre-push:** `make check` (fast lane only; full tests run in CI).
- Commit format: `type(scope): imperative summary` (≤ 72 chars). Types: `feat`, `fix`, `chore`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`.
- Agent commits MUST include trailers:
  ```
  AI-Lab-Run: <uuid>
  AI-Lab-Card: <KEY>
  AI-Lab-Agent: <uuid>
  ```
- Branch naming: `ailab/{CARD-KEY}-{slug}`; release branches `release/{version}`; no direct commits to `main` (branch protection).

### 3.8 CI workflows

| Workflow | Trigger | Jobs | Required |
|---|---|---|---|
| `ci.yml` | PR, push to `main` | `fmt`, `clippy`, `rust-unit`, `rust-integration`, `frontend-lint`, `frontend-unit`, `frontend-build`, `e2e`, `a11y`, `visual`, `openapi-compat`, `security-secrets`, `security-deps`, `security-image`, `docs-build`, `sow-lint` | Yes |
| `images.yml` | tag, manual | build api/worker/web/runner, Trivy scan, SBOM, sign (cosign keyless) | Release gate |
| `nightly.yml` | cron 02:00 UTC | load (k6), chaos, fuzz, soak, dependency audit | Advisory → blocks release |
| `release.yml` | tag `v*` | changelog, images, docs deploy, GitHub release | Release gate |

CI rules: no secrets in logs; least-privilege `permissions:` per job; concurrency cancels superseded PR runs; caches keyed by lockfile hash; total CI ≤ 15 min on PR.

### 3.9 Dependency and supply-chain governance

- `renovate.json`: weekly schedule (Monday 06:00 UTC), grouped patch/minor, majors require review, security updates immediate, automerge only for dev-dependencies with green CI.
- `deny.toml`: allowlist licenses (Apache-2.0, MIT, BSD-2/3, ISC, Unicode-DFS, MPL-2.0, CC0, OFL); deny unknown licenses, wildcard versions, git dependencies (except explicitly pinned), yanked crates.
- `npm audit --audit-level=high` in CI; `cargo audit` in CI; Trivy for images (HIGH/CRITICAL fail).
- SBOM: CycloneDX per image and for the workspace; stored as release artifacts.
- Lockfiles are always committed; `cargo update`/`npm update` only via Renovate PRs, never ad hoc on feature branches.

### 3.10 Configuration and secrets hygiene

- `.env.example` documents every variable with a safe placeholder; no real secret is ever committed.
- `.gitignore` covers `.env`, `*.pem`, `*.key`, `target/`, `node_modules/`, `dist/`, `artifacts/`, `workspaces/`, `.DS_Store`.
- `gitleaks` scans history and staged changes; secrets found → build fails and incident is opened.
- Local master key is generated, stored only in `.env` (gitignored) and never printed.

### 3.11 Contribution and review

- `CONTRIBUTING.md` includes: setup, branch/commit rules, PR checklist, review checklist, test commands, docs requirements, code style, security rules.
- Review checklist: correctness, scope, tests, security, performance, docs, accessibility (UI), observability, rollback.
- Every PR links a card and a WP; reviewer ≠ author; merger ≠ author.
- CODEOWNERS: backend crates → backend owner; `web/` → frontend owner; `infra/` → platform owner; `.docs/product/` → analyst; `migrations/` → data owner; security-sensitive paths → QA.

### 3.12 Performance of the developer loop

- Incremental Rust check ≤ 10 s warm; Angular rebuild ≤ 5 s warm.
- `make check` ≤ 3 min on reference hardware.
- CI PR pipeline ≤ 15 min; unit tests parallelized with `cargo-nextest`; Playwright sharded 2×.
- Flaky tests are quarantined within 24 h and fixed/deleted within 5 days; a flaky test blocks release.

---

## 4. Deliverables (checklist)

- [x] Monorepo skeleton with every directory and placeholder crate/app (12 crates, Angular app, infra and docs directories).
- [x] `rust-toolchain.toml`, `.tool-versions`, `Cargo.toml` workspace with lints and profiles (Rust 1.99.1, edition 2024, clippy `-D warnings` green).
- [x] `web/` Angular 22.2 workspace, strict TypeScript 6.0 (ADR-0024), ESLint/Prettier, budgets (`npm run lint`, `npm run build`, `npm test` green).
- [x] `Makefile` with all §3.5 targets, each idempotent and non-interactive (`make help` lists 41 targets).
- [x] `scripts/doctor.sh` validating Docker/Compose/ports/disk/tools/keys/connectivity (`make doctor` OK, 4 environment warnings).
- [x] `scripts/sow-lint.sh` validating SOW statuses, links, dependencies and checklists (WP-00; self-test 11/11).
- [x] `lefthook.yml` with pre-commit, commit-msg and pre-push hooks (`scripts/hooks/*`; bad-subject and missing-trailer tests).
- [x] `.github/workflows/ci.yml` with all required jobs; branch protection documented (16 names match).
- [x] `.github/workflows/images.yml`, `nightly.yml`, `release.yml` skeletons with the exact jobs.
- [x] `renovate.json`, `deny.toml`, `.gitignore`, `.dockerignore`, `.editorconfig`, `.gitattributes`.
- [x] `CODEOWNERS`, `CONTRIBUTING.md`, `.docs/product/review-checklist.md`, PR template (WP-00 template).
- [x] `.env.example` complete and consistent with Appendix D.4 (40/40 variables including `AILAB_RUNNER_TOKEN` per CR-0004, placeholders only).
- [x] `README.md` quickstart verified from a clean clone (`make setup && make check` exit 0 in 169 s).
- [x] Contract generation scripts (`gen-openapi.sh`, `gen-asyncapi.sh`) and drift check in CI (generation owned by WP-16; job present).

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP02-T01 | Create monorepo tree + workspace manifests | Dev | 2 | WP-01 | `cargo metadata`, `ng version` succeed |
| WP02-T02 | Pin toolchains and package manager | Dev | 1 | T01 | Versions match §3.2 exactly |
| WP02-T03 | Configure Rust lints/profiles/sqlx offline | Dev | 1 | T01 | `make lint` green on empty workspace |
| WP02-T04 | Configure Angular strict + ESLint/Prettier + budgets | Dev | 2 | T01 | `make frontend-build` green |
| WP02-T05 | Implement Makefile + scripts (doctor, wait-for, sow-lint) | Dev | 2 | T01 | All targets work on clean clone |
| WP02-T06 | Hooks + commit policy | Dev | 1 | T05 | Hooks block bad commits |
| WP02-T07 | CI workflows + branch protection doc | Dev | 3 | T05 | Required checks configured |
| WP02-T08 | Dependency governance (Renovate/deny/audit) | Dev | 1 | T07 | CI security jobs green |
| WP02-T09 | Contribution docs, CODEOWNERS, PR template | Analyst | 2 | T06 | Review checklist complete |
| WP02-T10 | Clean-clone bootstrap test | QA | 1 | All | `make check` ≤ 10 min from clone |

---

## 6. Acceptance criteria

1. From a clean clone on the reference host, `make doctor && make up && make check` succeeds in ≤ 10 minutes without manual steps.
2. `make help` lists every target; every target is idempotent and non-interactive.
3. A commit with a non-conventional message is rejected locally; a commit containing a secret is rejected by gitleaks.
4. CI required checks match the exact names in WP-00 §3.8 and block merge on failure.
5. `cargo-deny` and `npm audit` fail on a deliberately introduced disallowed license or high CVE (test fixture).
6. `.env.example` contains every variable from Appendix D.4 and no real secret.
7. `scripts/sow-lint.sh` detects a broken WP link and an invalid status (test fixtures).
8. `make openapi` is deterministic: running twice produces no diff; CI fails on drift.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence.
- [x] Clean-clone bootstrap test recorded: `make setup && make check` exit 0 in 169 s (macOS, Docker 24.0.7, Compose 2.23.3, Rust 1.99.1, Node container 24.22). `make up` is deferred to WP-03.
- [x] CI green on `main` with all required checks: local `make check` green; remote CI run `42409540109` on `review/wp-02-repository-devex` reports all 16 required checks passing; branch protection applied with the 16 contexts (CR-0003).
- [x] Hooks verified with a bad-commit test: bad subject exit 1, good subject exit 0, missing agent trailers exit 1; gitleaks private-key fixture exit 1.
- [x] Contribution docs reviewed by Tech Lead: independent `APPROVED` review in `.docs/product/evidence/WP-02-review.md`.
- [x] Status `done` with sign-off (2026-11-07).

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Toolchain drift between local and CI | High | Pinned files; CI version check; `.tool-versions` |
| Slow developer loop | Medium | Incremental profiles, nextest, caching, budgets |
| CI minutes blow up | Medium | Concurrency cancel, sharding, targeted jobs |
| Supply-chain compromise | High | Lockfiles, deny, audit, SBOM, pinned digests |
| Hidden manual steps | Medium | Clean-clone test; doctor script; no interactive targets |
| Pre-commit hooks bypassed | Medium | CI re-runs all checks; hooks are convenience, not the gate |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §5, §13, §20, §21
- `../technical-specification/APPENDIX-D-DELIVERY.md` §D.1 (M0), §D.4
- `SOW.md` §10, §11

## Progress log

2026-11-07 | T01 | Cargo.toml, crates/*, web/* | cargo metadata OK; Angular build OK | — | T02
2026-11-07 | T02 | rust-toolchain.toml, .tool-versions, Cargo.toml, web/package.json | rustc 1.99.1; Node 24.22; npm 11.19 | ADR-0024 (TS 6.0.x) | T03
2026-11-07 | T03 | Cargo.toml, rustfmt.toml, clippy.toml, crates/* | cargo fmt CHECK OK; cargo clippy -D warnings OK | — | T04
2026-11-07 | T04 | web/tsconfig.json, web/angular.json, web/eslint.config.js, web/.prettierrc, web/package.json | npm lint OK; npm run build OK (217 kB raw, 59.6 kB transfer); npm test 2/2 | ADR-0024 | T05
2026-11-07 | T05 | Makefile, scripts/doctor.sh, scripts/wait-for.sh, scripts/run-web.sh, scripts/hooks/* | make help lists 33 targets; doctor OK (4 warnings) | — | T06
2026-11-07 | T06 | lefthook.yml, scripts/hooks/commit-msg.sh, pre-commit.sh, pre-push.sh, install.sh | commit-msg rejects bad subject; accepts conventional | — | T07
2026-11-07 | T07 | .github/workflows/ci.yml, images.yml, nightly.yml, release.yml, .github/required-checks.txt | 16 required job names match | — | T08
2026-11-07 | T08 | renovate.json, deny.toml, .gitignore, .dockerignore, .editorconfig, .gitattributes, .env.example | JSON/YAML parse OK; env covers Appendix D.4 | ADR-0024 (TS major blocked) | T09
2026-11-07 | T09 | CODEOWNERS, CONTRIBUTING.md, .docs/product/review-checklist.md, CHANGELOG.md, SECURITY.md, CODE_OF_CONDUCT.md, LICENSE | docs link check OK | — | T10
2026-11-07 | T10 | clean-clone simulation | make setup + make check exit 0 in 169 s (macOS, Docker 24.0.7, Compose 2.23.3, Rust 1.99.1, Node container 24.22); frontend build 217.16 kB raw / 59.63 kB transfer; npm audit high gate exit 0 (15 moderate); gitleaks private-key fixture exit 1 | — | review
2026-11-07 | review-fix | .tools/test-supply-chain.sh, .tools/fixtures/supply-chain/*, .github/ISSUE_TEMPLATE/*, .docs/adr/registry.md, web/angular.json, .github/workflows/ci.yml, Makefile | supply-chain 2/2 negative fixtures rejected; make check OK; frontend build OK | ADR-0024 | re-review
2026-11-07 | closure | .docs/product/evidence/WP-02-review.md | independent review APPROVED | AC1 WP-03, AC8 WP-16 | WP-03
2026-11-07 | tool-fix | scripts/sow-lint.sh | replaced recursive bash-3.2 cycle detection with an awk topological sort; 20/20 stable runs; self-test 11/11 | — | gates
2026-11-07 | review | .docs/sow/WP-02-REPOSITORY-DEVEX-review-plan.md, CR-0004 | make check OK; supply-chain 2/2; hooks reject bad input; openapi deterministic; env 40/40 | CR-0004 | QA review
