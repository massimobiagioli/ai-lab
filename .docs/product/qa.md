# AI-Lab QA & Merge

| Field | Value |
|---|---|
| Document | `.docs/product/qa.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | QA |
| Related | `.docs/product/kanban.md`, `.docs/product/security.md`, `crates/application/src/qa.rs` |

QA is independent verification, not a phase. The merge policy enforces separation of duties, green checks, DoD and autonomy.

## 1. Verdicts

- `Approved`: the reviewer verified every acceptance criterion.
- `ChangesRequested`: findings are attached and the card returns to `DOING`.

The reviewer must differ from the author; a self-review is rejected with `SEPARATION_OF_DUTIES`.

## 2. Merge checks

A merge is allowed only when all hold: CI green, PR merged, merge SHA recorded, DoD complete, no P0/P1 open, budget remaining, QA approved and reviewer ≠ author.

## 3. Autonomy

| Level | Merge authority |
|---|---|
| A0 | Human approves every transition and merge |
| A1 | Agents move to REVIEW; human merges |
| A2 | Agents merge with QA approval, green CI, no P0/P1 and budget |
| A3 | As A2, plus project/dependency/release-patch scope |

A0/A1 produce `HUMAN_MERGE_REQUIRED` for an agent merge. A2/A3 allow a clean merge to proceed.

## 4. Violations

Stable codes: `SEPARATION_OF_DUTIES`, `QA_NOT_APPROVED`, `CI_NOT_GREEN`, `PR_NOT_MERGED`, `MERGE_SHA_MISSING`, `DOD_INCOMPLETE`, `P0_P1_OPEN`, `BUDGET_EXHAUSTED`, `HUMAN_MERGE_REQUIRED`.

## 5. Tests

`cargo test -p ailab-application` covers a clean A2 merge, self-review, human-required A1, and the collected violation set.

## 6. Deferred to consumer WPs

- Reviewer assignment and UI: WP-09/WP-19.
- CI/merge API integration: WP-12/WP-16.
- Independent review matrix and E2E: WP-21.
- Release acceptance: WP-22.

## 7. References

- `.docs/sow/WP-15-QA-MERGE.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.4, §7.4
- `.docs/adr/adr-0011.md`, `.docs/adr/adr-0018.md`
- `crates/application/src/qa.rs`
