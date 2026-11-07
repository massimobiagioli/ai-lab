# WP-{NN} — {Name}

| Field | Value |
|---|---|
| WP ID | `WP-{NN}` |
| Status | **{ready|progress|done}** |
| Owner | {Tech Lead|Analyst|Dev|QA} |
| Milestone | {M0..M8} |
| Estimate | {points} points |
| Depends on | {WP-XX, ... or —} |
| Blocks | {WP-XX, ... or —} |
| Requirement traceability | {FR-*, NFR-*, DOC-*, ...} |
| Version | {x.y.z} |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| {YYYY-MM-DD} | — | `ready` | {actor} | {reason} |

---

## 1. Objective

{One outcome-oriented paragraph.}

## 2. Scope

### 2.1 In scope

- {item}

### 2.2 Out of scope

- {explicit exclusion}

## 3. Engineering specification

### 3.1 {Area}

{Architecture, interfaces, data model, algorithms, configuration, security, failure handling, performance. No TBDs.}

## 4. Deliverables (checklist)

- [ ] {artifact path — what done looks like}
- [ ] {artifact path — what done looks like}

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| {WPXX}-T01 | {description} | {role} | {est} | — | {testable acceptance} |

## 6. Acceptance criteria

1. {Testable criterion.}
2. {Testable criterion.}

## 7. WP Definition of Done

- [ ] All deliverables checked and linked to evidence.
- [ ] `scripts/sow-lint.sh` green on the full SOW.
- [ ] Relevant quality gates green.
- [ ] No open P0/P1; no unresolved blocker.
- [ ] Documentation and ADRs updated.
- [ ] Status updated with sign-off row.

## 8. Risks and mitigations

| Risk | Impact | Likelihood | Mitigation | Owner |
|---|---|---|---|---|
| {risk} | {High|Medium|Low} | {High|Medium|Low} | {mitigation} | {owner} |

## 9. References

- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` {section}
- `.docs/technical-specification/APPENDIX-{X}.md` {section}
- `.docs/sow/SOW.md` {section}
