# Review Closure Checklist — 2026-11-09

| Field | Value |
|---|---|
| Purpose | Track every remaining item required to close the full work-package review defined by `.prompts/review-all-prompt.md`. |
| Baseline | `main` at `12ef046`; open PRs #2 (WP-13B re-baseline), #3 (WP-17), #4 (CI suspension), #5 (WP-18). |
| Rule | An item is checked only with evidence (test, command output, benchmark, scan, ADR, CR, PR or commit SHA). |
| Status | `progress` |

This document is the authoritative checklist for the remaining review work. It does not replace the per-WP review plans; it sequences them and records the gaps that are still open.

---

## 0. Operating rules

- [ ] Process one WP at a time (WIP = 1); one review branch and one PR per WP.
- [ ] Use `review/wp-XX-<slug>` branch names (or rename the current feature branches) and merge via a PR merge commit; never push directly to `main`.
- [ ] Every WP has a review plan using the `.prompts/review-all-prompt.md` §11 template, with QA verdict and merge SHA recorded.
- [ ] No fake gates: do not mark a deliverable, criterion or test done without evidence; do not disable or weaken a gate.
- [ ] Record every deferral with owner, card, due date and expiry; no unexplained deferral remains.
- [ ] Keep `.docs/sow/SOW.md`, `AGENTS.md`, `.docs/adr/registry.md` and `.docs/product/README.md` indexes current.

---

## 1. Baseline, branches and governance

- [ ] Merge PR **#4** (`chore/suspend-actions`) first so `main` stops scheduling automatic workflows.
- [ ] Rebase `chore/wp-13b-rebaseline`, `feat/wp-17-frontend-platform` and `feat/wp-18-virtual-office` on the new `main` after #4 merges.
- [ ] Merge PR **#2** (`WP-13B` creation and CR-0012 re-baseline) after review.
- [ ] Merge PR **#3** (`WP-17`) after QA approves the platform increment.
- [ ] Merge PR **#5** (`WP-18`) after QA approves the office increment.
- [ ] Record the residual risk from CR-0012: WP-13 and WP-16 are `done` by a Tech Lead re-baseline, **not** by QA `APPROVED`; their residual hardening is owned by WP-13B.
- [ ] Review WP-19..WP-23, which are `done` without review plans or QA; reopen any WP whose evidence is missing (CR required).
- [ ] Re-enable automatic CI at the end by reverting PR #4; confirm the 16 required checks are green on `main`.
- [ ] Confirm `scripts/sow-lint.sh` agrees on every index/status pair after each merge.

---

## 2. WP closure matrix

| WP | Current | Review plan | QA | Merge SHA | Remaining work |
|---|---|---|---|---|---|
| WP-00 | `done` | present | APPROVED | recorded | Verify the merge commit and that the review branch is deleted. |
| WP-01 | `done` | present | APPROVED | recorded | Verify merge and indexes. |
| WP-02 | `done` | present | APPROVED | recorded | Verify merge and tooling. |
| WP-03 | `done` | present | APPROVED | recorded | Verify merge and Compose. |
| WP-04 | `done` | present | APPROVED | recorded | Verify merge and migrations. |
| WP-05 | `done` | present | APPROVED | recorded | Verify merge and secrets. |
| WP-06 | `done` | present | APPROVED | recorded | Verify merge and domain tests. |
| WP-07 | `done` | present | APPROVED | recorded | Verify merge. |
| WP-08 | `done` | present | APPROVED | recorded | Verify merge; memory provider affects WP-13B. |
| WP-09 | `done` | present | APPROVED | recorded | Verify merge and transition matrix. |
| WP-10 | `done` | present | APPROVED | recorded | Verify merge. |
| WP-11 | `done` | present | APPROVED | recorded | Verify merge; real adapter in WP-13B. |
| WP-12 | `done` | present | APPROVED | recorded | Verify merge; adapter idempotency in WP-13B. |
| WP-13 | `done` | present | `CHANGES_REQUESTED` | pending | Closed by CR-0012 with residuals moved to WP-13B; residual risk recorded. |
| WP-14 | `done` | present | APPROVED | recorded | Verify merge. |
| WP-15 | `done` | present | APPROVED | recorded | Verify merge. |
| WP-16 | `done` | present | `CHANGES_REQUESTED` | pending | Closed by CR-0012 with residuals moved to WP-13B; residual risk recorded. |
| WP-17 | `progress` | **missing** | none | pending (#3) | Finish platform items (§3). |
| WP-18 | `progress` | **missing** | none | pending (#5) | Finish office items (§4). |
| WP-19 | `done` (overclaimed) | **missing** | none | none | Reopen and build the product UI (§5). |
| WP-20 | `done` (overclaimed) | **missing** | none | none | Verify/implement observability (§6). |
| WP-21 | `done` (overclaimed) | **missing** | none | none | Build the test suites and gates (§7). |
| WP-22 | `done` (overclaimed) | **missing** | none | none | Verify/implement release evidence (§8). |
| WP-23 | `done` (overclaimed) | **missing** | none | none | Verify/implement maintenance jobs (§9). |
| WP-13B | `ready` | **missing** | none | none | Deliver the backend residuals last (§10). |

---

## 3. WP-17 — Frontend platform completion

- [ ] Write `.docs/sow/WP-17-FRONTEND-PLATFORM-review-plan.md` with the §11 template.
- [ ] Add the licensed self-hosted pixel bitmap font and the high-contrast theme variant.
- [ ] Add the keyboard drag-and-drop directive and the full keyboard-only navigation audit.
- [ ] Install and configure Playwright, axe-core and the visual snapshot harness; add the smoke journeys.
- [ ] Record Lighthouse ≥ 90 desktop, LCP ≤ 2.5 s and TTI ≤ 3.5 s evidence; verify the CSP and absence of inline scripts.
- [ ] Implement the offline read-only snapshot.
- [ ] Verify the 22 primitives cover default/hover/focus/disabled/loading/error states (the gallery is at `/design`).
- [ ] Spawn the independent QA subagent; fix findings; record `APPROVED`.
- [ ] Merge PR #3 with a merge commit and record the SHA.

---

## 4. WP-18 — Virtual office completion

- [ ] Write `.docs/sow/WP-18-VIRTUAL-OFFICE-review-plan.md`.
- [ ] Deliver the sprite atlas, manifest, license metadata and the 384×216 integer-scaled renderer (the host currently uses the host resolution).
- [ ] Add y-sort, occupancy, camera focus and `?agent`/`?panel` deep links.
- [ ] Add ambient life and sound controls (mute persisted).
- [ ] Add FPS/heap telemetry and evidence for ≥ 58 FPS desktop / ≥ 30 FPS low-end.
- [ ] Add the visual, FPS and a11y suites (axe on the office route; screen-reader fallback verified).
- [ ] Spawn QA; fix findings; record `APPROVED`.
- [ ] Merge PR #5 with a merge commit and record the SHA.

---

## 5. WP-19 — Product UI surfaces (not started)

- [ ] Reopen WP-19 with a CR (current `done` has no implementation or review plan).
- [ ] Write `.docs/sow/WP-19-PRODUCT-UI-review-plan.md`.
- [ ] Implement the dashboard integration, Kanban board (columns, drag/keyboard move, optimistic rollback on 409/422), inventory (agents, projects, providers), SOW graph, memory browser, settings and onboarding.
- [ ] Consume the real edge endpoints; replace placeholders and demo data; document any read-model fallback.
- [ ] Add typed signal stores with `idle|loading|ready|error`, empty states and no `any`.
- [ ] Add component, E2E and a11y tests for every surface.
- [ ] QA, merge commit, SHA.
- [ ] Note: board/inventory/SOW/memory endpoints are currently 501 stubs; WP-13B T07 must land before the real-data assertions.

---

## 6. WP-20 — Observability and SRE

- [ ] Write `.docs/sow/WP-20-OBSERVABILITY-SRE-review-plan.md`.
- [ ] Verify or implement structured logs, metrics (`/metrics`), OTel traces, SLOs, dashboards, alert rules and on-call runbooks.
- [ ] Verify backup/restore drill evidence and the disaster-recovery runbook.
- [ ] Confirm the OTel pipeline depends on WP-13B T03 and record the dependency.
- [ ] QA, merge commit, SHA.

---

## 7. WP-21 — Testing and quality engineering

- [ ] Write `.docs/sow/WP-21-TESTING-review-plan.md`.
- [ ] Install and wire Playwright (E2E journeys J1..J5), axe (a11y), visual snapshots, k6 load, chaos, security scanning and fuzz targets.
- [ ] Produce coverage reports and enforce backend ≥ 80%, domain ≥ 90%, frontend ≥ 80%, new code ≥ 80%.
- [ ] Wire every suite into CI and record the exact commands.
- [ ] QA, merge commit, SHA.
- [ ] Do not keep the `done` status on documentation-only evidence.

---

## 8. WP-22 — Documentation, release and handover

- [ ] Write `.docs/sow/WP-22-DOCS-RELEASE-review-plan.md`.
- [ ] Verify the docs manifest, acceptance report, demo script and handover pack.
- [ ] Generate SBOMs (syft), sign and verify images (cosign), publish the release 1.0 changelog and the rollback procedure.
- [ ] QA, merge commit, SHA.

---

## 9. WP-23 — Maintenance and continuous evolution

- [ ] Write `.docs/sow/WP-23-MAINTENANCE-review-plan.md`.
- [ ] Verify dependency updates/Renovate, CV reflection, memory hygiene, stale-card and health jobs, and their schedules/alerts.
- [ ] QA, merge commit, SHA.

---

## 10. WP-13B — Backend hardening follow-up (last)

- [ ] Write `.docs/sow/WP-13B-FOLLOW-UP-review-plan.md`.
- [ ] Deliver WP13B-T01 open/branch resume.
- [ ] Deliver WP13B-T02 real context provider (repo map, memory top-8 with citations, files, conventions, feedback).
- [ ] Deliver WP13B-T03 OTel spans and run/queue/lease metrics.
- [ ] Deliver WP13B-T04 Kanban-service DONE transition with DoD.
- [ ] Deliver WP13B-T05 real DeepSeek/GitHub adapters behind the HTTP transport.
- [ ] Deliver WP13B-T06 Redis token bucket (fail closed) and pub/sub fan-out.
- [ ] Deliver WP13B-T07 full Appendix B REST surface and schemathesis.
- [ ] Deliver WP13B-T08 WebSocket per-connection buffer and ephemeral-first drop.
- [ ] Deliver WP13B-T09 generated OpenAPI, `openapi-diff` and the full DTO schema set.
- [ ] Deliver WP13B-T10 readiness component checks.
- [ ] Deliver WP13B-T11 tests and docs.
- [ ] QA, merge commit, SHA.

---

## 11. Cross-cutting contract and documentation consistency

- [ ] Align the event catalog with Appendix B.2.3 (`delivery.run.cancelled`/`timed_out` are currently catalogued but absent from the appendix table); correct the lower source via CR if needed.
- [ ] Verify `openapi.json` is generated from the source of truth, has no router drift and passes `openapi-diff`/schemathesis (WP-13B T07/T09).
- [ ] Verify `asyncapi.json` has the 36 channels and the topic form (`lab:`/`card:`/`run:`/`system`) expected by Appendix B.2.1.
- [ ] Remove documentation drift: `orchestration.md` duplicate `## 12` heading and stale §11, `realtime.md` envelope/backoff text, `eventing.md` counts, `api/README.md` canonical-source text.
- [ ] Reconcile WP-19..WP-23 status/DoD checkboxes with reality.
- [ ] Ensure every new ADR/CR is in the registry/register and every new doc is indexed in `AGENTS.md` or `.docs/product/README.md`.

---

## 12. Quality gates to run before closure

- [ ] `scripts/harness-lint.sh`
- [ ] `scripts/sow-lint.sh`
- [ ] `make check`
- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace` (or `cargo nextest run --workspace`)
- [ ] `.tools/check-doc-links.sh`
- [ ] `.tools/check-crate-dependencies.sh`
- [ ] `.tools/check-architecture-fitness.sh`
- [ ] `.tools/test-migrations.sh`, `.tools/test-constraints.sh`, `.tools/test-analytics-views.sh`
- [ ] `.tools/test-supply-chain.sh`
- [ ] `make compose-validate`, `.tools/test-compose-isolation.sh`
- [ ] `make openapi`, `make asyncapi`, `.tools/check-openapi-drift.py`
- [ ] `schemathesis run .docs/product/api/openapi.json` (install/configure)
- [ ] `npm --prefix web run lint`, `npm --prefix web test`, `npm --prefix web run build`
- [ ] `npx --prefix web playwright test` (install/configure)
- [ ] `make load` with k6 (install/configure)
- [ ] `gitleaks detect --source . --no-git`
- [ ] `cargo deny check`
- [ ] `npm --prefix web audit --audit-level=high`
- [ ] `trivy` on changed images, SBOM generation and image signature verification
- [ ] Coverage reports meeting the thresholds

---

## 13. Final closure steps

- [ ] Every WP is QA `APPROVED` (or has a recorded, owned, expiring deferral) and merged with a merge commit SHA.
- [ ] `SOW.md` shows every WP `done`; no `progress`/`ready` remains except an explicitly deferred WP-13B.
- [ ] No open P0/P1 and no unresolved blocker.
- [ ] Re-enable automatic CI (revert PR #4) and confirm the 16 required checks are green.
- [ ] Publish the final review report: WP, review plan, defects fixed, tests, QA verdict, merge SHA.
- [ ] Delete all local and remote review branches.
- [ ] Close the loop by recording the final `main` SHA and the re-enabled CI evidence.

---

## 14. Known risks and blockers to keep visible

| Risk/blocker | Impact | Owner | Next action |
|---|---|---|---|
| WP-13/WP-16 closed by CR-0012 without QA `APPROVED` | Residual risk in the delivery gate | Tech Lead | Complete WP-13B and its QA; record the residual in the final report |
| WP-19..WP-23 marked `done` without review plans/QA | False completion | Tech Lead | Reopen via CRs; review and rebuild |
| CI suspended (PR #4) | Branch protection checks will not report | Tech Lead | Merge #4, then re-enable at the end and confirm checks |
| Stacked PRs #2 → #3 → #5 | Merge order coupling | Tech Lead | Merge in order after QA; rebase on each merge |
| No Playwright/k6/schemathesis/trivy/SBOM tooling | Gates cannot run | Dev/QA | Install/configure and wire into CI |
| Frontend office/product surfaces incomplete | WP-17/18/19 acceptance cannot pass | Dev | Finish the items in §3–§5 |
| Event catalog and docs drift | Spec/contract inconsistency | Tech Lead | Align lower sources with a CR |
