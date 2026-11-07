# AI-Lab Testing

| Field | Value |
|---|---|
| Document | `docs/testing.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | QA |
| Related | `docs/quality-gates.md`, `.docs/sow/WP-21-TESTING.md` |

Testing follows TDD (Red → Green → Refactor) and is deterministic: no wall-clock dependence, no sleep-based synchronization and no calls to real DeepSeek or GitHub.

## 1. Test layers

| Layer | Tool | Scope | Status |
|---|---|---|---|
| Unit | `cargo test` / Vitest | Domain, application, infrastructure, components | Delivered |
| Integration | `cargo test` + fixtures | Migrations, constraints, analytics, integrity | Delivered |
| Contract | Fake transports + JSON contracts | LLM, SCM, OpenAPI/AsyncAPI | Delivered |
| E2E | Playwright | J1–J4 journeys | Deferred (WP-21 follow-up) |
| Visual / a11y | Playwright + axe | Office and product UI | Deferred |
| Load / soak | k6 | SLO budgets | Deferred |
| Chaos / fuzz | Scripted + `cargo fuzz` | Failure modes, parsers | Deferred |
| Security | gitleaks, cargo-deny, npm audit, Trivy, fixtures | Secrets, deps, images, sandbox | Partial |

## 2. Commands

```bash
cargo test --workspace
npm --prefix web test
npm --prefix web run lint
npm --prefix web run build
scripts/harness-lint.sh
scripts/sow-lint.sh
.tools/check-doc-links.sh
.tools/check-crate-dependencies.sh
.tools/check-architecture-fitness.sh
.tools/test-migrations.sh
.tools/test-constraints.sh
.tools/test-analytics-views.sh
.tools/test-supply-chain.sh
make check
make db-check
```

## 3. Fixtures and determinism

- `scripts/seed-demo.sql`: fixed UUIDs, idempotent.
- `.tools/fixtures/analytics-seed.sql`: fixed analytics values.
- Fake `HttpTransport` and `LlmProvider` implementations for LLM/SCM tests.
- Fixed clocks and sequential ids in `crates/testkit`.
- The office and dashboard tests use fixed metrics and deterministic frames.

## 4. Coverage

Backend line ≥ 80%, domain ≥ 90%, frontend ≥ 80%, new code ≥ 80%. Coverage may not be lowered to unblock a merge; a waiver needs a Tech Lead decision and a follow-up card.

## 5. Flaky policy

- Quarantine within 24 hours with an owner and a card.
- Fix or delete within 5 days.
- A flaky test blocks release; retries must not be used to hide flakiness.

## 6. Acceptance catalog

The `AT-*` tests in Appendix D remain the release contract. AT-001/AT-024/AT-027/AT-028 are exercised by the local gates; AT-007/AT-008/AT-012/AT-025/AT-026/AT-029 require the Playwright, k6 and chaos suites in the WP-21 follow-up.

## 7. References

- `.docs/sow/WP-21-TESTING.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §19
- `.docs/technical-specification/APPENDIX-D-DELIVERY.md` §D.2
- `docs/quality-gates.md`
