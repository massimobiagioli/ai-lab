# WP-22 — Documentation, Release & Handover Review Plan

| Field | Value |
|---|---|
| WP | `WP-22` |
| Contract | `.docs/sow/WP-22-DOCS-RELEASE.md` |
| Review branches | `review/p14-wp22-docs`, `review/p15-wp22-release` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_progress` |

## 1. Contract completeness

- [x] Header, objective, scope, §3.1–3.7, deliverables, tasks, acceptance criteria, DoD, risks and references present.
- [x] CR-0018 reopens WP-22 with a status-history entry and a progress log.

## 2. Specification conformance

- [x] Documentation manifest audited (`.docs/product/docs-manifest.md`): 19 artifacts, gaps named.
- [x] ADR set accepted and linked; architecture, domain, database, API, AsyncAPI, Kanban, agents, office, security, operations and testing docs present and link-checked.
- [x] Runbooks and the operator guide present; release checklist and changelog present.
- [x] Acceptance report maps AC-001..015 to evidence and states the gaps (`.docs/product/acceptance-report.md`).
- [x] Golden demo script within five minutes (`.docs/product/demo-script.md`).
- [x] Handover checklist, ownership, limitations and support boundaries (`.docs/product/handover.md`).
- [ ] SBOM, signing, release 1.0, rollback — P15.

## 3. Deliverables

- [x] CR-0018, review plan, docs manifest, acceptance report, testing doc and handover.
- [x] Product documentation index updated.
- [ ] Release artifacts and signatures — P15.

## 4. Security

- [x] No secret in the new docs; `gitleaks` and `cargo deny` are part of `make check`.
- [x] Disclosure policy and support boundaries documented.

## 5. Review and QA independence

- [x] The reviewer is not the author; the QA verdict is a separate file.
- [ ] Final sign-off after P15.

## 6. Gates and evidence

- [x] `scripts/harness-lint.sh`, `scripts/sow-lint.sh`, `.tools/check-doc-links.sh`, `make check`.
- [ ] Release/SBOM/signing gates — P15.

## 7. Verdict

Documentation and handover are delivered; the acceptance report is honest about the remaining evidence gaps. WP-22 stays `progress` until P15 and the AC evidence land.

## Progress log

2026-11-09 | P14 | .docs/sow/WP-22-DOCS-RELEASE-review-plan.md, .docs/product/{docs-manifest,acceptance-report,handover,testing}.md | review plan and package; docs link check | release/SBOM deferred P15 | P15
