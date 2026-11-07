# WP-02 — Repository, Toolchain & Developer Experience Review Plan

| Field | Value |
|---|---|
| WP | `WP-02` |
| Contract | `.docs/sow/WP-02-REPOSITORY-DEVEX.md` |
| Review branch | `review/wp-02-repository-devex` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_review` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present and consistent.
- [x] Requirement traceability (`VER-001..008`, `OPS-010..014`, `NFR-011`, `DOC-001..003`) maps to the specification sections cited in §9.

## 2. Specification conformance

- [x] Specification §13, §20 (repository), §21 and §5 (stack) and Appendix D.4 (environment variables) — evidence: `Makefile`, `rust-toolchain.toml`, `.tool-versions`, `.env.example`, `APPENDIX-D` §D.4.
- [x] Appendix C.4 PR template and WP-00 §3.8 required checks — evidence: `.github/pull_request_template.md`, `.github/required-checks.txt`, `ci.yml`.

## 3. Deliverables

- [x] Monorepo skeleton (12 crates, Angular app, infra, docs) — evidence: `crates/`, `web/`, `infra/`, `.docs/`.
- [x] `rust-toolchain.toml`, `.tool-versions`, workspace lints/profiles — evidence: files; `cargo metadata` and `clippy -D warnings` green.
- [x] `web/` Angular strict workspace — evidence: `npm run lint`, `npm run build`, `npm test` via `make check` (Node container).
- [x] `Makefile` with all §3.5 targets, idempotent and non-interactive — evidence: `make help` lists 41 targets.
- [x] `scripts/doctor.sh` — evidence: `make doctor` runs; `--generate-key`.
- [x] `scripts/sow-lint.sh` + WP-00 self-test — evidence: `.tools/sow-lint-self-test.sh` 11/11.
- [x] Hooks (`lefthook.yml`, `scripts/hooks/*`) — evidence: bad subject exit 1, good exit 0, missing trailers exit 1, merge-marker rejection.
- [x] `.github/workflows/ci.yml` with the 16 required jobs — evidence: 16 names match `.github/required-checks.txt`.
- [x] `images.yml`, `nightly.yml`, `release.yml` skeletons — evidence: files.
- [x] `renovate.json`, `deny.toml`, `.gitignore`, `.dockerignore`, `.editorconfig`, `.gitattributes` — evidence: files; supply-chain negatives.
- [x] `CODEOWNERS`, `CONTRIBUTING.md`, `review-checklist.md`, PR template — evidence: files.
- [x] `.env.example` complete vs Appendix D.4 — evidence: 40/40 including `AILAB_RUNNER_TOKEN` (CR-0004); placeholders only.
- [x] `README.md` quickstart — evidence: `make check` green on the reference host.
- [x] Contract generation scripts + drift job — evidence: `scripts/gen-openapi.sh`, `scripts/gen-asyncapi.sh`; generation owned by WP-16.

## 4. Acceptance criteria

- [x] AC1 clean clone `make doctor && make up && make check` ≤ 10 min — `make check` green; `make up` is WP-03-owned and the Compose file now exists; recorded as downstream.
- [x] AC2 `make help` lists every target; non-interactive — evidence: 41 targets, all fail-fast guards, no prompts.
- [x] AC3 bad commit and secret rejected locally — evidence: `scripts/hooks/commit-msg.sh` bad exit 1 / good exit 0 / missing trailers exit 1; gitleaks fixture exit 1.
- [x] AC4 CI checks match WP-00 §3.8 and block merge — evidence: 16 job keys, `required-checks.txt`, branch protection (CR-0003).
- [x] AC5 `cargo-deny`/`npm audit` reject a disallowed license and high CVE — evidence: `.tools/test-supply-chain.sh` 2/2 negatives.
- [x] AC6 `.env.example` has every Appendix D.4 variable and no real secret — evidence: 40/40 (CR-0004), placeholders only.
- [x] AC7 `sow-lint` detects invalid status and broken link — evidence: `.tools/sow-lint-self-test.sh` 11/11.
- [x] AC8 `make openapi` deterministic; CI drift check — evidence: two consecutive runs leave no diff; generation is WP-16-owned.

## 5. Implementation and tests

- [x] Toolchain pinning enforced — files match §3.2.
- [x] Hooks reject bad input — test: direct hook invocation.
- [x] Supply-chain negatives — test: `.tools/test-supply-chain.sh`.
- [x] No real external API in tests.
- [x] Coverage thresholds met (repository/tooling WP; `make check` green).

## 6. Security and quality gates

- [x] No secret in `.env.example`; gitleaks hook rejects the fixture.
- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `make check` green; `make openapi`/`make asyncapi` deterministic; `.tools/test-supply-chain.sh` green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/review-checklist.md` and contribution docs updated.
- [x] `.docs/adr/registry.md` consistent; ADR-0024 linked.
- [x] `AGENTS.md` indexes updated for this review plan.
- [x] `.docs/sow/SOW.md` and WP-02 status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED`.
- [ ] QA evidence: reproduced gates (harness 16, sow-lint 31, doc links 141, `make check`, `cargo deny check`, supply-chain 2/2, hooks, openapi determinism, remote CI run 42409540109 with all 16 required checks).
- [x] First QA pass: `CHANGES_REQUESTED` for a broken `frontend-unit` command and the uncorrected TypeScript pin; fixed (CI/quality-gates corrected, CR-0005).
- [x] Second QA pass: `CHANGES_REQUESTED` for the unresolvable Trivy action, `cargo deny` wildcard failures and the unsupported CI-green claim; fixed (trivy v0.36.0, `publish = false`, on-demand image pull) and verified with a green remote CI run.

## 9. Deferred items and residual risks

- [x] AC1 `make up` end-to-end — owner: Dev, WP-03, Compose delivered.
- [x] AC8 server-side OpenAPI/AsyncAPI generation — owner: Dev, WP-16, deterministic copy in place.
- [x] Coverage enforcement (backend/domain/frontend) — owner: QA, WP-21; the gate table is normative, enforcement tracked there.
- [x] No unexplained deferral remains.

## 10. Decisions

- [x] CR-0004 adds `AILAB_RUNNER_TOKEN` to Appendix D.4 (linked ADR-0023).
- [x] CR-0005 aligns specification §5.1 TypeScript pin to 6.0.x (linked ADR-0024).
