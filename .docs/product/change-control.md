# AI-Lab Change Control

| Field | Value |
|---|---|
| Document | `.docs/product/change-control.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Tech Lead |
| Record location | `.docs/product/change-requests/` (one file per CR) |
| Related | `.docs/product/governance.md`, `.docs/product/status-model.md`, `.docs/product/quality-gates.md` |

Change control keeps scope, architecture, contracts and budget decisions explicit and auditable. A change request (CR) is a record, not an approval shortcut: it never deletes history, it supersedes.

---

## 1. When a CR is required

A CR is required for:

1. A scope change, including a new feature or a removed deliverable.
2. A new dependency outside the approved stack.
3. An architecture or technology change.
4. A contract break in REST, WebSocket, AsyncAPI, database schema or event catalog.
5. Reopening a `done` work package.
6. A budget, autonomy or exposure change.
7. Milestone re-baselining.

Small, reversible implementation decisions inside an approved card do not need a CR. They are recorded in the card log or, when architectural, as an ADR.

## 2. CR form (exact fields)

Every CR file uses these field names:

| Field | Meaning |
|---|---|
| `id` | `CR-NNNN`, allocated sequentially and never reused. |
| `title` | Imperative summary. |
| `context` | Why the change is proposed now. |
| `proposed_change` | Exactly what changes. |
| `impact_scope` | Requirements, WPs, cards and contracts affected. |
| `impact_schedule` | Milestones and dates affected. |
| `impact_cost` | Budget and effort delta. |
| `impact_risk` | Risks introduced or retired. |
| `alternatives` | Considered options and why they were rejected. |
| `decision` | `approved`, `rejected` or `deferred`, with rationale. |
| `approver` | Role and date. |
| `date` | Decision date (UTC). |
| `linked_adr` | ADR identifier or `—`. |

## 3. Approval matrix

| Change | Approver | Consulted |
|---|---|---|
| WP internal task reordering without scope change | Tech Lead | Dev |
| New dependency within the approved stack | Tech Lead | Dev, QA |
| Architecture or contract change | Tech Lead + ADR | Analyst, Dev, QA |
| Scope change, milestone re-baseline, budget/autonomy/exposure change | Human Operator | Tech Lead, QA |
| Security exception | QA + Human Operator, with expiry date | Tech Lead |

A rejected CR is retained in `.docs/product/change-requests/` with `decision: rejected`. A deferred CR states the condition that reopens it.

## 4. Process

1. Author creates `.docs/product/change-requests/CR-NNNN.md` from `.docs/product/templates/change-request.md`.
2. Author fills every field and links affected WPs, cards and contracts.
3. The approver records the decision in the same file. The file is never rewritten to hide the original proposal; a superseding CR is created instead.
4. When the change is architectural, an ADR is created or superseded in the same change.
5. Affected WP status histories record the CR identifier.
6. `scripts/sow-lint.sh` validates that every CR referenced by a WP exists and has all required fields.

## 5. Emergency security path

An emergency security fix may bypass the normal pull order when a P0/P1 security issue is active. It MUST still:

1. Pass the security gates (`security-secrets`, `security-deps`, `security-image`) and the automated test suite.
2. Be reviewed by an actor other than the author.
3. Be retro-documented with a CR and, when architectural, an ADR within 24 hours.

## 6. Records

- Register: `.docs/product/change-requests/README.md`.
- Files: `.docs/product/change-requests/CR-NNNN.md`.
- Identifier allocation: the next unused four-digit number in the register.
- A CR that changes an accepted ADR links both the CR and the new ADR.
- A CR that changes a WP updates that WP's status history in the same change.

## 7. References

- `.docs/sow/SOW.md` §12
- `.docs/sow/WP-00-GOVERNANCE.md` §3.9
- `.docs/CONSTITUTION.md` §5
- `.docs/technical-specification/APPENDIX-D-DELIVERY.md` §D.3
