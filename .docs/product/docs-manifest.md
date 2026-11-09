# AI-Lab Documentation Manifest

| Field | Value |
|---|---|
| Version | 1.0.0 |
| Status | Review (P14) |
| Gate | `DOC-GATE`: a release is blocked if any artifact is missing, the docs build fails, links break or screenshots are stale |

Paths are relative to the repository root. `present` means the artifact exists and is link-checked; `partial` means it exists but carries an explicit gap; `missing` means it must land before release.

| # | Artifact | Path | Status | Evidence / gap |
|---|---|---|---|---|
| 1 | README and quickstart | `README.md` | present | Clean-clone steps; timing video pending AC-001 |
| 2 | Architecture (C4) | `.docs/product/architecture.md` | present | Context/container/component and trust boundaries |
| 3 | Domain model | `.docs/product/domain-model.md` | present | Aggregates, invariants and events |
| 4 | ADRs | `.docs/adr/` | present | 25 accepted ADRs, registry linked |
| 5 | API reference | `.docs/product/api/openapi.json` | present | Generated; contract tests in `make check` |
| 6 | Event reference | `.docs/product/api/asyncapi.json` | present | Generated from the event catalog |
| 7 | Database dictionary and ERD | `.docs/product/database.md` | present | Tables, columns, indexes and ownership |
| 8 | Kanban semantics | `.docs/product/kanban.md` | present | Matrix, DoR/DoD and error codes |
| 9 | Agents and prompts | `.docs/product/agents.md` | present | Charters, roster, order and guardrails |
| 10 | Office architecture | `.docs/product/office.md` | present | Engine, assets, licenses and budget |
| 11 | Security | `.docs/product/security.md` | present | Threat model, sandbox and exposure |
| 12 | Operations | `.docs/product/operations.md` | present | Install, backup, restore, upgrade and scale |
| 13 | Runbooks | `.docs/product/runbooks/` | present | 11 runbooks with symptoms and remediation |
| 14 | Testing | `.docs/product/testing.md` | present | Commands, fixtures, seed and flaky policy |
| 15 | Contributing | `CONTRIBUTING.md` | present | Conventions, review checklist and DoD |
| 16 | Diagrams | `.docs/product/diagrams/` | present | Mermaid source plus SVG; regenerable |
| 17 | Changelog, SECURITY, LICENSE, CoC | repo root | present | SemVer changelog, Apache-2.0, disclosure and CoC |
| 18 | Demo script and media | `.docs/product/demo-script.md` | partial | Script ≤5 min; recording (`media/demo.mp4`) missing (AC-015) |
| 19 | Glossary | `.docs/product/glossary.md` | present | Terms match the domain model |

## Open items

- Demo recording and captions (P15 / AC-015).
- Timed clean-clone evidence (AC-001).
- The acceptance gaps recorded in `.docs/product/acceptance-report.md`.
