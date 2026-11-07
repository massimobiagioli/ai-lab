# AI-Lab Quality Gates

| Field | Value |
|---|---|
| Document | `.docs/product/quality-gates.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | QA |
| Related | `.docs/product/governance.md`, `.docs/product/status-model.md`, `.docs/product/metrics.md` |
| Enforcement | CI workflow `.github/workflows/ci.yml`, branch protection, `scripts/sow-lint.sh` |

A pull request may merge only when every applicable gate below is green. Branch protection requires all listed status check names. A gate bypass is a P1 incident; it is never a "temporary" exception.

---

## 1. Required check names

These names are exact. They are the required status contexts for the default branch and the source of truth for branch protection.

| Check | Owner | Command | Threshold | Failure action |
|---|---|---|---|---|
| `fmt` | Dev | `cargo fmt --all --check` | Zero diff | Author formats; merge blocked |
| `clippy` | Dev | `cargo clippy --workspace --all-targets -- -D warnings` | Zero warnings | Author fixes; merge blocked |
| `rust-unit` | Dev | `cargo nextest run --workspace --lib --bins` | All pass | Author fixes; P1 if flaky > 24 h |
| `rust-integration` | Dev | `cargo nextest run --workspace --test '*'` | All pass | Author fixes; merge blocked |
| `frontend-lint` | Dev | `npm --prefix web run lint` | Zero errors | Author fixes; merge blocked |
| `frontend-unit` | Dev | `npm --prefix web test -- --run` | All pass; line coverage ≥ 80% | Author fixes; merge blocked |
| `frontend-build` | Dev | `npm --prefix web run build` | Exit 0; bundle budgets met | Author fixes; merge blocked |
| `e2e` | QA | `npx --prefix web playwright test` | All pass; no new flaky entry | QA triages; release blocked |
| `a11y` | QA | `npx --prefix web playwright test --project=a11y` | Zero serious/critical axe findings | QA files defect; merge blocked |
| `visual` | QA | `npx --prefix web playwright test --project=visual` | Within approved threshold | QA reviews diff; merge blocked |
| `openapi-compat` | Dev | `make openapi` + `make asyncapi` + `git diff --exit-code` | No drift | Author regenerates; merge blocked |
| `security-secrets` | QA | `gitleaks detect --source . --no-git` | Zero findings | Revoke secret; rotate; P0 |
| `security-deps` | QA | `cargo deny check` + `npm --prefix web audit --audit-level=high` + `.tools/test-supply-chain.sh` | No high/critical; negative fixtures rejected | Upgrade or approved ADR; merge blocked |
| `security-image` | QA | `trivy image <changed-image>` | No high/critical with a fix | Rebuild; merge blocked |
| `docs-build` | Analyst | `scripts/harness-lint.sh` + docs build + link check + `.tools/check-architecture-fitness.sh` | Zero broken links; fitness green; build exit 0 | Author fixes; release blocked |
| `sow-lint` | Tech Lead | `scripts/sow-lint.sh` | Exit 0 | Author fixes; merge blocked |

## 2. Coverage thresholds

| Scope | Threshold | Enforced by |
|---|---|---|
| Backend line | ≥ 80% | `rust-unit` + coverage report |
| Domain line | ≥ 90% | `rust-unit` + domain coverage report |
| Frontend line | ≥ 80% | `frontend-unit` |
| New code | ≥ 80% | Diff coverage in the PR |

Coverage may not be lowered to unblock a merge. A waiver requires a Tech Lead decision recorded in the PR and a follow-up card.

## 3. Security gates

- **Secrets:** `gitleaks` runs on every push and PR. A finding is a P0: revoke, rotate, purge, then reopen the PR.
- **Dependencies:** `cargo-deny` checks advisories, licenses, bans and sources; `npm audit` fails on high and critical.
- **Images:** Trivy scans every changed image; no high or critical finding with an available fix may ship.
- **Supply chain:** lockfiles are committed; images are digest-pinned; SBOM and signatures are produced at release.
- **Sandbox:** the escape suite and prompt-injection fixtures are required before WP-14 and WP-15 close.

No test may call the real DeepSeek or GitHub API. Fakes and fixtures are mandatory.

## 4. Contract gates

- OpenAPI and AsyncAPI are regenerated with `make openapi` and `make asyncapi`; drift fails `openapi-compat`.
- Schemathesis runs against `.docs/product/api/openapi.json`; zero failures allowed.
- The Kanban 36-case transition matrix and DoR/DoD guard suite are required before merge for WP-09 changes.
- The event catalog schema check is required for WP-06/WP-16 changes.

## 5. Performance and resilience gates

| Gate | Command | Threshold |
|---|---|---|
| Load | `make load` | API read p95 ≤ 120 ms, p99 ≤ 300 ms; write p95 ≤ 250 ms, p99 ≤ 600 ms |
| Board | Playwright + k6 | 500 cards p95 ≤ 400 ms |
| Realtime | tracing + Playwright | WS event → render p95 ≤ 250 ms |
| Office FPS | RAF probe | ≥ 58 desktop, ≥ 30 low-end |
| Soak | `make load` 60 min | Memory growth ≤ 10%, error rate ≤ 0.1% |
| Restore | `make restore-drill` | Restore ≤ RTO; checksums match |

Performance gates run nightly and before release. A regression blocks release.

## 6. Documentation gates

- `.docs/` changes require `scripts/harness-lint.sh`.
- Product documentation changes require a docs build and link check.
- A user-visible change requires a changelog entry.
- Architecture or contract changes require an ADR.
- Screenshots and GIFs are regenerated from the deterministic seed and must match the current UI.

## 7. Branch protection

The default branch requires every check in §1 to pass, requires a pull request, dismisses stale approvals on new commits, and blocks force pushes and deletions. The required contexts are stored in `.github/required-checks.txt`.

Applying branch protection is an operator action performed once a GitHub remote exists:

```bash
gh api -X PUT "repos/{owner}/{repo}/branches/main/protection" \
  --input .github/branch-protection.json
```

Until a remote exists, the repository records branch protection as an open external blocker. The check names and configuration file are delivered so the action is deterministic.

## 8. Gate ownership and escalation

| Situation | Owner | Action |
|---|---|---|
| Author cannot make a gate pass | Dev | Escalate to Tech Lead within 1 working day |
| Gate is flaky | QA | Quarantine within 24 h with an owner and card; fix or delete within 5 days |
| Gate is wrong or too strict | Tech Lead | Change through a CR; never bypass silently |
| Security finding | QA | P0/P1 path; incident runbook; release blocked |
| Gate bypassed | Tech Lead | Record a P1 incident and a retrospective action |

## 9. Local equivalents

`make check` is the fast local gate and MUST run the same commands as `fmt`, `clippy`, `frontend-lint`, `frontend-unit`, `sow-lint` and `docs-build` link checking. WP-02 owns the Makefile implementation; until it exists, run the commands in §1 directly.

## 10. Evidence

Each PR records, under `Verification`:

- the exact commands run and their results;
- the coverage number for new code;
- links to reports, screenshots, benchmarks or run IDs;
- any approved waiver with its approver and follow-up card.

## 11. References

- `.docs/sow/SOW.md` §10
- `.docs/sow/WP-00-GOVERNANCE.md` §3.8
- `.docs/CONSTITUTION.md` §6–§7
- `.docs/PRINCIPLES.md` §2
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §17, §19
- `.docs/technical-specification/APPENDIX-D-DELIVERY.md` §D.2
