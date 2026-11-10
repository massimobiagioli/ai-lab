# AI-Lab Acceptance Report (AC-001..015)

| Field | Value |
|---|---|
| Version | 1.0.0-draft |
| Status | Incomplete — P14 draft, pending P15 release evidence and operator acceptance |
| Owner | QA (agent) |
| Related | `.docs/sow/WP-22-DOCS-RELEASE.md` §3.4 |

Status meanings: `evidenced` (artifact and reproducible evidence linked), `partial` (evidence exists but a named gap remains), `not evidenced` (no acceptable evidence yet).

| AC | Criterion | Evidence | Status |
|---|---|---|---|
| AC-001 | Bootstrap | `scripts/doctor.sh`, `make reset` (2026-11-09: 0 → running), `README.md` | partial: timed clean-clone log and video missing |
| AC-002 | Inventory | WP-19 inventory tests, `make frontend-test`; API inventory tests | partial: E2E recording missing |
| AC-003 | Idea → cards | WP-10 materialization tests; SOW JSON fixtures | partial: timing histogram missing |
| AC-004 | Kanban invariants | Orchestrator/application guard tests, `make test` | partial: matrix report not exported |
| AC-005 | Office | WP-18 office tests, Playwright `/` smoke and axe | partial: FPS report and screenshots missing |
| AC-006 | SOW and plan | WP-10 tests, acyclic WBS fixtures, `.docs/product/sow.md` | partial: traceability export missing |
| AC-007 | Execution | WP-13 run pipeline tests | not evidenced: no live run trace and merge SHA |
| AC-008 | Memory | WP-08 retrieval and leakage tests | partial: nDCG report missing |
| AC-009 | CV | WP-07/WP-19 CV tests | partial: version evidence links missing |
| AC-010 | Autonomy and budget | Policy matrix tests, budget-stop tests | partial: signed policy matrix missing |
| AC-011 | Security | `gitleaks`, `cargo deny`, `npm audit`, `.docs/product/security.md` | partial: ASVS checklist missing |
| AC-012 | SLOs | k6 1 VU 0 % failed, p95 2.6 ms (2026-11-09); `.docs/product/slo.md` | partial: seven dashboards and synthetic alert pending WP-24 T07 |
| AC-013 | DR | Restore drill, `.docs/product/evidence/WP-20-restore-drill.md` (52 s) | evidenced |
| AC-014 | Docs | `.tools/check-doc-links.sh` 171 links; `docs-manifest.md` | evidenced |
| AC-015 | Demo | `.docs/product/demo-script.md` | not evidenced: recording and timings missing |

## Sign-off

| Role | Name | Date | Verdict |
|---|---|---|---|
| Tech Lead | pending | — | — |
| QA | qa-review-agent | 2026-11-09 | NOT APPROVED (draft) |
| Human Operator | pending | — | — |

The report cannot be signed until P15 lands the release evidence and every `partial`/`not evidenced` row closes.
