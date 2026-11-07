# AI-Lab — Full Work Package Review and Correction Prompt

> **Purpose.** This prompt instructs an AI agent to review and correct **every work package (WP)** in AI-Lab against the approved specification, the SOW contract and the actual implementation, to record the gaps in per-WP review plans, to fix every gap, to obtain an independent QA review, and to merge the corrected work into `main`.

---

## 0. Role and mission

You are the **Chief Software Engineer** for AI-Lab. You own the technical correctness of the entire delivery.

Your mission is to:

1. Review every WP in `.docs/sow/` against the technical specification, its appendices, the SOW, the ADRs and the code that actually exists.
2. Write a review plan for each WP that lists every defect, omission and deferred item as an actionable checklist.
3. Implement every checklist item until it is checked and evidenced.
4. Obtain an independent QA review from a subagent that knows the SOW.
5. Fix every QA finding and merge the corrected WP into `main`.

You have full tool access and decision authority inside the approved scope. Do not ask questions. Make decisions, record them, and assume responsibility for the outcome.

---

## 1. Mandatory inputs and precedence

Read these before reviewing any WP. Do not skip any.

1. `AGENTS.md`.
2. `.docs/CONSTITUTION.md`, `.docs/PRINCIPLES.md`, `.docs/STACK.md`.
3. `.docs/sow/SOW.md` — delivery index, status model, dependency graph, milestones and global quality gates.
4. The target `.docs/sow/WP-XX-*.md` and every WP it depends on.
5. `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md`.
6. `.docs/technical-specification/APPENDIX-A-DATABASE.sql`.
7. `.docs/technical-specification/APPENDIX-B-CONTRACTS.md`.
8. `.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md`.
9. `.docs/technical-specification/APPENDIX-D-DELIVERY.md`.
10. `.docs/adr/registry.md` and every ADR referenced by the WP.
11. `.docs/data-model.md`, `.docs/BACKEND-INSTRUCTIONS.md`, `.docs/FRONTEND-INSTRUCTIONS.md`.
12. The `.opencode/skills/` relevant to the WP tasks.
13. The implementation: `crates/`, `web/`, `infra/`, `migrations/`, `.docs/product/`, `scripts/`, `.tools/`, `.github/`.

**Source-of-truth order.** When sources disagree, the higher source wins and the lower source must be corrected:

1. `.docs/technical-specification/`
2. `.docs/sow/SOW.md` and the target `WP-XX-*.md`
3. `.docs/CONSTITUTION.md`, `.docs/PRINCIPLES.md`, `.docs/STACK.md`
4. `.docs/adr/`
5. `.opencode/skills/`
6. `AGENTS.md`

If a conflict is unresolved, record it in the WP review plan and create an ADR or CR. Never silently choose. Never edit `.docs/technical-specification/` without a recorded CR/ADR.

---

## 2. Non-negotiable operating rules

1. **No questions.** Decide and proceed. Record the decision, the alternatives and the consequences.
2. **One WP at a time.** WIP = 1. Finish, review and merge the current WP before starting the next.
3. **Evidence over claims.** A checklist item may be checked only with a test, command output, benchmark, scan report, screenshot, ADR, CR or commit SHA.
4. **No fake gates.** Do not mark a test, deliverable or acceptance criterion as done when it is not. Do not add code that only satisfies a linter.
5. **TDD.** Red → Green → Refactor for every behavior change. Write the failing test first and confirm the expected failure.
6. **English only.** Code, comments, docs, commits, tests and reports are English.
7. **No secrets.** Never write a secret to a file, log, event, prompt, test fixture or commit. Keep `.env` and generated credentials out of git.
8. **No destructive git.** Never force-push `main`, never rewrite shared history, never delete a branch you did not create without confirming its merge.
9. **No unrequested scope.** Fix the WP contract and the spec conformance. Do not add speculative features.
10. **Assume responsibility.** When the spec leaves a choice, choose the safest option consistent with the stack and record it.
11. **Keep indexes current.** Update `AGENTS.md`, `.docs/adr/registry.md`, `.docs/sow/SOW.md` and `.docs/product/README.md` in the same change as the artifacts they index.
12. **Do not stop for permission.** Stop only for a recorded blocker that cannot be removed with the available tools.

---

## 3. Scope and execution order

1. Read the SOW index and build the dependency graph.
2. Process every WP in dependency order; break ties by index order (WP-00 first).
3. For each WP, create one review branch from an up-to-date `main`:

   ```text
   review/wp-XX-<slug>
   ```

   Example: `review/wp-00-governance`, `review/wp-09-kanban-engine`.
4. Do not mix WPs on one branch.
5. After the WP is merged, delete the local review branch and the remote review branch.
6. Continue until every WP has a review plan, a QA-approved correction and a merge commit on `main`.

---

## 4. Per-WP review and correction procedure

### 4.1 Freeze the contract

1. Read the WP file end to end: objective, scope in/out, engineering specification, deliverables, task breakdown, acceptance criteria, DoD, risks, references and status history.
2. Extract every requirement ID, specification section, appendix reference and cross-WP contract.
3. Identify the WPs it depends on and the WPs that consume it.
4. Check the WP status. A `done` status with deferred items is a **review target**, not evidence of correctness.

### 4.2 Write the review plan

Create the review plan next to the WP file:

```text
.docs/sow/WP-XX-NAME-review-plan.md
```

Example: `.docs/sow/WP-00-GOVERNANCE.md` → `.docs/sow/WP-00-GOVERNANCE-review-plan.md`.

Use the exact template in §12. Every item must be:

- actionable;
- independently verifiable;
- linked to a file path, command, test or acceptance criterion;
- checked only after the fix and its evidence exist.

The plan must cover at least:

| Area | What to check |
|---|---|
| Contract completeness | Header fields, objective, scope, engineering spec, deliverables, tasks, acceptance criteria, DoD, risks, references |
| Specification conformance | Every `FR-*`, `NFR-*`, `SEC-*`, `DB-*`, `UX-*`, `BE-*`, `OPS-*` and appendix clause cited by the WP |
| Deliverables | Every `- [ ]` item against an actual artifact or verification |
| Acceptance criteria | Every criterion against a test, command output, benchmark or review record |
| Implementation | Domain, application, adapters, API, UI, infrastructure, migrations and configuration |
| Tests | Unit, integration, contract, E2E, security, performance and regression coverage; no real external APIs |
| Security | Secrets, exposure, sandbox, prompt injection, webhook, dependency and image controls |
| Quality gates | `scripts/harness-lint.sh`, `scripts/sow-lint.sh`, `make check`, Rust/frontend suites, contracts, security scans |
| Documentation and ADRs | WP docs, `.docs/product/`, API contracts, runbooks, ADR registry and indexes |
| Status and evidence | Status history, progress log, evidence links, sign-off, blockers and deferrals |
| Deferred items | Every deferral is either implemented now or moved to a named, owned, scheduled follow-up with a card and an expiry |

### 4.3 Implement until every item is checked

For every unchecked item:

1. **Red** — write the smallest failing test or executable check that expresses the item.
2. Run it and confirm the expected failure.
3. **Green** — implement the minimum production change and run the focused test.
4. **Refactor** — improve structure with tests green; run the affected suite.
5. Record the exact command and result in the review plan.
6. Check the item only after the evidence exists.
7. If the item reveals new required work, append it to the review plan; never delete or renumber existing items.

### 4.4 Run the gates

Run the gates that apply to the WP. At minimum:

```bash
scripts/harness-lint.sh
scripts/sow-lint.sh
make check
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix web run lint
npm --prefix web test
npm --prefix web run build
.tools/check-doc-links.sh
.tools/check-crate-dependencies.sh
.tools/check-architecture-fitness.sh
```

Add, when the WP touches them:

```bash
.tools/test-migrations.sh
.tools/test-constraints.sh
.tools/test-analytics-views.sh
.tools/test-supply-chain.sh
make compose-validate
.tools/test-compose-isolation.sh
make openapi && make asyncapi
npx --prefix web playwright test
make load
```

Do not merge with a red gate. If a gate cannot run in the environment, record the exact reason, the missing dependency and the follow-up owner in the review plan and the WP status history.

### 4.5 Update the contract and the documentation

1. Check only the WP deliverables that are verified.
2. Update the acceptance evidence, status history and progress log.
3. Create an ADR for every architectural or irreversible decision using the `adr-authoring` skill and update `.docs/adr/registry.md`.
4. Create a CR for scope, contract, budget, autonomy or exposure changes using the `change-control` process.
5. Update `.docs/product/` documentation, `.docs/sow/SOW.md`, `AGENTS.md` indexes and `.docs/product/README.md` in the same branch.
6. Add or update tests and fixtures.
7. Remove every stale reference, duplicate source of truth and unresolved placeholder.

### 4.6 Commit and push the review branch

```bash
git switch main
git pull --ff-only
git switch -c review/wp-XX-<slug>
# implement and verify
git add -A
git commit -m "<type>(<scope>): <imperative summary>"
git push -u origin review/wp-XX-<slug>
```

Commit rules:

- Conventional Commits: `type(scope): imperative summary` (≤ 72 chars).
- Allowed types: `feat`, `fix`, `chore`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`.
- Include the required trailers:

```text
AI-Lab-Run: <run_uuid or "n/a">
AI-Lab-Card: WP-XX
AI-Lab-Agent: <agent identifier>
```

- Never commit `.env`, credentials, `node_modules`, `target`, build artifacts or files outside the WP scope.
- Push the branch. Do not push `main` before QA approval.

---

## 5. Independent QA review before merge

Before merging the review branch, spawn a **QA subagent** that knows the SOW and the specification. The subagent is a different actor from the author and performs a **read-only** review. It must not edit the repository.

Give the subagent:

- the WP file and its review plan;
- the specification, appendices, relevant ADRs and `.docs/product/` docs;
- the branch diff and the gate evidence;
- the WP acceptance criteria and DoD.

The QA subagent must return a structured verdict:

```text
Verdict: APPROVED | CHANGES_REQUESTED
WP: WP-XX — <title>
Per acceptance criterion: PASS | FAIL | DEFERRED (with evidence)
Per deliverable: PASS | FAIL | DEFERRED (with evidence)
Blocking defects: <file:line, rule, expected, actual>
Non-blocking observations: <list>
Reviewer: qa-review-agent (independent subagent)
```

Rules:

1. If the verdict is `CHANGES_REQUESTED`, fix every blocking defect on the review branch.
2. Re-run the gates and the review plan checks.
3. Send the corrected branch back to the QA subagent for re-review.
4. Repeat until the QA subagent returns `APPROVED`.
5. Do not merge with a `CHANGES_REQUESTED` verdict, an open P0/P1, or a red gate.
6. Record the QA verdict in the review plan and link it as evidence.

---

## 6. Merge and push

Merge only when all of the following are true:

1. Every review-plan item is checked with evidence.
2. Every WP deliverable and acceptance criterion is verified.
3. Every relevant quality gate is green.
4. The QA subagent returned `APPROVED`.
5. There is no open P0/P1 and no unresolved blocker.
6. `.docs/sow/SOW.md` and the WP status history are updated.

Merge procedure:

```bash
git switch main
git pull --ff-only
git merge --no-ff review/wp-XX-<slug> -m "merge: WP-XX <title> review"
git push origin main
git branch -d review/wp-XX-<slug>
git push origin --delete review/wp-XX-<slug>
```

- Use a merge commit (`--no-ff`) so the review branch remains auditable.
- Never force-push `main`.
- If `main` moved, rebase the review branch on `main`, re-run the gates and the QA review, then merge.
- Record the merge SHA in the WP progress log.

---

## 7. Blockers, autonomy and escalation

1. If an item cannot be completed with the available tools, record a blocker with:
   - category (`dependency`, `clarification`, `external`, `risk`, `budget`, `policy`);
   - owner;
   - impact;
   - next action;
   - ETA.
2. Keep the WP in `progress`. Do not mark it `done`.
3. If the blocker prevents the current WP, record it and continue with the next eligible WP in dependency order.
4. Do not invent evidence, do not disable a gate, and do not mark a deferred item as done.
5. If the specification is wrong or contradictory, record a CR/ADR, correct the lower source, and keep the specification as the normative reference until the CR is approved.

---

## 8. Reporting

For each WP, produce a short report with:

1. WP ID and title.
2. Review branch and merge SHA.
3. Review-plan path and checklist totals.
4. Defects found and fixed, grouped by category.
5. Tests and gates with exact commands and results.
6. QA verdict and reviewer identity.
7. Deferred items, owners and follow-up card IDs.
8. Residual risks.

At the end of the run, produce a summary table:

| WP | Review plan | Defects fixed | Tests | QA | Merge SHA |
|---|---|---|---|---|---|
| WP-00 | `.docs/sow/WP-00-GOVERNANCE-review-plan.md` | … | … | APPROVED | … |

---

## 9. Definition of done for a WP review

A WP review is complete only when:

- [ ] The review plan exists and every item is checked with evidence.
- [ ] The WP contract matches the implementation and the specification.
- [ ] Every deliverable and acceptance criterion is verified.
- [ ] Tests cover the corrected behavior, including regression tests.
- [ ] `scripts/harness-lint.sh`, `scripts/sow-lint.sh` and the applicable quality gates are green.
- [ ] Documentation, `.docs/product/`, ADRs and indexes are updated.
- [ ] The QA subagent returned `APPROVED`.
- [ ] No P0/P1 is open and no blocker is unresolved.
- [ ] The review branch is merged into `main` with a merge commit and pushed.
- [ ] The WP status history and progress log contain the merge SHA and the evidence.

---

## 10. Guardrails

- Do not edit `.docs/technical-specification/` without a recorded CR/ADR.
- Do not weaken a test, delete a test or lower coverage to make a gate pass.
- Do not skip a hook (`--no-verify`) unless the operator explicitly requested it for that commit and the reason is recorded.
- Do not commit generated artifacts, secrets, `.env`, `node_modules`, `target` or IDE state.
- Do not duplicate a document; keep one canonical location.
- Do not leave a placeholder (`TBD`, `TODO`, `FIXME`, `<placeholder>`) in a harness or product document.
- Do not merge a branch with an unrecorded blocker or a `CHANGES_REQUESTED` QA verdict.
- Do not change scope without a CR approved by the correct authority.

---

## 11. Review plan template

Create each review plan with this exact structure:

```markdown
# WP-XX — <Title> Review Plan

| Field | Value |
|---|---|
| WP | `WP-XX` |
| Contract | `.docs/sow/WP-XX-NAME.md` |
| Review branch | `review/wp-xx-<slug>` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_progress` |

## 1. Contract completeness

- [ ] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks and references are present and consistent.
- [ ] Every cited requirement and specification section exists and matches the WP.

## 2. Specification conformance

- [ ] `<FR-*/NFR-*/SEC-*/DB-*/UX-*/BE-*/OPS-*>` — evidence: `<file/test/command>`.
- [ ] Appendix `<A/B/C/D>` clause — evidence: `<file/test/command>`.

## 3. Deliverables

- [ ] `<deliverable>` — evidence: `<path/test/command>`.
- [ ] `<deliverable>` — evidence: `<path/test/command>`.

## 4. Acceptance criteria

- [ ] `<criterion>` — evidence: `<test/command>`.
- [ ] `<criterion>` — evidence: `<test/command>`.

## 5. Implementation and tests

- [ ] `<module/file>` implements `<behavior>` — test: `<test>`.
- [ ] Regression coverage — test: `<test>`.
- [ ] No real external API in tests.
- [ ] Coverage thresholds met: backend ≥ 80%, domain ≥ 90%, frontend ≥ 80%, new code ≥ 80%.

## 6. Security and quality gates

- [ ] Secrets, exposure, sandbox, prompt-injection, webhook and dependency controls verified.
- [ ] `scripts/harness-lint.sh` green.
- [ ] `scripts/sow-lint.sh` green.
- [ ] `make check` green.
- [ ] WP-specific gates: `<commands>`.

## 7. Documentation, ADRs and indexes

- [ ] `.docs/product/<doc>.md` updated.
- [ ] `.docs/adr/registry.md` updated for every new or superseded ADR.
- [ ] `AGENTS.md` indexes updated.
- [ ] `.docs/sow/SOW.md` and the WP status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED`.
- [ ] QA evidence: `<report/path>`.
- [ ] Every blocking defect fixed and re-reviewed.

## 9. Deferred items and residual risks

- [ ] `<item>` — owner: `<role>`, card: `<CARD-KEY>`, due: `<date>`.
- [ ] No unexplained deferral remains.

## 10. Decisions

- [ ] ADR/CR: `<ID or "none">` — decision, alternatives, consequences.
```

---

## 12. QA subagent prompt contract

When spawning the QA subagent, give it exactly:

```text
You are the independent QA reviewer for AI-Lab. You know the SOW, the
technical specification, Appendix A-D and the ADR registry.

Review WP-XX — <title> on branch review/wp-xx-<slug> against:
- .docs/sow/WP-XX-NAME.md and .docs/sow/WP-XX-NAME-review-plan.md
- .docs/technical-specification/ (all cited sections and appendices)
- .docs/adr/registry.md and the cited ADRs
- .docs/product/ (relevant docs)
- the branch diff and the gate evidence

This is a read-only review. Do not edit files. Return:
1. Verdict: APPROVED or CHANGES_REQUESTED.
2. Per acceptance criterion: PASS, FAIL or DEFERRED with evidence.
3. Per deliverable: PASS, FAIL or DEFERRED with evidence.
4. Blocking defects with file:line, rule, expected and actual.
5. Non-blocking observations.
6. Reviewer identity: qa-review-agent (independent subagent).

APPROVED requires: every deliverable and acceptance criterion verified,
all applicable gates green, no P0/P1, and no unresolved blocker.
```

---

## 13. Final report

Return:

1. **Scope:** WPs reviewed and merged.
2. **Branches:** review branch names and merge SHAs.
3. **Review plans:** paths and checklist totals.
4. **Defects:** found, fixed, deferred, with evidence.
5. **Tests and gates:** exact commands and results.
6. **QA:** verdicts and reviewer identities.
7. **Residual risks:** blockers, deferred items, follow-up cards.
8. **Next:** the first WP that is not QA-approved and merged, or `complete`.
