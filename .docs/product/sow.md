# AI-Lab SOW

| Field | Value |
|---|---|
| Document | `.docs/product/sow.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Analyst |
| Related | `.docs/product/planning.md`, `crates/domain/src/delivery.rs` |

A SOW is a versioned structured document. Approval freezes a version; later changes create a new immutable version.

## 1. Mandatory sections

Executive summary, objectives, scope in, scope out, stakeholders, functional requirements, non-functional requirements, architecture, milestones, deliverables, acceptance criteria, assumptions, risks, timeline, budget, governance, change control.

A document is rejected if a required section is missing or empty, if the executive summary is under 120 characters, if no `FR-*`/`NFR-*` requirement exists, or if a requirement id is duplicated. `SowDocument::validate_requirements` additionally rejects a missing priority or acceptance criterion, an unknown FR dependency, and an NFR without a target or measurement.

## 2. Lifecycle

```
Draft → InReview → Approved → Superseded
                  ↘ Rejected
Approved/Draft --revise--> Draft (new immutable version)
```

- Versions are immutable and numbered from 1.
- `revise` requires a non-empty change note and returns to draft.
- `approve` records the approver and freezes the current version.
- `reject` requires a reason and returns the SOW to draft.
- `supersede` requires an approved SOW with a newer approved version.
- The Markdown rendering is deterministic and its SHA-256 is the content hash.

## 3. Tests

`scripts/run-rust.sh cargo test -p ailab-domain` covers required sections, duplicate requirement IDs, structured-requirement validation, deterministic Markdown, the submit/approve/reject/revise/supersede lifecycle, version immutability and the change-note guard.

## 4. Deferred to consumer WPs

- Idea intake, dedupe and fallback card: WP-10 follow-up/WP-16.
- JSON Schema, diff and PDF export: WP-16/WP-22.
- Generation pipeline with streaming and repair: WP-10 follow-up using WP-11.
- APIs, DTOs, OpenAPI, events and audit: WP-16.

## 5. References

- `.docs/sow/WP-10-SOW-PLANNING.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.5
- `.docs/technical-specification/APPENDIX-B-CONTRACTS.md`
- `crates/domain/src/delivery.rs`
