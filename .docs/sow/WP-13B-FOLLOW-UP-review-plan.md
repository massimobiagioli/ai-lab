# WP-13B — Backend Hardening Follow-up Review Plan

| Field | Value |
|---|---|
| WP | `WP-13B` |
| Contract | `.docs/sow/WP-13B-FOLLOW-UP.md` |
| Review branches | `review/p17-wp13b-resume` … `review/p24-wp13b-tests` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_progress` |

## 1. Contract completeness

- [x] Header, objective, scope, resume/context/OTel/DoD/adapters/Redis/REST/WS/OpenAPI specification, deliverables, tasks, acceptance criteria, DoD, risks and references present.
- [x] CR-0012 opened WP-13B for the WP-13/WP-16 residuals; the review starts at P17.

## 2. T01 — Resume open/branch recovery

- [x] `ExternalPrState` maps to `RecoveryAction::{Done, ResumeReview, ResumePush, Retry}` with a unit test (`crates/orchestrator/src/pipeline.rs`).
- [x] `WiredExecutor::recover` reads the pull request and merge SHA from the SCM (`crates/orchestrator/src/wired.rs`).
- [x] A worker killed after merge completes from the external merge SHA (`PostgresWorker::tick`).
- [ ] The worker executes `ResumeReview` and `ResumePush`: today both fall through to a full requeue, so the tail re-runs implement/verify instead of resuming at review or push.
- [ ] Head-SHA validation: the recovered PR head is not compared with the persisted branch head before merging.
- [ ] Tests for the open-PR and branch-only resume paths.

## 3. T02–T11

- [ ] T02 real context provider.
- [ ] T03 OTel spans and metrics.
- [ ] T04 server-enforced DoD.
- [ ] T05 real LLM/SCM adapters.
- [ ] T06 Redis limiter and fan-out.
- [ ] T07 full REST surface and schemathesis.
- [ ] T08 WebSocket buffer/ephemeral.
- [ ] T09 generated OpenAPI and breaking gate.
- [ ] T10 readiness components.
- [ ] T11 tests and docs.

## 4. Security and operations

- [ ] Review with the `security-first` skill when T05–T08 land.

## 5. Review and QA independence

- [x] The reviewer is not the author.
- [ ] QA verdict after T01–T11.

## 6. Gates

- [ ] Full gates at T01 completion and at the WP close.

## 7. Verdict

WP-13B is `progress`. The recovery mapping and external-state probe exist, but the resume tail and head-SHA guard are the first open T01 items; T02–T11 remain.

## Progress log

2026-11-09 | P17 | crates/orchestrator/src/{pipeline,wired,postgres}.rs | T01 audit: mapping + recover probe present; resume tail and head-SHA guard missing | T01 open | T01 tail
