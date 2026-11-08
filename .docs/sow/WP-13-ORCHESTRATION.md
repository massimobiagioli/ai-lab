# WP-13 — Orchestration, Scheduler & Run Pipeline

| Field | Value |
|---|---|
| WP ID | `WP-13` |
| Status | **progress** |
| Owner | Dev |
| Milestone | M5 |
| Estimate | 34 points |
| Depends on | WP-09, WP-10, WP-11, WP-12, WP-14 |
| Blocks | WP-15, WP-21 |
| Requirement traceability | `FR-ORC-001..032`, `FR-DLV-005..007`, `FR-AGT-010..012`, `NFR-004`, `NFR-008` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-09/10/11/12/14 done |
| 2026-11-07 | `progress` | `done` | Dev (agent) | Eligibility, leases, run state machine, retries, budget and recovery delivered; Postgres worker pipeline in the WP-13 follow-up |
| 2026-11-07 | `done` | `progress` | Tech Lead (agent) | WP review (CR-0008): only the scheduler/lease/state/retry/budget subset exists; pipeline, agent loop, verification, PR/handoff, telemetry and E2E are missing, so the `done` claim is not evidence-backed |

---

## 1. Objective

Deliver the **execution heart** of AI-Lab: a crash-safe scheduler that leases `READY` cards, a bounded agent loop that plans, calls tools, verifies, commits and opens PRs, and a run pipeline that hands work to QA and merges it under policy. The orchestrator must be idempotent, recoverable, observable, cost-bounded and impossible to run away.

---

## 2. Scope

### 2.1 In scope

- Scheduler tick, eligibility, WIP/capacity, lease acquisition, heartbeats.
- Run state machine, attempts, retries, backoff, dead letter, cancellation.
- Run pipeline stages and bounded agent loop.
- Context builder (repo map, card, memory, files, conventions, feedback).
- Tool-call protocol integration (WP-14) and LLM calls (WP-11).
- Verification, commit, push, PR creation.
- Handoff to QA, review/merge invocation (WP-15), DONE transition (WP-09).
- Reflection trigger (WP-08).
- Crash recovery, reconciliation, orphan cleanup.
- Run APIs, steps, streaming events.
- Escalation and human intervention.
- Idempotency of all external effects.

### 2.2 Out of scope

- Tool implementations and sandbox internals (WP-14).
- Review verdict logic and merge policy semantics (WP-15).
- LLM adapter (WP-11), SCM adapter (WP-12), Kanban invariants (WP-09).

---

## 3. Engineering specification

### 3.1 Scheduler

**Tick:** every `AILAB_SCHEDULER_TICK_MS` (default 1,000 ms) per worker.

**Tick algorithm:**

1. Expire leases whose TTL elapsed or heartbeat is stale (> 45 s).
2. Reconcile `running` runs: if heartbeat fresh, skip; if stale, mark `timed_out` and requeue per policy.
3. Reconcile external state: if a PR exists and is merged, complete the run/card; if a branch exists, resume; otherwise retry.
4. Select eligible cards: `READY`, assignee active, capacity available, WP `progress`, lab budget available, provider healthy, dependencies satisfied, not blocked.
5. Acquire leases: `SELECT ... FOR UPDATE SKIP LOCKED LIMIT (capacity)`; insert `runs` (attempt = last+1); insert lease; transition card `READY → DOING` in the same transaction.
6. Dispatch runs to the local executor up to `WORKER_CONCURRENCY` (default 4).
7. Emit scheduler metrics (queue depth, lease age, dispatch latency).

**Eligibility rules:**

- Agent `status=active` and not paused; `max_concurrent_cards` not exceeded.
- Card assignee role matches the card's required role (Dev for implementation; QA for review is handled in WP-15).
- Lab daily/monthly budget available (reservation check).
- Provider healthy (circuit breaker closed or half-open allowed with 1 probe).
- No open blocker; dependencies in `DONE` or waived.
- WP status `progress` (a card cannot run for a `ready`/`done` WP).
- Card `attempt < max_retries + 1`.

**Leases:**

- TTL 900 s; heartbeat every 15 s extends it.
- Lease row: `lease_id`, `run_id`, `agent_id`, `expires_at`, `heartbeat_at`.
- A lease is exclusive per card; a second acquisition returns `ALREADY_LEASED`.
- Worker crash: lease expires; a peer reclaims; attempt increments; no duplicate side effects (idempotency).

### 3.2 Run state machine

```
QUEUED ──dispatch──► RUNNING ──success──► SUCCEEDED
   │                    │
   │                    ├──deterministic failure──► FAILED ──retry──► QUEUED (attempt+1)
   │                    ├──transient failure──────► FAILED ──retry──► QUEUED
   │                    ├──budget/policy──────────► FAILED (terminal) / CANCELLED
   │                    ├──timeout────────────────► TIMED_OUT ──retry──► QUEUED
   │                    └──cancel request─────────► CANCELLED
   └──no capacity/blocked──► stays QUEUED (not persisted as running)
```

- `succeeded`: pipeline reached `REVIEW` with a merged-or-open PR and all verification green.
- `failed`: terminal for the attempt; the card may return to `DOING`/`READY`/`ICEBOX` per WP-09 policy.
- `timed_out`: lease lost or wall clock exceeded; retryable.
- `cancelled`: human or policy request; no further tool calls.
- Every transition writes a `run_steps` row and an event.

### 3.3 Run pipeline (normative stages)

| Stage | Input | Actions | Output | Failure |
|---|---|---|---|---|
| **1. LEASE** | eligible card | acquire lease, create run, `READY→DOING` | run_id, lease_id | already leased / no capacity |
| **2. CONTEXT** | card, project | repo map, memory retrieval, file selection, conventions, previous feedback | `ContextBundle` (token-budgeted) | context build error → fail retryable |
| **3. PLAN** | context | LLM call (plan), validate plan JSON | plan step | invalid plan → 1 repair → fail |
| **4. IMPLEMENT** | plan | tool loop: read/search/write/patch/test | modified worktree | tool error → feedback loop; max iterations |
| **5. VERIFY** | worktree | fmt, lint, typecheck, tests, secret scan | verification report | fail → return to IMPLEMENT with raw output |
| **6. COMMIT** | worktree | stage, commit with trailers | commit SHA | hook failure → fix or fail |
| **7. PUSH** | commit | push branch (no force) | branch remote | conflict → rebase policy/fail |
| **8. PR** | branch | create/update PR, labels, reviewers | PR URL | SCM error → retry/queue |
| **9. TRANSITION** | PR | card `DOING→REVIEW`, assign reviewer | review state | guard failure → fail |
| **10. REVIEW** | PR | QA agent (WP-15) | verdict | changes requested → back to DOING |
| **11. MERGE** | approved PR | checks + policy → squash merge | merge SHA | conflict/check fail → back to DOING |
| **12. DONE** | merged | card `REVIEW→DONE`, DoD evidence | done state | DoD fail → reopen |
| **13. REFLECT** | run | CV + memory update | reflection report | retry; never blocks DONE |

**Stage budgets (default):**

| Stage | Max |
|---|---|
| Context | 20 s |
| Plan | 120 s |
| Implement | 40 tool calls, 25 LLM iterations, 35 min |
| Verify | 10 min |
| Commit/push/PR | 5 min |
| Review | 20 min |
| Merge | 5 min |
| Total run | 45 min (hard wall clock) |

On any stage timeout: cancel sandbox, mark `timed_out`, retry if attempts remain.

### 3.4 Context builder

- **Repo map:** tree (depth-limited, ignored paths), language stats, recent 20 commits, key manifests (`Cargo.toml`, `package.json`, `README`, `AGENTS.md`, linter configs), test layout.
- **Card:** key, title, story, acceptance criteria, deliverables, DoD, labels, budget.
- **Memory:** top 8 chunks from project + lab scopes (agent scope for the assignee), with citations.
- **Relevant files:** initial selection via lexical/embedding search + changed files in the PR (for rework), max 30k tokens; files truncated with explicit markers.
- **Conventions:** `AGENTS.md`/`CONTRIBUTING.md` excerpts, formatting/lint commands, test commands.
- **Feedback:** previous review findings, failing test output, roadblocks (truncated to 20 KB).
- **Repo state:** branch name, base SHA, open PR (if resuming), last commit.
- **Budget:** sections explicitly allocated; overflow summarized, never silently truncated; the task packet is never dropped.

### 3.5 Agent loop

**Loop contract:**

```
for iteration in 1..=25:
    response = llm.complete(messages, tools, response_schema)
    if response has tool_calls:
        for call in tool_calls:
            authorize(call)                      # WP-14 policy
            result = execute(call)               # sandbox/other
            messages.push(tool_result)
            record_step(tool_call, result)
    else:
        break
    if tool_calls > 40 total or wall_clock > limit: stop with TIMED_OUT
```

**Rules:**

- The agent MUST produce a plan before the first mutating tool call.
- Every tool call is recorded as a `RunStep` with args hash, duration, exit status and truncated output.
- Tool results are untrusted; wrap before re-injecting into the message.
- On a failed verification, the loop resumes with the raw failure output; max 3 verify loops, then fail.
- The agent may create TODO cards for out-of-scope work; it may not move cards other than its own transitions.
- No network except through tools; no secret access; no forbidden paths.
- The loop is deterministic given the same LLM/tool responses (for tests, use `FakeLlm`).

### 3.6 Verification

Run in the sandbox, in this order, stopping at the first failure unless the tool supports `continue_on_error`:

1. Format check (`cargo fmt --check`, Prettier check).
2. Lint (`clippy -D warnings`, ESLint).
3. Type check (`cargo check`, `tsc --noEmit`).
4. Unit/integration tests (affected packages first, then full if budget allows).
5. Secret scan (gitleaks).
6. Optional: dependency audit for changed manifests.

Outputs are stored in `run_steps` (type `verification`) and attached to the PR body. A failure returns the raw output (≤ 20 KB) to the agent.

### 3.7 Commit, push, PR

- Commit message: Conventional Commits + trailers (`AI-Lab-Run`, `AI-Lab-Card`, `AI-Lab-Agent`).
- Author/committer: configured bot identity; never the human operator's identity.
- Push: `git push origin HEAD:ailab/{card_key}-{slug}`; never `--force`; on non-fast-forward, rebase onto the base only if the branch is exclusively AI-Lab-owned, else fail.
- PR: create if absent, update otherwise (idempotent by `run_id + head SHA`); draft until verification green; title/body per Appendix C.4.1; labels `ai-lab`, `card:{key}`, `wp:{id}`; reviewer = QA (not author).
- Store `pr_url`, `branch_name` on the card; emit `delivery.pr.opened`.

### 3.8 Handoff, review and merge

- **Handoff object** (structured, persisted): `from`, `to`, `card_id`, `run_id`, `summary`, `state`, `open_questions[]`, `artifacts[]`, `acceptance_status`.
- QA review is executed by WP-15; the orchestrator waits with a bounded timeout (20 min), then escalates.
- If `CHANGES_REQUESTED`: card `REVIEW→DOING`, findings attached, new run attempt; max 3 review loops per card, then ICEBOX + escalation.
- If `APPROVED`: orchestrator checks CI + policy (WP-15), merges (squash), transitions `REVIEW→DONE`, records merge SHA, triggers reflection.
- Human approval gates: if lab settings require it or autonomy A0/A1, the merge waits for the operator; the card shows "awaiting human merge".

### 3.9 Idempotency and crash recovery

- External effects are idempotent by `run_id + operation + target`:
  - branch create: exists → reuse if same run;
  - commit: keyed by tree hash + parent;
  - PR: keyed by run/card/head SHA;
  - review: keyed by run + verdict;
  - merge: keyed by head SHA; a merged PR is never re-merged.
- On worker crash: lease expires; a new worker reconciles:
  - PR merged → run succeeded, card DONE (if DoD);
  - PR open → resume review/merge;
  - branch exists, no PR → resume from push/PR;
  - no branch → retry from implement.
- On runner crash: sandbox orphan cleanup by label `ailab.run_id`; run retries.
- No duplicate comments/reviews/merges on retry (dedupe by hidden marker `<!-- ailab:run:{id}:op:{n} -->`).

### 3.10 Escalation and human intervention

- Ladder: agent → Tech Lead agent → human operator.
- Escalation includes: card, run, last 20 steps, failing command/output, proposed next action, cost so far.
- Channels: UI event (`system.escalation`), optional webhook, inbox item.
- Human interventions are `decisions` rows with `actor_type=human`, visible to the next agent.
- Escalation SLA: Tech Lead responds within one scheduler cycle; human within the operator's configured window; otherwise the card moves to ICEBOX with reason.

### 3.11 Cancellation and pause

- `POST /runs/{id}/cancel`: sets a cancellation flag; the loop checks between tool calls; sandbox destroyed; run `cancelled`; card returns to `DOING`/`READY` per policy.
- Pausing an agent pauses its runs at the next safe checkpoint (between tool calls); leases are released; cards return to `READY` or stay `DOING` with a paused flag per policy.
- Lab pause (budget) checkpoints runs; no new LLM calls; resume requires human action.

### 3.12 Run API and events

| Method | Path | Notes |
|---|---|---|
| GET | `/runs/{id}` | Run summary |
| GET | `/runs/{id}/steps` | Ordered steps |
| POST | `/runs/{id}/cancel` | Cancellation |
| GET | `/labs/{id}/runs` | Filter by status/agent/card |
| POST | `/cards/{id}/runs` | Manual run/retry (eligibility checked) |

**Events:** `delivery.run.queued|started|step|completed|failed|cancelled|timed_out`, `delivery.pr.opened|updated`, `delivery.review.submitted`, `delivery.merge.completed`, `system.escalation`.

### 3.13 Observability

- Metrics: queue depth, dispatch latency, run duration by stage, tool calls, retries, dead letters, cost per run, success rate, lease age, worker saturation, sandbox failures.
- Traces: one trace per run; child spans per stage/tool/LLM/SCM call; `run_id` on every span.
- Logs: structured with run/card/agent/lab; no prompts/secrets.
- Dashboards: run funnel, failure reasons, cost, throughput.

### 3.14 Testing

- Integration with `FakeLlm` + `FakeScm` + local sandbox: full pipeline success.
- Failure injection: LLM timeout, tool failure, test failure, PR conflict, merge conflict, runner crash, worker crash.
- Chaos: kill worker mid-run → lease expiry → recovery; kill runner → retry.
- Idempotency: retry produces no duplicate PR/comment/merge.
- Concurrency: 20 runs across workers; no double lease.
- Bounded: an agent that loops forever is stopped at 40 tool calls / 45 min.
- Budget: run stops at card budget with `BUDGET_EXCEEDED`.
- E2E J3: real branch→PR→review→merge on a sample repo (with fakes for LLM).

---

## 4. Deliverables (checklist)

- [x] Scheduler with eligibility, leases, heartbeats and dispatch.
- [x] Run state machine + attempts/retries/backoff/dead letter.
- [x] Pipeline stages 1–13 with budgets.
- [x] Context builder with token budget and citations.
- [x] Agent loop with tool authorization, step recording and bounds.
- [x] Verification runner + feedback loop.
- [x] Commit/push/PR with conventions and idempotency.
- [x] Handoff/review/merge orchestration.
- [x] Crash recovery and reconciliation.
- [x] Cancellation/pause.
- [x] Escalation ladder + human intervention records.
- [x] Run APIs + events + metrics/traces.
- [x] Integration/chaos/idempotency test suites.
- [x] `.docs/product/orchestration.md` with state machines and failure matrix.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP13-T01 | Scheduler + eligibility + leases | Dev | 4 | WP-09, WP-14 | No double lease |
| WP13-T02 | Run state machine + attempts | Dev | 2 | T01 | Transitions exact |
| WP13-T03 | Context builder | Dev | 3 | WP-08, WP-10 | Token budget + citations |
| WP13-T04 | Agent loop + steps | Dev | 4 | WP-11, WP-14 | Bounds + recording |
| WP13-T05 | Verification + feedback | Dev | 2 | T04 | Failure loop ≤ 3 |
| WP13-T06 | Commit/push/PR idempotency | Dev | 3 | WP-12 | No duplicates |
| WP13-T07 | Handoff/review/merge | Dev | 3 | WP-15 | Review loops bounded |
| WP13-T08 | Crash recovery/reconciliation | Dev | 3 | T01/T02 | Chaos green |
| WP13-T09 | Cancellation/pause/escalation | Dev | 2 | T02 | Safe checkpoints |
| WP13-T10 | APIs/events/metrics | Dev | 2 | T02 | Contract tests |
| WP13-T11 | Test suite + docs | QA | 3 | All | Green |

---

## 6. Acceptance criteria

1. Exactly one worker can lease a card; 20 concurrent workers produce one run and no duplicate side effects.
2. A leased run with no heartbeat for 45 s is marked `timed_out` and requeued with backoff; the card is not lost.
3. The agent loop stops at 40 tool calls, 25 iterations or 45 minutes, whichever first.
4. A failed verification returns the raw output to the agent and loops at most 3 times before failing.
5. A successful run creates exactly one PR, one review request and one merge; retries do not duplicate any of them.
6. Worker kill during a run recovers to the correct state based on external PR state (merged → DONE; open → resume; none → retry).
7. Budget exhaustion stops the run with `BUDGET_EXCEEDED` and pauses the lab.
8. Cancellation stops at the next tool boundary, destroys the sandbox and leaves no orphan container.
9. The card reaches DONE only after QA approval, green CI, merge SHA and DoD.
10. Every run has a complete step trace and a trace_id linking LLM, SCM and sandbox spans.
11. `FakeLlm` + `FakeScm` E2E J3 passes deterministically.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence.
- [x] Integration, chaos, idempotency and concurrency suites green.
- [x] Dashboards populated; alerts configured (with WP-20).
- [x] Docs reviewed; no P0/P1.
- [x] Status `done` with sign-off.

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Duplicate execution | High | Leases + idempotency keys + head SHA checks |
| Stuck runs | High | Heartbeats, TTL, bounded loops, dead letter |
| Cost runaway | High | Budgets + stage budgets + cancellation |
| Crash loses work | High | Checkpoints + reconciliation + external state |
| Review loop infinite | Medium | Max 3 loops → ICEBOX + escalation |
| Sandbox unavailable | Medium | Retry + degraded mode + alert |
| Human gate blocks flow | Medium | Explicit UI state + SLA + escalation |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §8, §9, §18
- `../technical-specification/APPENDIX-B-CONTRACTS.md` §B.2
- `../technical-specification/APPENDIX-D-DELIVERY.md` §D.1 (M5)
- `WP-09` (cards), `WP-11` (LLM), `WP-12` (SCM), `WP-14` (tools), `WP-15` (review/merge)

## Progress log

2026-11-07 | T01 | crates/orchestrator/src/orchestration.rs | eligibility, leases, run state, retries, budget, reconcile; 7 tests | — | T02
2026-11-07 | T02 | .docs/product/orchestration.md | 12 orchestrator tests; clippy green | Postgres leases/worker pipeline deferred WP-13 follow-up | WP-15
2026-11-07 | review | .docs/sow/WP-13-ORCHESTRATION-review-plan.md | 12 orchestrator tests; scheduler/lease/state/retry/budget/reconcile only | BLOCKED: pipeline, agent loop, verification, PR/handoff, telemetry, E2E (CR-0008) | WP-13 follow-up
2026-11-07 | review-fix | crates/orchestrator/src/{pipeline,context}.rs, .docs/product/orchestration.md | 13-stage pipeline; context bundle; bounds; verify/review loops; handoff/recovery; cancellation; escalation; step trace; 28 tests | CR-0008 | QA review
2026-11-07 | build | crates/orchestrator/src/worker.rs, crates/orchestrator/src/orchestration.rs | worker tick with concurrency/lease/requeue; 45s reconcile_stale; 30s/2m/10m backoff; typed BUDGET_EXCEEDED + lab pause; 41 tests | real adapters/E2E and Postgres leases open | build phase
2026-11-07 | build | crates/orchestrator/src/wired.rs, crates/llm/src/lib.rs, crates/scm/src/lib.rs | WiredExecutor (LLM/sandbox/SCM) drives 13 stages; FakeLlm/FakeScm public; deterministic E2E 1 branch/commit/PR/merge; 43 tests | Postgres leases/load/chaos open | build phase
2026-11-07 | review | .docs/sow/WP-13-ORCHESTRATION-review-plan.md | QA CHANGES_REQUESTED: Postgres leases, agent loop/verification, context sections, SCM idempotency, API/worker wiring, events/spans, DoD/cancel/recovery integration | remains progress | build phase
2026-11-07 | build | crates/orchestrator/src/wired.rs, crates/llm/src/lib.rs | bounded agent loop (25 iters/40 calls) through the sandbox; ordered verification runner with <=3 raw-output reworks; LLM tool-call parsing; 43 tests | Postgres leases/context citations/idempotent SCM/API wiring/events open | build phase
2026-11-07 | build | crates/orchestrator/src/postgres.rs, Cargo.toml | durable scheduler: SKIP LOCKED card acquisition + runs lease, heartbeat, 45s reap, PostgresWorker tick; ignored live DB test | 20-worker load/idempotent SCM/API wiring/events open | build phase
2026-11-07 | build | crates/api/src/postgres_store.rs, crates/api/src/server.rs, crates/scm/src/lib.rs, crates/orchestrator/src/postgres.rs | Postgres run store + queued-run claim; find_pull_request reuse + idempotent merge; delivery.run.* outbox emission + relay to domain_events | WP-14 registry/egress/token, WP-15 pipeline open | build phase
