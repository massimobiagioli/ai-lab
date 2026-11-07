# AI-Lab Release Checklist

| Field | Value |
|---|---|
| Document | `docs/release-checklist.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Analyst |
| Related | `docs/quality-gates.md`, `docs/testing.md`, `docs/demo-script.md` |

## 1. Quality gates

- [ ] `scripts/harness-lint.sh` green.
- [ ] `scripts/sow-lint.sh` green with every WP `done`.
- [ ] `make check` green.
- [ ] `cargo test --workspace` green.
- [ ] Frontend lint/test/build green.
- [ ] Playwright E2E/a11y/visual green (WP-21 follow-up).
- [ ] `openapi-compat` and `asyncapi` drift check green.
- [ ] gitleaks, cargo-deny, npm audit high, Trivy high/critical clean.
- [ ] No open P0/P1; reviewer ≠ author; merger ≠ author.

## 2. Artifacts

- [ ] SBOM (CycloneDX) per image.
- [ ] Images signed with cosign.
- [ ] CHANGELOG updated; SemVer tag `v1.0.0`.
- [ ] API and AsyncAPI contracts published.
- [ ] ADR registry current.

## 3. Documentation

- [ ] README quickstart verified from a clean clone.
- [ ] Documentation manifest current (architecture, domain, database, Kanban, agents, office, security, operations, testing, contributing, demo, glossary).
- [ ] Docs build and link check green.
- [ ] Screenshots/GIFs generated from the deterministic seed.

## 4. Acceptance and rollback

- [ ] Golden demo recorded in ≤ 5 minutes.
- [ ] Acceptance report signed by QA and the Human Operator.
- [ ] Backup/restore drill recorded; RTO/RPO met.
- [ ] Rollback: previous image digests and the restore procedure documented.
- [ ] Branch protection applied when a remote exists.

## 5. Post-release

- [ ] Retrospective with three owned actions.
- [ ] `TODO` cards for corrective actions.
- [ ] Post-release health check after 24 hours.

## 6. Current gaps

- Playwright/k6/chaos suites (WP-21 follow-up).
- Live GitHub and DeepSeek smoke tests (operator credentials).
- Branch protection (no remote yet).
- Full product pages (WP-19 follow-up).
