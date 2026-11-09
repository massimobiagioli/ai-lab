# AI-Lab — End-to-End BDD Checklist (480 scenarios)

| Field | Value |
|---|---|
| Document | `.docs/e2e/BDD.md` |
| Version | 1.3.0 |
| Status | Ready for execution |
| Owner | QA |
| Audience | QA, Dev, Tech Lead, Release |
| Last QA review | 2026-11-09 (E2E-201 – E2E-240 gap closure; E2E-241 – E2E-360 Amiga-style UI; E2E-361 – E2E-480 Inventory and Lab management UI) |
| Related | `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md`, `APPENDIX-B-CONTRACTS.md`, `APPENDIX-C-AGENTS-KANBAN.md`, `.docs/product/` |

This document is the QA-owned, executable specification of the AI-Lab end-to-end behaviour and the checklist used to record execution. It is ordered from the simplest platform assertions to the most complex autonomous-delivery journeys, and it covers every use case of the product: inventory, labs and agents, Kanban, idea intake, SOW and planning, execution, QA and merge, memory, realtime, virtual office, product UI, security, observability, maintenance and recovery.

The scenarios are written in Gherkin and are intended to be implemented as Playwright API + browser tests. They are grouped by feature area; within each area the simplest behaviour comes first. Identifiers `E2E-001` … `E2E-480` are immutable: a scenario may be rewritten but never renumbered or deleted without a change request. New gaps found in a later review are appended with the next free identifier.

Section 22 is the dedicated Amiga-style UI certification: 120 scenarios that certify the palette, pixel grid, integer scaling, sprites, truthful state, interactions, accessibility, sound, performance, determinism and visual regression of the office. Section 23 is the Inventory and Lab management certification: 120 scenarios that certify the management surface for clarity, hierarchy, consistency, responsiveness, usability, accessibility and performance to a Jobs-quality bar. Neither section is a coverage afterthought; both are release-blocking contracts.

---

## How to use this checklist

Each scenario title starts with a checkbox marker. The marker is part of the checklist workflow, not part of the Gherkin semantics.

| Marker | Meaning | Required evidence |
|---|---|---|
| `[ ]` | Not executed in the current cycle | None |
| `[x]` | Executed and passed | CI run URL, artifact (trace/screenshot/report), date, tester |
| `[!]` | Executed and failed | Defect card id, failing assertion, log excerpt |
| `[-]` | Blocked or quarantined | Blocking reason, owner, quarantine card and review date |

Rules:

1. A scenario is marked `[x]` only after a green deterministic run in CI; a manual pass is allowed only for a scenario explicitly tagged `@manual`, and it still records evidence.
2. A failure must be recorded as `[!]` and linked to a defect card before the checklist is updated; a flaky scenario is quarantined with `[-]` within 24 hours and fixed or deleted within 5 days (`TEST-004`).
3. A regression reopens the scenario back to `[!]`; markers are never silently carried across releases.
4. The marker is reset to `[ ]` at the start of each release candidate unless the scenario is tagged `@regression-passed`.
5. Every checklist update records the release candidate, the environment, the commit SHA and the tester in the evidence record.

### Progress tracker

Update the counters as scenarios are executed. The `[ ]` column is the initial state for every area.

| # | Area | Scenarios | `[ ]` | `[x]` | `[!]` | `[-]` |
|---|---|---|---|---|---|---|
| 1 | Platform health and bootstrap | 6 | 6 | 0 | 0 | 0 |
| 2 | Providers and models | 8 | 8 | 0 | 0 | 0 |
| 3 | Labs, agents and CV | 12 | 12 | 0 | 0 | 0 |
| 4 | Projects and SCM credentials | 8 | 8 | 0 | 0 | 0 |
| 5 | Kanban board and card lifecycle | 22 | 22 | 0 | 0 | 0 |
| 6 | Kanban transitions and invariants | 20 | 20 | 0 | 0 | 0 |
| 7 | Idea intake | 6 | 6 | 0 | 0 | 0 |
| 8 | SOW lifecycle | 12 | 12 | 0 | 0 | 0 |
| 9 | Plan, WBS and materialization | 8 | 8 | 0 | 0 | 0 |
| 10 | Runs and orchestration | 16 | 16 | 0 | 0 | 0 |
| 11 | QA review and merge policy | 8 | 8 | 0 | 0 | 0 |
| 12 | Memory and knowledge | 12 | 12 | 0 | 0 | 0 |
| 13 | Realtime and events | 6 | 6 | 0 | 0 | 0 |
| 14 | Virtual office | 12 | 12 | 0 | 0 | 0 |
| 15 | Dashboard, UI and accessibility | 8 | 8 | 0 | 0 | 0 |
| 16 | Security and exposure | 8 | 8 | 0 | 0 | 0 |
| 17 | API contracts and observability | 8 | 8 | 0 | 0 | 0 |
| 18 | Maintenance, budget and metrics | 8 | 8 | 0 | 0 | 0 |
| 19 | Golden journeys J1–J4 | 6 | 6 | 0 | 0 | 0 |
| 20 | Reliability and recovery | 6 | 6 | 0 | 0 | 0 |
| 21 | QA gap-closure scenarios | 40 | 40 | 0 | 0 | 0 |
| 22 | Amiga-style UI certification | 120 | 120 | 0 | 0 | 0 |
| 23 | Inventory and Lab management UI certification | 120 | 120 | 0 | 0 | 0 |
| | **Total** | **480** | **480** | **0** | **0** | **0** |

---

## Conventions

- **Execution layer.** API-only scenarios drive `/api/v1` with the typed client; browser scenarios drive the Angular UI; journey scenarios combine both. Assertions target observable behaviour, never implementation internals.
- **Deterministic data.** All runs start from `make up && make seed-demo` (one lab, four agents, one project, one SOW, one plan, 12 cards, 30 events) or from an API fixture that creates the same shape. No test calls the real DeepSeek or GitHub APIs (`TEST-003`); `AILAB_DEV_FAKES=1` provides the fake LLM/SCM harness.
- **Truthful assertions.** A scenario may assert on persisted state, returned payloads, emitted events, UI projection, audit rows or metrics — never on sleeps. Time-dependent steps use the injectable `Clock`.
- **Traceability.** Every scenario carries tags that map to the normative requirement IDs it verifies (`@fr-*`, `@ux-*`, `@sec-*`, `@ops-*`, `@ac-*`) and a priority tag (`@p0`, `@p1`, `@p2`).
- **Definition of done for a scenario.** It is "covered" only when it exists as an automated test, passes deterministically in CI, and its evidence is linked from the owning work package.

### Specification coverage matrix

The QA cross-check maps every normative area of the capitolato to the scenarios that verify it. `Primary` is the original 200-scenario set; `Additional` covers the gap-closure set (section 21), the Amiga-style UI certification (section 22) and the Inventory and Lab management certification (section 23).

| Specification area | Primary | Additional |
|---|---|---|
| §3 Journeys J1–J4 | 19 | 21.7 |
| §4 Containers and bootstrap | 1, 20 | 21.6, 21.7 |
| §6.1 Inventory (provider, model, agent, project) | 2, 3, 4 | 21.1, 21.4, 23 |
| §6.2 Lab and autonomy | 3 | 21.1, 21.4, 23 |
| §6.3 Memory and knowledge | 12 | 21.4 |
| §6.4 Kanban and invariants | 5, 6 | 21.4 |
| §6.5 Delivery (idea, SOW, plan, run) | 7, 8, 9, 10, 11 | 21.2, 21.3 |
| §7 Agent operating model | 3, 11 | 21.1, 21.2 |
| §8 Orchestration | 10 | 21.3 |
| §9 LLM platform | 2, 18 | 21.2 |
| §10 GitHub integration | 4 | 21.1 |
| §11 Virtual office | 14 | 21.5, 22 |
| §11.2–§11.7 Amiga visual language, interaction and responsiveness | 14 | 22 |
| §11.7–§11.9 Management product surfaces (dashboard, inventory, SOW) | 15 | 22, 23 |
| §12 Frontend architecture and design system | 15 | 21.5, 22, 23 |
| §13 Backend architecture | 17 | 21.3 |
| §14 Data architecture | 5, 10, 12 | 21.4 |
| §15 API and realtime contracts | 13, 17 | 21.4, 21.5 |
| §16 Security and privacy | 16 | 21.6 |
| §17 Performance and capacity | 18 | 21.7 |
| §18 Reliability and disaster recovery | 20 | 21.7 |
| §19 Testing strategy | 21 (meta) | 21.7 |
| §20 DevOps and configuration | 1 | 21.6, 21.7 |
| §23 Global acceptance criteria | 19 | 21.7 |
| Appendices B and C (contracts, agents, Kanban) | 4, 5, 6, 13, 17 | 21.1 – 21.5 |

> Scenarios are end-to-end behaviour checks. Gates that are not scenario-shaped — mutation score, fuzzing, coverage thresholds, schema drift, dependency and image scanning — remain enforced by `cargo nextest`, `cargo-fuzz`, `.tools/check-coverage.sh`, `make openapi`, `cargo deny`, `npm audit`, gitleaks and Trivy as listed in `.docs/product/testing.md` and `.docs/product/quality-gates.md`; they are out of scope for this checklist and are not counted as missing scenarios.

### QA gap analysis (2026-11-09)

The cross-check against §6–§20 and §23 found the following gaps in the original 200 scenarios. Each gap is closed by a scenario in section 21.

| # | Gap found | Specification | Closed by |
|---|---|---|---|
| 1 | Provider status lifecycle (disable, enable, degrade) and model field validation were not exercised end to end. | §6.1, FR-INV-001, FR-INV-010 | E2E-201, E2E-202 |
| 2 | Per-provider and per-lab token buckets, SCM credential verification and classic-PAT warning were missing. | §6.1, FR-INV-004, FR-GH-001 | E2E-203, E2E-204 |
| 3 | Agent role validation, the full agent state machine, effective autonomy and self-modification guard were missing. | §6.1, §7.3, §7.4 | E2E-205, E2E-206, E2E-207 |
| 4 | Prompt determinism, character cap/override, role tool allowlists, routing justification and structured-output repair were missing. | §7.2, §9.1–§9.2, C.2.8 | E2E-208 – E2E-211 |
| 5 | Token-budget allocation, streaming coalescing, retry/circuit-breaker, untrusted wrapping and prompt persistence were missing. | §9.2 | E2E-212, E2E-213 |
| 6 | Commit trailers, context build, loop bounds, plan-before-mutation, verification gate and run trace fields were missing. | §8.2, §7.4, FR-AGT-023 | E2E-214 – E2E-218 |
| 7 | Structured handoff, escalation payload, human decisions and decision-to-memory promotion were missing. | §8.3, FR-DLV-007 | E2E-219 |
| 8 | Reflection report, memory tiers, Redis working memory, top-k bounds and PII/secret scrubbing were missing. | §6.3, §7.5, FR-MEM-003/004/005 | E2E-220 – E2E-223 |
| 9 | DoR dependencies, card splitting, work-only-via-cards, card field persistence, timezone rendering and project archive/transfer were missing. | §6.4, §6.1.4, UX-022, FE-013 | E2E-224, E2E-225 |
| 10 | Office camera, keyboard/hover, 16-agent layout, asset licensing, sound, background-tab sync, bundle/offline budgets and realtime dedupe were missing. | §11, §12, §15 | E2E-226 – E2E-230 |
| 11 | TLS profile, shell allowlist, short-lived repo token, audit export, data deletion, retention, fail-fast config, Compose hardening, performance/soak, rolling upgrade and acceptance evidence were missing. | §16, §17, §18, §20, §23 | E2E-231 – E2E-240 |

Section 22 (E2E-241 – E2E-360) is not a gap closure. It is a new, dedicated Amiga-style UI certification suite that raises the office from "covered" to "certified": palette-exact, pixel-perfect, state-truthful, accessible, measurable and release-blocking. Section 23 (E2E-361 – E2E-480) is the same idea applied to the Inventory and Lab management surface: clear, calm, consistent, responsive, accessible and fast, with a Jobs-quality visual and usability bar.

### Coverage summary

| # | Area | Scenarios | IDs |
|---|---|---|---|
| 1 | Platform health and bootstrap | 6 | E2E-001 – E2E-006 |
| 2 | Providers and models | 8 | E2E-007 – E2E-014 |
| 3 | Labs, agents and CV | 12 | E2E-015 – E2E-026 |
| 4 | Projects and SCM credentials | 8 | E2E-027 – E2E-034 |
| 5 | Kanban board and card lifecycle | 22 | E2E-035 – E2E-056 |
| 6 | Kanban transitions and invariants | 20 | E2E-057 – E2E-076 |
| 7 | Idea intake | 6 | E2E-077 – E2E-082 |
| 8 | SOW lifecycle | 12 | E2E-083 – E2E-094 |
| 9 | Plan, WBS and materialization | 8 | E2E-095 – E2E-102 |
| 10 | Runs and orchestration | 16 | E2E-103 – E2E-118 |
| 11 | QA review and merge policy | 8 | E2E-119 – E2E-126 |
| 12 | Memory and knowledge | 12 | E2E-127 – E2E-138 |
| 13 | Realtime and events | 6 | E2E-139 – E2E-144 |
| 14 | Virtual office | 12 | E2E-145 – E2E-156 |
| 15 | Dashboard, UI and accessibility | 8 | E2E-157 – E2E-164 |
| 16 | Security and exposure | 8 | E2E-165 – E2E-172 |
| 17 | API contracts and observability | 8 | E2E-173 – E2E-180 |
| 18 | Maintenance, budget and metrics | 8 | E2E-181 – E2E-188 |
| 19 | Golden journeys J1–J4 | 6 | E2E-189 – E2E-194 |
| 20 | Reliability and recovery | 6 | E2E-195 – E2E-200 |
| 21 | QA gap-closure scenarios | 40 | E2E-201 – E2E-240 |
| 22 | Amiga-style UI certification | 120 | E2E-241 – E2E-360 |
| 23 | Inventory and Lab management UI certification | 120 | E2E-361 – E2E-480 |
| | **Total** | **480** | |

---

## 1. Platform Health and Bootstrap (E2E-001 – E2E-006)

```gherkin
@health @p0 @fr-orc-001
Feature: Platform health and bootstrap

  Scenario: [ ] E2E-001 Liveness succeeds without dependencies
    Given the api process is running and the database and Redis are stopped
    When I call GET /api/v1/health/live
    Then the response status is 200
    And the body reports the process as alive
    And no database or Redis query is required to serve the request

  Scenario: [ ] E2E-002 Readiness reports all dependencies available
    Given the database, Redis and migrations are ready
    When I call GET /api/v1/health/ready
    Then the response status is 200
    And the body reports database, Redis and migrations as ready

  Scenario: [ ] E2E-003 Readiness fails when Redis is unreachable
    Given Redis is unreachable and the database is healthy
    When I call GET /api/v1/health/ready
    Then the response status is 503
    And the body names Redis as the unavailable dependency
    And the error uses the NOT_READY code

  Scenario: [ ] E2E-004 Provider and SCM health endpoints report component status
    Given a DeepSeek provider and a GitHub credential are configured
    When I call GET /api/v1/health/providers and GET /api/v1/health/scm
    Then both responses have status 200
    And each body reports the component status and the last probe time

  Scenario: [ ] E2E-005 Cold bootstrap reaches the running stack
    Given a clean clone and a valid environment file
    When I run make up following only the README
    Then the web UI responds at http://127.0.0.1:8080
    And every core service reports healthy
    And the elapsed time is at most 10 minutes
    And the result is recorded as evidence for AC-001

  Scenario: [ ] E2E-006 Readiness fails while migrations are pending
    Given the database is reachable but a migration is not applied
    When I call GET /api/v1/health/ready
    Then the response status is 503
    And the body reports pending migrations
```

---

## 2. Providers and Models (E2E-007 – E2E-014)

```gherkin
@inventory @p0 @fr-inv-001
Feature: LLM providers and model catalog

  Scenario: [ ] E2E-007 Create a DeepSeek provider with a write-only key
    Given I am on the providers inventory page
    When I submit name "DeepSeek", kind "deepseek", an HTTPS base URL and a valid API key
    Then the response status is 201
    And the body returns the provider id and only the last four key characters
    And the full API key is never present in the response

  Scenario: [ ] E2E-008 Reject a provider whose base URL is not HTTPS
    Given I have the provider create form open
    When I submit a base URL with the http scheme
    Then the response status is 422
    And the error code is VALIDATION_FAILED
    And the violation field is base_url

  Scenario: [ ] E2E-009 A stored API key is never readable
    Given a provider was created with a known API key
    When I call GET /api/v1/providers
    And I call GET /api/v1/providers/{provider_id}
    Then no response contains the plaintext key
    And each provider row exposes only api_key_last4

  Scenario: [ ] E2E-010 Provider verification probes the endpoint and updates status
    Given a provider whose fake endpoint returns a one-token completion
    When I call POST /api/v1/providers/{provider_id}/verify
    Then the response status is 200 and the result is healthy
    And the provider status becomes active
    And the last probe timestamp is recorded

  Scenario: [ ] E2E-011 Key rotation increments the envelope key version
    Given an active provider with a known last4
    When I call POST /api/v1/providers/{provider_id}/rotate-key with a new key
    Then the response status is 200
    And key_version is incremented
    And the new last4 reflects the new key
    And the old ciphertext can no longer be decrypted with the active version

  Scenario: [ ] E2E-012 The model catalog seeds exactly two models
    Given the database is freshly migrated
    When I call GET /api/v1/models
    Then the response contains exactly deepseek-flash and deepseek-v4-pro
    And deepseek-flash is marked as the default model
    And deepseek-v4-pro is marked as the escalation model

  Scenario: [ ] E2E-013 Model pricing configuration drives recorded cost
    Given a model row with fixed input and output prices
    When an LLM request records tokens_in and tokens_out
    Then the ledger cost equals (tokens_in * input_price + tokens_out * output_price) / 1_000_000 micro-USD
    And the cost is returned as an integer with no floating-point drift

  Scenario: [ ] E2E-014 Provider rate limiting queues work instead of failing it
    Given the provider rate limit is exhausted for the current minute
    When a run requests a completion
    Then the request is queued rather than rejected
    And the job is dispatched after the rate-limit window resets
    And a 429 with Retry-After is returned only to interactive callers that opt out of queueing
```

---

## 3. Labs, Agents and CV (E2E-015 – E2E-026)

```gherkin
@lab @inventory @p0 @fr-lab-001
Feature: Labs, agents and CV

  Scenario: [ ] E2E-015 A lab cannot be created without a Tech Lead
    Given I submit a new lab with name, slug and settings but no tech_lead agent
    When I call POST /api/v1/labs
    Then the response status is 422
    And the error code is TECH_LEAD_REQUIRED

  Scenario: [ ] E2E-016 Create a lab with exactly one Tech Lead
    Given I submit a lab with one veteran_lead agent in the tech_lead role
    When I call POST /api/v1/labs
    Then the response status is 201
    And the lab is returned with autonomy level A2 by default
    And the Tech Lead is active and belongs to the lab

  Scenario: [ ] E2E-017 A second active Tech Lead is rejected
    Given a lab already has one active tech_lead
    When I add another agent in the tech_lead role
    Then the response status is 422
    And the error code is TECH_LEAD_EXISTS

  Scenario: [ ] E2E-018 Add Analyst, Dev and QA agents with valid characters
    Given an existing lab
    When I add systems_analyst, scrappy_dev and pixel_artisan agents with unique names
    Then each response status is 201
    And each agent has one character from the fixed roster
    And the lab team now contains exactly one Tech Lead and the three added agents

  Scenario: [ ] E2E-019 An unknown character id is rejected
    Given the agent creation wizard
    When I submit character_id "unknown_hero"
    Then the response status is 422
    And the error code is VALIDATION_FAILED and the field is character_id

  Scenario: [ ] E2E-020 Agent description enforces the 120 to 600 character range
    Given the agent creation form
    When I submit a description of 119 characters
    Then the response status is 422
    When I submit a description of 601 characters
    Then the response status is 422
    When I submit a description of 120 characters or 600 characters
    Then the response status is 201 for each valid boundary

  Scenario: [ ] E2E-021 Pause and resume an agent
    Given an active agent that is idle
    When I call POST /api/v1/agents/{agent_id}/pause
    Then the agent status becomes paused
    When I call POST /api/v1/agents/{agent_id}/resume
    Then the agent status returns to idle
    And both changes emit agent.status_changed events

  Scenario: [ ] E2E-022 A retired agent keeps history and receives no new cards
    Given an active agent that has completed at least one card
    When I call DELETE /api/v1/agents/{agent_id} with typed confirmation
    Then the agent is retired, not deleted
    And its CV versions and evidence remain readable
    And READY cards can no longer be assigned to it
    And the office still renders its desk with a plaque

  Scenario: [ ] E2E-023 Agent concurrency capacity is bounded
    Given the agent create or edit form
    When I set max_concurrent_cards to 0 or to 5
    Then the response status is 422
    When I set it to 1 through 4
    Then the response status is 201 or 200

  Scenario: [ ] E2E-024 CV versions are append-only and gain evidence
    Given an agent with CV version 1 and no delivered cards
    When a run assigned to the agent completes a card
    Then a new CV version is appended without mutating version 1
    And the new version contains evidence pointing to the card and run
    And inventory.agent.cv_updated is emitted

  Scenario: [ ] E2E-025 CV deltas are bounded and proficiency is clamped
    Given an agent skill at proficiency 5
    When a reflection report proposes a delta of +1
    Then the stored proficiency remains 5
    And given a delta of -1 the proficiency decreases by at most one
    And no single run changes a skill by more than one point

  Scenario: [ ] E2E-026 Lab settings are validated as a whole
    Given the lab settings editor
    When I submit a per-card budget greater than the per-day budget
    Then the response status is 422 with a budget violation
    When I submit a valid settings object with WIP limits, autonomy, scheduler, office and maintenance
    Then the response status is 200
    And archiving the lab afterwards is terminal and requires typed confirmation
```

---

## 4. Projects and SCM Credentials (E2E-027 – E2E-034)

```gherkin
@inventory @scm @p0 @fr-inv-030
Feature: Projects and GitHub credentials

  Scenario: [ ] E2E-027 Create a GitHub project referencing an SCM credential
    Given a lab and an SCM credential exist
    When I submit a project with name, unique slug, owner/repo and credential_id
    Then the response status is 201
    And the project stores a credential reference, not a raw token
    And the default branch defaults to main

  Scenario: [ ] E2E-028 Duplicate project slug or key prefix is rejected
    Given a lab already has a project with slug "csv-lab" and prefix "CSV"
    When I create another project in the same lab with the same slug or prefix
    Then the response status is 422
    And the error code is VALIDATION_FAILED

  Scenario: [ ] E2E-029 Project key prefix pattern is enforced
    Given the project create form
    When I submit a prefix such as "1abc", "CSV-TOOL" or "abcdefghijk"
    Then the response status is 422
    When I submit "C" + up to nine uppercase alphanumerics
    Then the response status is 201

  Scenario: [ ] E2E-030 The SCM token is write-only
    Given I create a GitHub credential with a fine-grained PAT
    Then the response exposes only the last four characters
    And no subsequent GET returns the plaintext token
    And the stored value is encrypted ciphertext with a key version

  Scenario: [ ] E2E-031 Project sync is queued and reconciles repository state
    Given an active GitHub-connected project
    When I call POST /api/v1/projects/{project_id}/sync
    Then the response status is 202
    And a sync job records the repository metadata and default branch
    And the result is observable in the project detail

  Scenario: [ ] E2E-032 Unsafe repositories are refused
    Given a repository that is archived, disabled or empty
    When I connect it or trigger a sync
    Then the operation is refused with a validation error
    And no branch, PR or check operation is attempted

  Scenario: [ ] E2E-033 Branch naming and path safety are enforced
    Given card key "AIL-42" and title "CSV inference"
    When the adapter creates the branch
    Then the branch name is ailab/AIL-42-csv-inference
    And a path containing "..", a leading slash or ".git/" is rejected
    And a branch name with unsafe characters is rejected

  Scenario: [ ] E2E-034 Pull request conventions and webhook verification
    Given a branch and a card
    When the adapter opens the pull request
    Then the title is "[AIL-42] CSV inference"
    And the body contains card, summary, changes, acceptance criteria, verification, risks and the AI-Lab trace
    When GitHub delivers a webhook with a valid X-Hub-Signature-256
    Then the webhook is accepted
    When the signature is invalid or the delivery is a replay older than five minutes
    Then it is rejected and no state is applied
```

---

## 5. Kanban Board and Card Lifecycle (E2E-035 – E2E-056)

```gherkin
@kanban @p0 @fr-kan-001
Feature: Kanban board and card lifecycle

  Scenario: [ ] E2E-035 The board has exactly six fixed ordered columns
    Given an active project
    When I call GET /api/v1/projects/{project_id}/board
    Then the columns are exactly TODO, READY, DOING, REVIEW, DONE, ICEBOX in that order
    And the column names are not configurable

  Scenario: [ ] E2E-036 Card keys are allocated once and never reused
    Given project prefix "AIL" with no cards
    When I create three cards
    Then the keys are AIL-1, AIL-2 and AIL-3
    And deleting or completing a card does not free its key

  Scenario: [ ] E2E-037 Card creation validates required fields
    Given the project board
    When I create a card without a title or without a source
    Then the response status is 422
    And the violations name the missing fields

  Scenario: [ ] E2E-038 A canonical story with criteria and deliverables is accepted
    Given a card payload with Title, As a/I want/so that, Context, Given/When/Then criteria and deliverables
    When I call POST /api/v1/projects/{project_id}/cards
    Then the response status is 201
    And the stored story preserves every section
    And needs_refinement is false

  Scenario: [ ] E2E-039 A TODO card may be intentionally incomplete
    Given a human-created card with only a title and a partial story
    When I save it with needs_refinement true
    Then the response status is 201
    And the card remains in TODO
    And the flag is visible on the card front

  Scenario: [ ] E2E-040 Card reads return the card or a not-found error
    Given an existing card id
    When I call GET /api/v1/cards/{card_id}
    Then the response status is 200 and matches the stored card
    When I call GET /api/v1/cards/{unknown_id}
    Then the response status is 404 with code NOT_FOUND

  Scenario: [ ] E2E-041 Updates are protected by optimistic concurrency
    Given a card at version 3
    When two clients both PATCH the card using expected version 3
    Then the first update succeeds and the version becomes 4
    And the second update returns 409 VERSION_CONFLICT with the current card

  Scenario: [ ] E2E-042 Card lists use cursor pagination
    Given 120 cards in a project
    When I call GET /api/v1/projects/{project_id}/cards?limit=50
    Then the response contains data, next_cursor and total_estimate
    And following next_cursor twice returns the remaining cards without duplicates
    And a limit above 200 is rejected or clamped according to the contract

  Scenario: [ ] E2E-043 Card filters combine assignee, label, priority, overdue and source
    Given cards with different assignees, labels, priorities and due dates
    When I filter by assignee and priority=must and source=agent
    Then only matching cards are returned
    And combining overdue=true returns only cards past their due date

  Scenario: [ ] E2E-044 Sorting is restricted to an allowlist
    Given the card list endpoint
    When I request sort=title or sort=due_date
    Then the response status is 200 and the order matches
    When I request an unknown sort field or an injection-shaped value
    Then the response status is 422 and no dynamic SQL is executed

  Scenario: [ ] E2E-045 Comments can be added and listed
    Given an existing card
    When I call POST /api/v1/cards/{card_id}/comments with a non-empty body
    Then the response status is 201
    When I call GET /api/v1/cards/{card_id}/comments
    Then the new comment is returned in chronological order

  Scenario: [ ] E2E-046 Card events are immutable and ordered
    Given a card that has been created and moved
    When I call GET /api/v1/cards/{card_id}/events
    Then the events are returned in increasing seq order
    And each event has from, to, actor_type, actor_id and occurred_at
    And the events cannot be updated or deleted through any endpoint

  Scenario: [ ] E2E-047 Cards can be assigned and reassigned
    Given a READY card and two agents with compatible roles
    When I call POST /api/v1/cards/{card_id}/assign for the first agent
    Then the assignee is recorded and an event is emitted
    When I reassign to the second agent
    Then the assignee changes and both assignments are auditable

  Scenario: [ ] E2E-048 Blocked reasons require an allowed category and text
    Given a card being moved to ICEBOX
    When I omit the reason or use category "misc"
    Then the response status is 422
    When I use dependency, clarification, external, risk, budget or policy with text
    Then the move is accepted and the reason is stored

  Scenario: [ ] E2E-049 Overdue cards are flagged but never moved automatically
    Given a card whose due date is in the past
    When the board is loaded
    Then the card has overdue true and a visible badge
    And the card remains in its current column
    And an agent speech nudge is emitted, not a transition

  Scenario: [ ] E2E-050 Concurrent card creation allocates unique keys
    Given 100 concurrent create requests for one project
    When all requests complete
    Then exactly 100 distinct keys exist
    And the sequence has no duplicates and no gaps for committed cards

  Scenario: [ ] E2E-051 A 500-card board loads within budget
    Given a project with 500 cards
    When I load the board through the API and the UI
    Then the board renders all cards
    And the p95 load time is at most 400 ms

  Scenario: [ ] E2E-052 An invalid drag-and-drop is rolled back with the exact violation
    Given the Kanban board with a TODO card missing acceptance criteria
    When I drag the card to READY and drop it
    Then the card animates back to TODO
    And a toast shows the first violation verbatim
    And no optimistic state remains after the 422 response

  Scenario: [ ] E2E-053 Same-column transitions are always denied
    Given a card in DOING
    When I request a transition from DOING to DOING
    Then the response status is 422 INVALID_TRANSITION
    And no card_event is written

  Scenario: [ ] E2E-054 Reopening a DONE card creates a new card with a parent link
    Given a DONE card
    When I call POST /api/v1/cards/{card_id}/reopen with a reason
    Then a new card is created in TODO
    And its parent_id points to the original card
    And the original card remains DONE with its full audit trail
    And a direct DONE to TODO transition is denied

  Scenario: [ ] E2E-055 All 36 ordered column pairs match the transition matrix
    Given every ordered pair of the six columns
    When I attempt each transition with valid guards where the matrix allows it
    Then exactly the 12 allowed transitions succeed
    And the 24 denied transitions return 422 INVALID_TRANSITION
    And same-column pairs are denied

  Scenario: [ ] E2E-056 Invalid transitions return a machine-readable violation payload
    Given a card that cannot move from TODO to DONE
    When I call POST /api/v1/cards/{card_id}/transition
    Then the response status is 422
    And the body is RFC 9457 Problem Details with code INVALID_TRANSITION
    And violations is a non-empty array with rule, field and message
    And trace_id is present
```

---

## 6. Kanban Transitions and Invariants (E2E-057 – E2E-076)

```gherkin
@kanban @guards @p0 @fr-kan-020
Feature: Kanban transition guards

  Scenario: [ ] E2E-057 TODO to READY succeeds when the Definition of Ready is complete
    Given a TODO card with a canonical story, one criterion, one deliverable, estimate 3, priority must, an assignee, a future due date and a positive budget
    When I transition the card from TODO to READY
    Then the response status is 200
    And the card column becomes READY
    And a kanban.card.moved event is emitted

  Scenario: [ ] E2E-058 TODO to READY fails without an acceptance criterion
    Given a TODO card whose story has no acceptance criteria
    When I transition it to READY
    Then the response status is 422
    And the violation rule is DOR_AC_MISSING and the field is story.acceptance_criteria

  Scenario: [ ] E2E-059 TODO to READY fails without a deliverable
    Given a TODO card whose story has no deliverables
    When I transition it to READY
    Then the response status is 422
    And the violation rule is DOR_DELIVERABLE_MISSING

  Scenario: [ ] E2E-060 TODO to READY fails when the card still needs refinement
    Given a TODO card flagged needs_refinement true
    When I transition it to READY
    Then the response status is 422
    And the violation rule is DOR_NEEDS_REFINEMENT

  Scenario: [ ] E2E-061 TODO to READY fails on an invalid estimate or priority
    Given a TODO card with estimate 13 or priority wont
    When I transition it to READY
    Then the response status is 422
    And the violations include DOR_ESTIMATE_INVALID or DOR_PRIORITY_INVALID

  Scenario: [ ] E2E-062 TODO to READY fails without assignee, due date or budget
    Given a TODO card missing an assignee, a future due date or a positive budget
    When I transition it to READY
    Then the response status is 422
    And the violations include DOR_ASSIGNEE_MISSING, DOR_DUE_DATE_INVALID or DOR_BUDGET_INVALID

  Scenario: [ ] E2E-063 TODO to ICEBOX requires a categorized reason
    Given a TODO card
    When I transition it to ICEBOX with category risk and a resolution note
    Then the response status is 200
    And the blocked reason is stored and the card is in ICEBOX

  Scenario: [ ] E2E-064 ICEBOX to TODO requires a recorded resolution
    Given an ICEBOX card with a blocked reason
    When I transition it to TODO with a resolution text and actor
    Then the response status is 200
    And the blocked reason is cleared
    And needs_refinement is set to true

  Scenario: [ ] E2E-065 READY to DOING succeeds within the WIP limit
    Given a READY card assigned to an active agent below its DOING limit
    When the scheduler leases and claims the card
    Then the response status is 200
    And the card column becomes DOING
    And a run row exists for the attempt

  Scenario: [ ] E2E-066 READY to DOING fails when the WIP limit is exceeded
    Given the assignee already has the maximum number of DOING cards
    When I claim another READY card
    Then the response status is 422
    And the error code is WIP_LIMIT_EXCEEDED

  Scenario: [ ] E2E-067 READY to TODO is a Tech Lead refinement action
    Given a READY card and a Tech Lead actor
    When the Tech Lead moves it back to TODO with a reason
    Then the response status is 200
    And needs_refinement is set to true
    And a non-Tech-Lead actor is rejected with 403 or 422

  Scenario: [ ] E2E-068 READY to ICEBOX blocks the card and notifies the assignee
    Given a READY card
    When I move it to ICEBOX with a categorized reason
    Then the response status is 200
    And the assignee receives a notification
    And the card remains visible in ICEBOX with its reason

  Scenario: [ ] E2E-069 DOING to REVIEW succeeds with a valid PR and green checks
    Given a DOING card with a run that succeeded, an open PR, a pushed branch, a non-empty log and a reviewer different from the assignee
    When I transition it to REVIEW
    Then the response status is 200
    And the card column becomes REVIEW
    And delivery.pr.opened and kanban.card.moved are emitted

  Scenario: [ ] E2E-070 Self-review is rejected by separation of duties
    Given a DOING card whose reviewer_id equals its assignee_id
    When I transition it to REVIEW
    Then the response status is 422
    And the error code is SEPARATION_OF_DUTIES

  Scenario: [ ] E2E-071 DOING to READY is an abandon transition
    Given a DOING card with a running attempt
    When the Tech Lead or the assignee abandons it with a reason
    Then the response status is 200
    And the run is cancelled
    And the card returns to READY

  Scenario: [ ] E2E-072 DOING to ICEBOX requires a reason and a terminal run
    Given a DOING card whose run failed terminally
    When I move it to ICEBOX with a categorized reason
    Then the response status is 200
    And the run is not left running

  Scenario: [ ] E2E-073 REVIEW to DONE succeeds only with the full Definition of Done
    Given a REVIEW card with an approved QA verdict, reviewer different from author, green CI, a merged PR, a merge SHA, complete DoD and no P0/P1
    When I transition it to DONE
    Then the response status is 200
    And the card column becomes DONE
    And delivery.merge.completed is emitted and CV evidence is appended

  Scenario: [ ] E2E-074 REVIEW to DONE reports every unmet DoD rule
    Given a REVIEW card missing the merge SHA, with failing CI or without QA approval
    When I transition it to DONE
    Then the response status is 422
    And the violations include DOD_MERGE_MISSING, DOD_CI_NOT_GREEN or DOD_QA_NOT_APPROVED

  Scenario: [ ] E2E-075 REVIEW to DOING returns changes with findings attached
    Given a QA verdict CHANGES_REQUESTED with at least one finding
    When I transition the card from REVIEW to DOING
    Then the response status is 200
    And the findings are attached to the card history
    And the Dev can read the actionable feedback

  Scenario: [ ] E2E-076 REVIEW to ICEBOX requires a reason and findings
    Given a REVIEW card that cannot proceed
    When I move it to ICEBOX with a categorized reason and attached findings
    Then the response status is 200
    And the card is not silently dropped
```

---

## 7. Idea Intake (E2E-077 – E2E-082)

```gherkin
@kanban @ideas @p0 @fr-kan-010
Feature: Natural-language idea intake

  Scenario: [ ] E2E-077 An idea creates canonical TODO cards within the latency budget
    Given a lab with a Tech Lead and a project
    When I submit an idea with at least one actionable capability
    Then the response status is 202
    And one or more TODO cards appear
    And the p95 time from submission to visible cards is at most 3 seconds

  Scenario: [ ] E2E-078 Generated cards follow the canonical story template
    Given a submitted idea
    When the Tech Lead creates cards
    Then every card has a title of at most 80 characters
    And every card has a story with persona, capability, outcome and context
    And every card has at least one acceptance criterion and one deliverable
    And every card has a Fibonacci estimate and a priority in must, should or could

  Scenario: [ ] E2E-079 Agents can create only TODO cards
    Given an agent acting through the kanban tool
    When it attempts to create a card in READY or DONE
    Then the attempt is rejected with POLICY_DENIED
    And the card is created in TODO only when the request targets TODO

  Scenario: [ ] E2E-080 Idea submission is idempotent by key
    Given an idea submitted with Idempotency-Key "idea-42"
    When the same request is replayed within 24 hours
    Then the original response is returned
    And no duplicate cards are created

  Scenario: [ ] E2E-081 Idea generation is rate limited
    Given the idea generation rate limit of five per minute is exhausted
    When I submit a sixth idea in the same minute
    Then the response status is 429
    And the Retry-After header is present

  Scenario: [ ] E2E-082 Idea text is validated
    Given the idea composer
    When I submit empty text or text longer than 20000 characters
    Then the response status is 422
    And the error code is VALIDATION_FAILED
```

---

## 8. SOW Lifecycle (E2E-083 – E2E-094)

```gherkin
@delivery @sow @p0 @fr-dlv-002
Feature: Statement of Work lifecycle

  Scenario: [ ] E2E-083 Creating a SOW starts asynchronous generation
    Given an approved idea and a project
    When I call POST /api/v1/projects/{project_id}/sows
    Then the response status is 202
    And a generation run is visible with streaming progress
    And the SOW persists as a structured document plus Markdown artifact

  Scenario: [ ] E2E-084 A generated SOW contains every mandatory section
    Given a completed SOW generation
    When I read the SOW
    Then it contains executive_summary, objectives, scope_in, scope_out, stakeholders
    And it contains functional_requirements, non_functional_requirements and architecture
    And it contains milestones, deliverables, acceptance_criteria and assumptions
    And it contains risks, timeline, budget, governance and change_control

  Scenario: [ ] E2E-085 A SOW missing or with an empty mandatory section is rejected
    Given a SOW payload with a missing or empty objectives section
    When I validate or approve it
    Then the response status is 422
    And the violation names the missing section

  Scenario: [ ] E2E-086 A SOW with a short executive summary is rejected
    Given a SOW whose executive_summary is under 120 characters
    When I validate it
    Then the response status is 422
    And the violation names executive_summary

  Scenario: [ ] E2E-087 Duplicate requirement identifiers are rejected
    Given a SOW with two requirements both named FR-001
    When I validate it
    Then the response status is 422
    And the violation reports the duplicate id

  Scenario: [ ] E2E-088 Requirements need priority and testable criteria
    Given a SOW with a functional requirement missing a priority or acceptance criterion
    Then validation returns 422
    And given a non-functional requirement without a numeric target or measurement method
    Then validation returns 422

  Scenario: [ ] E2E-089 A SOW moves from draft to in review to approved
    Given a draft SOW
    When I submit it for review
    Then the status becomes in_review
    When I approve it with an approver
    Then the status becomes approved and the approver is recorded

  Scenario: [ ] E2E-090 Approval freezes the current version
    Given an approved SOW at version 1
    When I attempt to edit version 1 in place
    Then the edit is rejected
    And version 1 remains immutable and readable

  Scenario: [ ] E2E-091 Revision requires a change note and creates a new draft version
    Given an approved SOW at version 1
    When I revise it with an empty change note
    Then the response status is 422
    When I revise it with a non-empty change note
    Then a new immutable version 2 enters draft
    And version 1 is unchanged

  Scenario: [ ] E2E-092 Rejection requires a reason and returns to draft
    Given a SOW in review
    When I reject it without a reason
    Then the response status is 422
    When I reject it with a reason
    Then the SOW returns to draft and the reason is recorded

  Scenario: [ ] E2E-093 Superseding requires a newer approved version
    Given an approved SOW version 1
    When I supersede it with an unapproved draft
    Then the operation is rejected
    When a newer approved version exists
    Then superseding succeeds and the old version is marked superseded

  Scenario: [ ] E2E-094 The SOW diff and export are deterministic
    Given two SOW versions
    When I call GET /api/v1/sows/{sow_id}/diff
    Then a unified diff is returned
    When I export the SOW as Markdown and as PDF
    Then the Markdown rendering is deterministic and its SHA-256 equals the content hash
    And both artifacts download with the correct content type
```

---

## 9. Plan, WBS and Materialization (E2E-095 – E2E-102)

```gherkin
@delivery @planning @p0 @fr-dlv-003
Feature: Plan, WBS and materialization

  Scenario: [ ] E2E-095 A plan is generated for an approved SOW
    Given an approved SOW
    When I call POST /api/v1/projects/{project_id}/plans
    Then the response status is 202
    And the plan is a DAG of tasks with keys, roles, estimates, deliverables and criteria
    And every task maps to exactly one functional requirement

  Scenario: [ ] E2E-096 Empty task lists and duplicate keys are rejected
    Given a plan with no tasks
    Then validation fails
    And given two tasks with the same task_key
    Then validation fails

  Scenario: [ ] E2E-097 Dangling or self-referential dependencies are rejected
    Given a task that depends on itself
    Then validation fails
    And given a task that depends on a missing key
    Then validation fails

  Scenario: [ ] E2E-098 Cyclic plans are rejected
    Given a plan where task A depends on B and B depends on A
    When I validate the plan
    Then the response status is 422
    And the error reports a cycle

  Scenario: [ ] E2E-099 Tasks must have an estimate, a deliverable and a criterion
    Given a task with no estimate, no deliverable or no acceptance criterion
    When I validate the plan
    Then the response status is 422
    And the violation names the incomplete task

  Scenario: [ ] E2E-100 Materialization preview selects leaves in topological order
    Given a valid acyclic plan
    When I request a materialization preview
    Then the preview returns only leaf tasks
    And the order is a valid topological order
    And rerunning the preview performs no writes and returns the same list

  Scenario: [ ] E2E-101 Functional requirement coverage is enforced
    Given a plan with an unknown, missing or uncovered functional requirement
    When I validate coverage
    Then the response status is 422
    And the uncovered requirement is named

  Scenario: [ ] E2E-102 Materialization creates cards with full traceability
    Given an approved plan and auto_ready disabled
    When I materialize it
    Then TODO cards are created using project keys
    And each card carries story, estimate, role, dependencies, task_key, covers and sow_id
    And parent links are preserved
    And auto_ready is allowed only when every DoR check passes
```

---

## 10. Runs and Orchestration (E2E-103 – E2E-118)

```gherkin
@delivery @orchestration @p0 @fr-orc-002
Feature: Scheduler, leases and run pipeline

  Scenario: [ ] E2E-103 Scheduler eligibility explains every rejected card
    Given READY cards that are inactive, over capacity, over budget or already leased
    When the scheduler evaluates them
    Then each ineligible card reports an explicit reason
    And an eligible READY card with an active assignee is selected

  Scenario: [ ] E2E-104 A card has at most one lease
    Given a READY card leased by worker A
    When worker B attempts to lease the same card
    Then worker B receives Conflict or ALREADY_LEASED
    And no second run is created

  Scenario: [ ] E2E-105 Lease TTL, heartbeat and reaping behave as specified
    Given a lease with TTL 900 seconds and a heartbeat every 15 seconds
    When the run heartbeats
    Then the lease is extended
    When the heartbeat stops and the TTL elapses
    Then reaping removes the lease
    And the run is timed out

  Scenario: [ ] E2E-106 The run state machine rejects invalid transitions
    Given a run in Succeeded
    When I attempt to move it to Running
    Then the transition is rejected
    And the valid transitions Queued to Running to Succeeded, Failed or TimedOut and Cancelled are accepted

  Scenario: [ ] E2E-107 Starting a run on an eligible card moves it to DOING
    Given an eligible READY card
    When I call POST /api/v1/cards/{card_id}/runs
    Then the response status is 202
    And a queued run is created
    And the card becomes DOING when the run starts

  Scenario: [ ] E2E-108 Starting a run on an ineligible card is rejected
    Given a card that is not READY or an agent that is paused
    When I call POST /api/v1/cards/{card_id}/runs
    Then the response status is 422
    And the reason is reported

  Scenario: [ ] E2E-109 Run steps are typed, ordered and streamed
    Given a running card
    When the run emits plan, tool_call, command, test, commit, pr and review steps
    Then the steps are persisted in order with arguments hash, duration and capped output
    And the UI receives them through the realtime stream
    And no step output exceeds 20 KB

  Scenario: [ ] E2E-110 Transient failures retry with bounded backoff
    Given a run that fails with a transient network error
    When the scheduler retries
    Then the attempts back off at 30 seconds, 2 minutes and 10 minutes with jitter
    And after the third retry the card is dead-lettered to ICEBOX with category risk

  Scenario: [ ] E2E-111 Deterministic failures return to DOING with feedback
    Given a run that fails because tests fail deterministically
    When the failure is classified
    Then the card returns to DOING with the failing output as feedback
    And the attempt counter increments

  Scenario: [ ] E2E-112 Policy failures dead-letter the card and escalate
    Given a run that violates a hard stop
    When the failure is classified as policy
    Then the card moves to ICEBOX
    And an escalation reaches the Tech Lead and optionally the operator
    And system.escalation is emitted

  Scenario: [ ] E2E-113 Budget exhaustion stops the run safely
    Given a run whose spend would exceed the card or daily budget
    When the next charge is evaluated
    Then the run stops before the charge is applied
    And BUDGET_EXCEEDED is persisted
    And the lab is paused and system.budget.exhausted is emitted

  Scenario: [ ] E2E-114 Missed heartbeats cancel and requeue the run
    Given a running card whose agent misses three heartbeats
    When the scheduler reconciles
    Then the run is cancelled or timed out
    And the card is requeued with backoff

  Scenario: [ ] E2E-115 A worker crash releases the lease after the TTL
    Given a worker holding a lease crashes
    When the lease TTL elapses
    Then the lease is released and the run is marked timed_out
    And the card becomes eligible for a new attempt

  Scenario: [ ] E2E-116 Restart reconciliation handles merged, open and branch-only states
    Given in-flight runs when the worker restarts
    When reconciliation runs
    Then a merged PR moves the card to DONE
    And an open PR resumes review
    And an existing branch resumes the push
    And a run with no evidence is retried

  Scenario: [ ] E2E-117 External side effects are idempotent
    Given a run that opens a PR, comments and merges
    When the operation is replayed with the same run_id, operation and target
    Then the external side effect is not duplicated
    And the replayed response matches the original

  Scenario: [ ] E2E-118 Failed runs are visible and replayable
    Given a dead-lettered run
    When I call GET /api/v1/labs/{lab_id}/runs?status=failed
    Then the failed run is returned with its error and cost
    And the office filing cabinet opens the same list
    And a replay action creates a new attempt without mutating the old run
```

---

## 11. QA Review and Merge Policy (E2E-119 – E2E-126)

```gherkin
@delivery @qa @p0 @fr-agt-001
Feature: Independent review and merge policy

  Scenario: [ ] E2E-119 An approved verdict requires every criterion to pass
    Given a review with all acceptance criteria marked pass and no P0, P1 or P2 finding
    When QA submits the verdict
    Then the verdict is approved
    And the review carries review_id, run_id, reviewer_id, head_sha and the matrix

  Scenario: [ ] E2E-120 Changes requested requires actionable findings
    Given a review with at least one failing criterion
    When QA submits changes_requested without findings
    Then the verdict is rejected
    When findings with severity, file and fix are attached
    Then the verdict is accepted and the card returns to DOING

  Scenario: [ ] E2E-121 Self-review is rejected
    Given a reviewer who is also the author
    When the review is submitted
    Then the response status is 422
    And the error code is SEPARATION_OF_DUTIES

  Scenario: [ ] E2E-122 Merge requires all policy conditions
    Given a PR that is not merged, has failing CI or lacks QA approval
    When a merge is attempted
    Then the merge is refused with PR_NOT_MERGED, CI_NOT_GREEN or QA_NOT_APPROVED
    When all conditions hold and the merge SHA is recorded
    Then the merge proceeds

  Scenario: [ ] E2E-123 A0 and A1 require a human merge for agent-initiated merges
    Given a lab at autonomy A0 or A1
    When an agent attempts to merge
    Then the merge is refused with HUMAN_MERGE_REQUIRED
    And the operator is prompted to merge

  Scenario: [ ] E2E-124 A2 and A3 allow autonomous merge after the checks
    Given a lab at autonomy A2 or A3 with QA approval, green CI, no P0/P1 and budget remaining
    When the agent merges with a squash merge
    Then the merge succeeds
    And the card can reach DONE

  Scenario: [ ] E2E-125 Repeated merges of the same head are idempotent
    Given a merge already completed for head SHA abc123
    When the merge is invoked again for the same head
    Then the response is success
    And no second merge is performed

  Scenario: [ ] E2E-126 Forbidden changes and protected branches are hard stops
    Given a diff touching .github/workflows, a secret or more than 1500 lines
    When the merge policy evaluates it
    Then the merge is refused with FORBIDDEN_DIFF or the matching hard stop
    And a merge targeting a protected branch is refused with PROTECTED_BRANCH
```

---

## 12. Memory and Knowledge (E2E-127 – E2E-138)

```gherkin
@memory @p0 @fr-mem-001
Feature: Scoped memory and hybrid retrieval

  Scenario: [ ] E2E-127 Exactly three memory scopes exist
    Given the memory API
    When I create a document with scope lab, project or agent and a matching scope id
    Then the document is accepted
    When I use scope global or an unknown scope
    Then the response status is 422

  Scenario: [ ] E2E-128 Scope id must match the scope
    Given a project-scoped document
    When I provide a lab id as scope_id
    Then the response status is 422
    And no document or embedding is written

  Scenario: [ ] E2E-129 Chunking is deterministic
    Given a document of N words
    When I ingest it twice
    Then the chunk boundaries and token counts are identical
    And chunks target 800 words with 120-word overlap

  Scenario: [ ] E2E-130 Embeddings have 384 dimensions
    Given an ingested chunk
    When I inspect its embedding
    Then the vector has exactly 384 dimensions

  Scenario: [ ] E2E-131 Secret-shaped documents are quarantined
    Given a document containing a known secret shape such as sk-, ghp_ or a PEM header
    When I ingest it
    Then the response status is 422
    And the error code is MEMORY_QUARANTINED
    And the document is not embedded

  Scenario: [ ] E2E-132 Hybrid retrieval ranks by the specified formula
    Given a fixed retrieval fixture
    When I search
    Then the weighted score is 0.7 * max(cosine, 0) + 0.3 * lexical_rank
    And the final score applies exp(-age_days / 30) and the scope boost
    And results are ordered by score to the fixture tolerance

  Scenario: [ ] E2E-133 Retrieval never crosses scopes
    Given a lab, a project and an agent each with distinct documents
    When I search the project scope for text that exists only in the lab scope
    Then no lab-scoped chunk is returned
    And the SQL filter is applied before scoring

  Scenario: [ ] E2E-134 Near-duplicate chunks are deduplicated
    Given two chunks with cosine similarity at least 0.92 in the same scope
    When I search
    Then only one of them is returned

  Scenario: [ ] E2E-135 Compaction and retention respect pins
    Given episodes older than 30 days and raw episodes older than 180 days
    When compaction runs
    Then old episodes are summarized into semantic documents
    And raw episodes older than 180 days are deleted
    And pinned episodes are preserved

  Scenario: [ ] E2E-136 Every retrieval is logged for audit and evaluation
    Given a search request
    When it completes
    Then a log row records the query, scopes, chunk ids and scores

  Scenario: [ ] E2E-137 Memory notes can be created, pinned and deleted
    Given the memory explorer
    When I add a note, pin it and later delete it
    Then each action succeeds and is audited
    And the pinned note survives compaction

  Scenario: [ ] E2E-138 Memory import and export respect size and privacy rules
    Given a memory import larger than 10 MB
    When I submit it
    Then the request is rejected
    And given a valid export request the data is returned without plaintext secrets
```

---

## 13. Realtime and Events (E2E-139 – E2E-144)

```gherkin
@realtime @p0 @ws-001
Feature: WebSocket realtime and event replay

  Scenario: [ ] E2E-139 The WebSocket handshake returns hello metadata
    Given an authenticated realtime client
    When I connect to GET /api/v1/ws
    Then the server sends hello with server_time, seq and topics
    And the envelope format matches the AsyncAPI contract

  Scenario: [ ] E2E-140 Subscribe and unsubscribe are acknowledged
    Given a connected client
    When I subscribe to lab:{id} and card:{id}
    Then the server acknowledges the subscription
    When I unsubscribe from card:{id}
    Then no further card events are delivered to that client

  Scenario: [ ] E2E-141 State events are persisted and ephemeral events are not
    Given a connected client
    When a card moves and an agent types
    Then kanban.card.moved is persisted and replayable
    And agent.typing is coalesced and never persisted

  Scenario: [ ] E2E-142 Reconnect resumes from the last applied sequence
    Given a client that disconnects after seq 100
    When it reconnects with since=100
    Then only events with seq greater than 100 are delivered
    And duplicate event ids are removed and order is by seq

  Scenario: [ ] E2E-143 A gap larger than the buffer triggers HTTP backfill
    Given a reconnect whose gap exceeds the in-memory buffer
    When the client resumes
    Then it calls GET /api/v1/events?after={seq}
    And the events are applied in order without loss

  Scenario: [ ] E2E-144 Backpressure and heartbeat are enforced
    Given a slow client with a full 256-message buffer
    When more events arrive
    Then ephemeral events are dropped first
    And the connection closes with code 1013 if state events cannot be delivered
    And the server pings every 20 seconds and expects a pong within 10 seconds
```

---

## 14. Virtual Office (E2E-145 – E2E-156)

```gherkin
@office @ux-001 @p0
Feature: Pixel virtual office

  Scenario: [ ] E2E-145 Every active agent is rendered with a persistent desk
    Given a lab with four agents
    When I open /lab/{labId}
    Then each agent appears at a desk
    And each desk plaque shows name, role icon and a status LED

  Scenario: [ ] E2E-146 Animation is derived truthfully from domain state
    Given agents that are idle, working, blocked and paused
    When the office renders
    Then idle shows idle, working shows type, blocked shows blocked and paused shows sleep
    And no agent invents activity that is not in the domain

  Scenario: [ ] E2E-147 Pathfinding is deterministic and avoids corner cutting
    Given an agent that must walk around furniture
    When it computes a path
    Then the path is the shortest octile path
    And diagonals do not cut blocked corners
    And the same grid and seed produce the same positions

  Scenario: [ ] E2E-148 Clicking an agent opens the inspector
    Given the office is open
    When I click an agent
    Then the Agent Inspector opens with Live, CV, Memory and Controls tabs

  Scenario: [ ] E2E-149 Deep links restore selection and panel
    Given the URL /lab/{labId}?agent={agentId}&panel=cv
    When the page loads
    Then the inspector opens on the CV tab for that agent
    And the object exposes a shareable URL

  Scenario: [ ] E2E-150 Office objects open their bound surfaces
    Given the office is open
    When I click the whiteboard, server rack, bookshelf and filing cabinet
    Then they open Kanban, System Health, Memory explorer and Failed Runs respectively

  Scenario: [ ] E2E-151 Speech bubbles come from real events
    Given a domain event that produces an agent message
    When the event arrives
    Then a bubble of at most 120 characters appears
    And it auto-dismisses after 6 seconds
    And clicking it opens the detail panel

  Scenario: [ ] E2E-152 Reduced motion is honored
    Given prefers-reduced-motion is set
    When the office renders
    Then walking and ambient motion are replaced with instant or static poses
    And state changes and speech text remain visible

  Scenario: [ ] E2E-153 The screen-reader office list announces transitions
    Given a screen reader is active
    When agent states change
    Then the hidden office list exposes agent, status, current card and actions
    And a live region announces the transition

  Scenario: [ ] E2E-154 The office meets its frame-rate budget
    Given 16 agents and 200 animated props
    When the office runs at 1440p on the reference host
    Then desktop FPS is at least 58
    And the low-end profile sustains at least 30 FPS

  Scenario: [ ] E2E-155 Scaling is integer and pixelated
    Given a viewport of any size
    When the office renders
    Then the internal resolution is 384x216
    And the scale factor is an integer
    And the canvas uses image-rendering pixelated with no fractional blur

  Scenario: [ ] E2E-156 Retired agents keep a desk and the door opens the retired list
    Given a retired agent
    When I open the office and click the exit door
    Then the retired agent appears as an empty desk with a plaque
    And the door opens the retired agents list
```

---

## 15. Dashboard, UI and Accessibility (E2E-157 – E2E-164)

```gherkin
@ui @ux-040 @p0
Feature: Dashboard, product surfaces and accessibility

  Scenario: [ ] E2E-157 The dashboard shows real KPIs and never fake zeros
    Given the demo seed is loaded
    When I open /
    Then the KPI strip shows active agents, cards by column, spend today and success rate
    And while loading it shows a loading state, and on error an error card with retry
    And it never reports zeros as if they were real data

  Scenario: [ ] E2E-158 The activity feed and health pill reflect live state
    Given recent domain events
    When the dashboard loads
    Then the activity feed shows the last 50 events
    And the system health pill reflects the current health status

  Scenario: [ ] E2E-159 The onboarding checklist guides first-run setup
    Given a fresh installation with no provider
    When I open the dashboard
    Then the checklist lists provider, key, lab, Tech Lead, GitHub, project and idea
    And each completed step is marked done
    And the next incomplete step is actionable

  Scenario: [ ] E2E-160 Empty states are illustrated and actionable
    Given a lab with no projects
    When I open the projects inventory
    Then an illustrated empty state with a single clear CTA is shown
    And there is no dead end

  Scenario: [ ] E2E-161 Board manipulation works from the keyboard
    Given a focused TODO card
    When I press M and the arrow keys and Enter
    Then the card moves according to the transition guards
    And Escape cancels the move
    And focus remains visible throughout

  Scenario: [ ] E2E-162 The shell is responsive across breakpoints
    Given widths at or above 1280, between 768 and 1279, and below 768
    When I load the app at each width
    Then the sidebar is expanded, collapsed to icons and a bottom sheet respectively
    And the Kanban becomes a single-column swipe on narrow screens
    And orientation changes reflow without a reload

  Scenario: [ ] E2E-163 Automated accessibility checks find no serious issues
    Given the routes /, /settings, /memory and /design
    When axe runs with WCAG 2.2 AA rules
    Then there are no critical or serious violations
    And focus is visible and meaning is never conveyed by color alone

  Scenario: [ ] E2E-164 The global error boundary degrades gracefully
    Given the API returns an unexpected error
    When I trigger a failing action
    Then a pixel-styled toast with a trace id and retry appears
    And the screen is not blank
    And the error is reported to /api/v1/telemetry/frontend with the build hash
```

---

## 16. Security and Exposure (E2E-165 – E2E-172)

```gherkin
@security @p0 @sec-001
Feature: Security, exposure and guardrails

  Scenario: [ ] E2E-165 The default exposure is loopback with no login
    Given the default configuration
    When the api starts
    Then it binds to 127.0.0.1
    And no bearer token is required for local requests

  Scenario: [ ] E2E-166 Non-loopback binding requires a strong token
    Given AILAB_BIND is not loopback and AILAB_API_TOKEN is missing or shorter than 32 bytes
    When the api starts
    Then it refuses to start with exit code 78
    And with a valid token every /api and /ws request except /health/live requires it
    And the UI stores the token in memory and sessionStorage only

  Scenario: [ ] E2E-167 Security headers are present
    Given any web response
    When I inspect the headers
    Then CSP, nosniff, Referrer-Policy, Permissions-Policy and COOP are present
    And no inline or third-party script is allowed by the CSP

  Scenario: [ ] E2E-168 Mutations require JSON and an allowed origin
    Given a mutation request with a non-JSON content type or a disallowed Origin
    When I submit it
    Then it is rejected
    And a same-origin JSON mutation is accepted

  Scenario: [ ] E2E-169 Secrets are encrypted with authenticated encryption
    Given a provider key encrypted with AES-256-GCM and an AAD of table:column:row:key_version
    When the ciphertext is moved to another row or column
    Then decryption fails
    And only last4 is ever returned by the API

  Scenario: [ ] E2E-170 Logs and errors never leak secrets
    Given a request containing a bearer token or an API key
    When the request is logged or fails
    Then the log line and the Problem Details are redacted
    And no secret appears in any response, log or trace

  Scenario: [ ] E2E-171 GitHub webhooks are verified and replay-protected
    Given the webhook secret
    When a delivery has a valid X-Hub-Signature-256
    Then it is accepted
    And an invalid or stale signature is rejected with 401
    And a duplicate X-GitHub-Delivery is acknowledged once with 202

  Scenario: [ ] E2E-172 The sandbox blocks escape and unapproved egress
    Given a run executing in the sandbox
    When the workload tries to access the host filesystem, the Docker socket, a non-allowlisted host, escalate privileges, fork-bomb or fill the disk
    Then each attempt is denied
    And the container is non-root, read-only, without host mounts and with network none by default
```

---

## 17. API Contracts and Observability (E2E-173 – E2E-180)

```gherkin
@api @observability @p0 @api-002
Feature: API contracts, metrics and traces

  Scenario: [ ] E2E-173 Errors use RFC 9457 Problem Details
    Given any failing request
    When the error response is inspected
    Then it contains type, title, status, detail, instance, code, violations and trace_id
    And the code is a stable catalog value such as INVALID_TRANSITION or VALIDATION_FAILED

  Scenario: [ ] E2E-174 Correlation identifiers are propagated
    Given I send a valid X-Request-Id
    When the request is processed
    Then the response echoes X-Request-Id
    And a traceparent-compatible trace id is attached to logs and error payloads

  Scenario: [ ] E2E-175 Rate limits are enforced by class
    Given the read, write and idea/SOW generation limits
    When I exceed 120 reads, 30 mutations or 5 generations per minute
    Then the matching request receives 429 with Retry-After
    And requests below the limit are unaffected

  Scenario: [ ] E2E-176 The OpenAPI document matches the served API
    Given the published .docs/product/api/openapi.json
    When schemathesis runs against the API
    Then every documented path and schema conforms
    And an additive change is accepted while a removal requires /v2

  Scenario: [ ] E2E-177 List endpoints follow the pagination contract
    Given a list endpoint
    When I request limit and cursor and sort and filters
    Then the response is data, next_cursor and total_estimate
    And limits are capped at 200 and unknown filters are rejected

  Scenario: [ ] E2E-178 Metrics and structured logs are exposed and redacted
    Given the api and worker are running
    When I call GET /metrics
    Then Prometheus metrics include HTTP, DB pool, Redis, queue depth, lease age, run, LLM, cost, WS, event lag and office FPS
    And every request log line is JSON with method, path, status and duration but no secrets

  Scenario: [ ] E2E-179 Traces link the whole pipeline
    Given a run that reaches an LLM and GitHub call
    When the trace is inspected
    Then API, use case, DB, outbox, worker, LLM and GitHub spans share one trace_id
    And errors and run pipelines are sampled at 100 percent

  Scenario: [ ] E2E-180 Health and system status are independently observable
    Given the api, worker, provider and SCM
    When I call /health/live, /health/ready, /health/providers and /health/scm
    Then each reports its own component status
    And a system.health event is emitted every 15 seconds
```

---

## 18. Maintenance, Budget and Metrics (E2E-181 – E2E-188)

```gherkin
@maintenance @metrics @p1 @fr-maint-001
Feature: Maintenance, budgets and flow metrics

  Scenario: [ ] E2E-181 Card and daily budgets are enforced
    Given a lab with a per-card and per-day budget
    When a run charges tokens
    Then the ledger records the cost per card, agent and lab
    And the run stops before exceeding either budget
    And the remaining budget is exposed as a metric

  Scenario: [ ] E2E-182 Budget exhaustion pauses the lab and can resume
    Given the lab budget is exhausted
    When a run reaches the limit
    Then system.budget.exhausted is emitted
    And active runs checkpoint safely and the lab is paused
    When the operator increases the budget
    Then the lab resumes and queued work continues

  Scenario: [ ] E2E-183 Flow metrics are derived from immutable events
    Given the demo seed
    When I call GET /api/v1/labs/{lab_id}/metrics
    Then lead time, cycle time p50 and p95, throughput, WIP and blocked counts are returned
    And the values agree with the card_events trail

  Scenario: [ ] E2E-184 Spend is visible at every level with a trend
    Given completed runs with recorded costs
    When I view spend by lab, project, card and agent
    Then each level shows its spend
    And a 30-day trend and a forecast are available

  Scenario: [ ] E2E-185 The maintenance job catalog is seeded and idempotent
    Given the migrated database
    When the worker ticks
    Then the job catalog includes data_integrity, memory_compaction, cv_reflection_backfill, dependency_check, stale_card_scan, overdue_notify and provider_health
    And each job runs idempotently and records a structured outcome

  Scenario: [ ] E2E-186 Maintenance jobs create work without auto-moving cards
    Given a dependency finding and a card stale for more than seven days
    When the dependency_check and stale_card_scan jobs run
    Then a TODO card is created for the dependency finding
    And the stale card is flagged with a refinement nudge
    And no card is moved automatically

  Scenario: [ ] E2E-187 Memory hygiene and CV backfill are bounded and observable
    Given episodes eligible for compaction and runs missing CV evidence
    When memory_compaction and cv_reflection_backfill run
    Then compaction respects pins and retention
    And the CV backfill is bounded to 500 runs
    And both jobs emit completion events and metrics

  Scenario: [ ] E2E-188 Provider health probing drives status and events
    Given a provider endpoint that starts failing
    When the provider_health probe runs on its 60-second schedule
    Then the provider status becomes degraded
    And a health event is emitted and surfaced in the UI
```

---

## 19. Golden Journeys J1–J4 (E2E-189 – E2E-194)

```gherkin
@journey @p0 @ac-003
Feature: Golden end-to-end journeys

  Scenario: [ ] E2E-189 J1 Idea to backlog
    Given a project and a lab with a Tech Lead
    When the operator submits an idea in natural language
    Then canonical TODO cards are created within 3 seconds p95
    And the cards animate into the Kanban
    And the office announces them with a speech bubble and an activity event
    And each card has story, criteria, deliverables, estimate and priority

  Scenario: [ ] E2E-190 J2 Idea to approved SOW and materialized plan
    Given a project, a GitHub connection, a lab and a team
    When the operator requests a SOW and approves it
    Then the Analyst produces a structured SOW and an acyclic WBS
    And every section is versioned and diffable
    And materialization creates TODO cards linked to the SOW and plan

  Scenario: [ ] E2E-191 J3 Autonomous execution end to end
    Given a READY card with DoR satisfied and autonomy A2
    When the scheduler leases it to the assignee
    Then the agent creates a branch, implements, runs tests in the sandbox and opens a PR
    And the card moves to REVIEW
    And QA verifies every criterion independently
    Then the Tech Lead merges per policy and the card reaches DONE
    And the merge SHA, audit trail and cost accounting are recorded
    And CV and memory are updated by reflection

  Scenario: [ ] E2E-192 J4 Watch and steer
    Given the office is open
    When the operator clicks agents, the whiteboard, the server rack and the bookshelf
    Then the office shows live status, current card, log lines, CV and memory
    And an invalid drag is rejected with the exact reason
    And pause, resume, reassign and message controls work and are audited

  Scenario: [ ] E2E-193 The deterministic demo reproduces the golden path
    Given make up and make seed-demo
    When I follow .docs/product/demo-script.md
    Then the seed reproduces one lab, four agents, one project, one SOW, 12 cards and 30 events
    And the golden path completes in at most five minutes
    And the expected visual output is reproducible

  Scenario: [ ] E2E-194 The full setup is possible through the UI
    Given a fresh installation
    When the operator uses only the UI
    Then they create a provider, a lab, a Tech Lead, an Analyst, a Dev, a QA and a GitHub-connected project
    And every step validates inline and reaches a ready state
```

---

## 20. Reliability and Recovery (E2E-195 – E2E-200)

```gherkin
@reliability @p1 @rel-001
Feature: Reliability, backup and recovery

  Scenario: [ ] E2E-195 Backup and restore drill meet the recovery objectives
    Given a populated database and artifacts
    When make backup runs and then make restore-drill restores into a scratch database
    Then the dump and manifest contain every table and artifact reference
    And the restored row counts match the source
    And the restore completes within the RTO of one hour
    And no master key is present in the backup

  Scenario: [ ] E2E-196 Nightly integrity checks are report-only
    Given a database with no orphans or invariant violations
    When the data_integrity job runs
    Then it reports zero findings
    And given an injected orphan or projection mismatch
    Then it reports the finding as P1
    And it never performs a destructive automatic repair

  Scenario: [ ] E2E-197 Graceful shutdown drains and releases resources
    Given in-flight requests and active runs
    When the service receives a shutdown signal
    Then it stops accepting new work
    And it drains for at most 20 seconds
    And it checkpoints or finishes runs, releases leases and flushes telemetry

  Scenario: [ ] E2E-198 Dependency degradation is explicit and safe
    Given Redis is unavailable
    When the app is configured for degraded mode
    Then a read-only banner is shown and correctness is preserved
    And given Redis fail-closed rate limiting the API returns an explicit error rather than allowing unbounded traffic
    And a provider outage queues work and surfaces a visible status instead of losing data

  Scenario: [ ] E2E-199 A provider outage stalls delivery but does not corrupt state
    Given the LLM provider is down
    When a run attempts a completion
    Then the run is queued or failed as transient according to policy
    And the card and audit state remain consistent
    And the provider health UI shows the outage

  Scenario: [ ] E2E-200 No milestone regresses a previously accepted journey
    Given the full E2E-001 to E2E-199 suite
    When CI runs on the release candidate
    Then every deterministic scenario passes or is explicitly quarantined with an owner and a card
    And no P0 or P1 is open
    And the evidence is attached to the release acceptance record
```

---

## 21. QA Gap-Closure Scenarios (E2E-201 – E2E-240)

These scenarios close the gaps found by the 2026-11-09 cross-check against §6–§20 and §23. They use the same checklist markers and evidence rules as the primary set.

### 21.1 Inventory, agents and team

```gherkin
@inventory @team @p1 @fr-inv-001
Feature: Inventory and team gap closure

  Scenario: [ ] E2E-201 Provider status lifecycle is controllable and audited
    Given an active provider with a valid key
    When I disable it, request a completion and then enable it again
    Then the disabled provider is not selected for new runs
    And a failed health probe marks it degraded with a visible status
    And enabling it restores active status
    And every status change is written to the audit log with actor and reason

  Scenario: [ ] E2E-202 Model fields are validated and the catalog stays consistent
    Given the model create or edit form
    When I submit a non-positive context window or a negative input or output cost
    Then the response status is 422
    And a valid model with positive windows and non-negative costs is accepted
    And the catalog never contains a third v1 model by accident

  Scenario: [ ] E2E-203 Provider and lab token buckets queue work on exhaustion
    Given a provider rate limit and a lab rate limit are configured
    When both buckets are exhausted for the current window
    Then new completion requests are queued rather than failed
    And the queue drains when the window resets
    And the per-provider and per-lab counters are independent

  Scenario: [ ] E2E-204 SCM credential verification and PAT class are explicit
    Given a fine-grained GitHub PAT
    When I create and verify the SCM credential
    Then the verification state is recorded without exposing the token
    And a classic PAT is accepted with a visible warning
    And an invalid or expired token sets the credential to unverified with a reason

  Scenario: [ ] E2E-205 Unknown agent roles are rejected
    Given the agent create form
    When I submit role "manager" or any value outside tech_lead, analyst, dev and qa
    Then the response status is 422
    And the role violation is reported

  Scenario: [ ] E2E-206 The agent state machine is explicit and observable
    Given an offline agent
    When it starts, receives a lease, becomes blocked, resolves the block, pauses and resumes
    Then the status sequence follows offline, idle, working, blocked, working, paused, idle
    And each change emits agent.status_changed
    And the office reflects the new state within 250 ms p95

  Scenario: [ ] E2E-207 Effective autonomy and self-modification are enforced
    Given a lab at A2, a card at A1 and an agent at A3
    When a run computes its effective autonomy
    Then the effective level is the minimum of the three, A1
    And a lab cannot enable A3 without an explicit opt-in and a hard monthly budget
    And an agent attempt to change its own charter, CV, autonomy, budget or scheduler settings is rejected
    And the rejection is audited
```

### 21.2 Prompt assembly, LLM policy and tool policy

```gherkin
@llm @agents @p1 @fr-llm-010
Feature: Prompt and LLM policy gap closure

  Scenario: [ ] E2E-208 Prompt assembly is deterministic and reproducible
    Given the same role, character, CV, hard rules, tools, task packet and untrusted context
    When the prompt is assembled twice
    Then the rendered sections appear in the fixed Appendix C order
    And prompt_hash is identical for identical inputs and differs when an input changes
    And the character section is capped at 400 tokens, with role rules winning and character_override logged on conflict

  Scenario: [ ] E2E-209 Role tool allowlists deny and log policy violations
    Given a Dev, an Analyst, a QA and a Tech Lead
    When each calls a tool outside its role allowlist
    Then the call returns POLICY_DENIED
    And the denial is logged with run id, agent id and tool name
    And every allowed tool call is recorded in the run steps

  Scenario: [ ] E2E-210 Model routing justifies escalation
    Given a default classification request
    When it is routed
    Then it uses deepseek-flash
    And given the SOW architecture section, a third debugging attempt, a security review or a Tech Lead conflict
    Then deepseek-v4-pro may be used with a recorded justification
    And a pro use without an allowed reason is rejected

  Scenario: [ ] E2E-211 Structured outputs validate and repair exactly once
    Given a structured call against a JSON Schema
    When the model returns invalid JSON
    Then one repair request is sent
    And a second invalid response fails with InvalidOutput
    And a valid response is parsed into the typed value before use

  Scenario: [ ] E2E-212 Token budgets allocate context and summarize overflow
    Given a model context window with system, memory, repo, task and reserve budgets
    When the context exceeds a section budget
    Then system is capped at 20 percent, memory at 25, repo at 35, task at 15 and reserve at 5
    And overflow is summarized or omitted by priority
    And the task packet is never silently dropped

  Scenario: [ ] E2E-213 Streaming, retries and untrusted content are safe
    Given a UI-visible SOW or plan generation
    When tokens stream to the client
    Then deltas are coalesced at 50 ms
    And 429 and 5xx responses retry with exponential backoff and Retry-After
    And the circuit opens after five consecutive failures and half-opens after 60 seconds
    And untrusted content is wrapped and cannot break out of its wrapper
    And prompts and responses are persisted redacted with the configured retention
```

### 21.3 Delivery, orchestration and traceability

```gherkin
@delivery @orchestration @p1 @fr-orc-010
Feature: Delivery and orchestration gap closure

  Scenario: [ ] E2E-214 Agent commits and pull requests are traceable
    Given a Dev agent commit
    When the commit and PR are created
    Then the commit uses a Conventional Commit type and scope
    And the commit footer carries AI-Lab-Run, AI-Lab-Card and AI-Lab-Agent
    And the PR title is "[{CARD_KEY}] {TITLE}"
    And the PR body includes the full AI-Lab trace block

  Scenario: [ ] E2E-215 Context build is complete and bounded
    Given a READY card
    When the run builds context
    Then it includes the card, repo map, memory hits, relevant files, conventions and previous review feedback
    And the initial file payload does not exceed 120 KB or 30k tokens
    And the task packet is always present

  Scenario: [ ] E2E-216 The agent loop is bounded and plans before mutating
    Given a run that reaches the implement stage
    When the loop runs
    Then it makes at most 40 tool calls and 25 iterations within 45 minutes
    And a plan RunStep is persisted and streamed before any file is written
    And a response never exceeds 8192 output tokens

  Scenario: [ ] E2E-217 Verification runs before commit and feeds failures back
    Given a run ready to commit
    When verification runs in the sandbox
    Then formatter, linter, type check, unit tests and secret scan execute in order
    And a failure returns the raw output, truncated to 20 KB, to the agent
    And a run cannot commit while verification is red

  Scenario: [ ] E2E-218 Run records and step traces are complete
    Given a completed run
    When I read the run and its steps
    Then the run records card, agent, attempt, status, lease, sandbox, tokens, cost, error and result summary
    And each step records type, arguments hash, duration, exit status and capped output
    And all steps share one trace_id

  Scenario: [ ] E2E-219 Handoffs, escalations and human decisions are structured
    Given an agent that cannot finish its card
    When it hands off or escalates
    Then the Handoff object contains from, to, card, summary, state, open questions, artifacts and acceptance status
    And free-form chat is not accepted as a handoff
    And escalation includes the last 20 log lines, the failing command and a proposed next action
    And a human intervention is recorded as a Decision visible to the next agent
```

### 21.4 Memory, knowledge, Kanban and data

```gherkin
@memory @kanban @data @p1 @fr-dlv-007
Feature: Memory, knowledge and data gap closure

  Scenario: [ ] E2E-220 Reflection updates CV and proposes memory safely
    Given a completed or failed run
    When the reflection job runs
    Then it produces a ReflectionReport with outcome, skills demonstrated and missing, failure modes, lessons and memory candidates
    And skill deltas stay within minus one to plus one per run and proficiency stays within one to five
    And a memory candidate is written only after cosine deduplication at 0.92 or higher and secret scanning

  Scenario: [ ] E2E-221 Decisions are promoted to project memory
    Given a run decision with rationale, alternatives and links
    When it is promoted
    Then a project-scoped memory document is created as an ADR candidate
    And it is linked to the card and run for traceability

  Scenario: [ ] E2E-222 Memory tiers and Redis working memory behave as specified
    Given episodic, semantic and procedural memory
    When I inspect retention
    Then episodes are append-only with the 180-day raw retention and summarized compaction
    And semantic knowledge is stored as documents and vectors
    And procedural knowledge is the agent CV, tools and playbooks
    And Redis keeps the last 40 messages per run, a retrieval cache with TTL 300 seconds and namespaced keys
    And a Redis cache miss never breaks correctness

  Scenario: [ ] E2E-223 Retrieval bounds and data scrubbing are enforced
    Given a memory search without an explicit k
    When it runs
    Then the default is 8 and the maximum is 50
    And secret and PII scrubbing runs before embedding and before sending context to the LLM
    And a scrubbed secret never appears in a chunk or a prompt

  Scenario: [ ] E2E-224 Kanban dependencies and card splitting are enforced
    Given a READY candidate with an open dependency
    When the Tech Lead attempts to move it to READY without a waiver
    Then the response status is 422 with DOR_DEPENDENCY_OPEN
    And a Tech Lead waiver with a reason allows the move
    And a card estimated 13, or a card larger than 8 that should be split, is refused or split before READY

  Scenario: [ ] E2E-225 Work is card-bound and card data is durable
    Given any agent action that mutates the repository
    When the action is attempted outside a card and run
    Then it is rejected
    And a card persists parent_id, sow_id, budget_usd, risk and version
    And due dates are stored in UTC and rendered in the operator timezone with Intl
    And archiving a project blocks new cards, while moving a project between labs requires an audited transfer
```

### 21.5 Virtual office and frontend

```gherkin
@office @frontend @p1 @ux-016
Feature: Office and frontend gap closure

  Scenario: [ ] E2E-226 Office camera and object navigation are complete
    Given the office is open
    When I hover an object
    Then a pixel tooltip and outline appear
    When I press Tab, Enter and Escape
    Then focus cycles through interactive objects, Enter opens and Escape closes
    And wheel and drag pan and zoom snap to integer factors 1x to 4x
    And the fit button resets the camera

  Scenario: [ ] E2E-227 The office scales to sixteen agents without overlap
    Given a lab with sixteen agents
    When the office renders
    Then every agent is visible at a desk, hot desk or standing position
    And no two agents or desks overlap
    And the layout remains navigable

  Scenario: [ ] E2E-228 Office assets are licensed and original
    Given the sprite, font and sound manifest
    When the asset linter runs
    Then every asset has author, source and redistributable license metadata
    And a missing license fails the build
    And no asset is copied from the reference product and no emoji is used in the office

  Scenario: [ ] E2E-229 Sound and background-tab behaviour respect the operator
    Given the office is running
    When a card completes or is blocked
    Then the matching chime plays at the configured volume
    And mute disables it and prefers-reduced-motion suppresses non-essential motion
    When the tab is backgrounded
    Then simulation pauses but WebSocket state continues to synchronize

  Scenario: [ ] E2E-230 Frontend budgets, offline mode and event dedupe hold
    Given a production build
    When the bundle and Lighthouse budgets are measured
    Then initial JS is at most 350 KB gzip, office at most 180 KB, CSS at most 60 KB and performance is at least 90
    And a lost connection shows the last snapshot in read-only mode and queues no mutation
    And replayed state events are deduplicated by event_id and applied by monotonic seq
```

### 21.6 Security and operations

```gherkin
@security @operations @p1 @sec-003
Feature: Security and operations gap closure

  Scenario: [ ] E2E-231 TLS is terminated only by an explicit proxy profile
    Given the optional TLS profile
    When I start the stack with it
    Then TLS terminates at the proxy and the app does not generate self-signed certificates silently
    And the API security headers and HSTS are present when TLS is active

  Scenario: [ ] E2E-232 Sandbox shell and credentials are least-privilege
    Given a run in the sandbox
    When the agent requests a command outside the allowlist, such as curl piped to sh
    Then the command is blocked
    And package installation requires the egress allowlist and is logged
    And only a short-lived repo-scoped token is injected, then revoked after the run
    And no production credential is available inside the sandbox

  Scenario: [ ] E2E-233 The audit log covers sensitive actions and is exportable
    Given provider key create, rotate and delete, agent create and retire, autonomy and budget changes, merges, project connect and disconnect, manual overrides, memory deletion and settings changes
    When each action occurs
    Then an append-only audit row records actor, action, target, safe before and after, trace id and timestamp
    And the audit log can be exported without exposing secrets

  Scenario: [ ] E2E-234 Data export, deletion and retention are honored
    Given a lab with cards, runs, memory vectors, chunks and artifacts
    When the operator exports all data and then deletes the lab with typed confirmation
    Then the export contains the relational data and artifact references
    And deletion removes rows, vectors, chunks and artifacts, is audited and is asynchronous
    And a prompt or response deletion request completes within 24 hours including embeddings

  Scenario: [ ] E2E-235 Configuration and images are hardened and fail fast
    Given an invalid AILAB_* value or a missing master key
    When the process starts
    Then it exits with a clear message and never logs the secret
    And every Compose service has CPU, memory and pid limits, a healthcheck and a pinned image digest
    And the release image has an SBOM and a signature
```

### 21.7 Performance, reliability and acceptance

```gherkin
@performance @reliability @acceptance @p1 @nfr-004
Feature: Performance, reliability and acceptance gap closure

  Scenario: [ ] E2E-236 Latency and duration budgets are met
    Given the defined load profile
    When the API serves reads and writes and a run executes
    Then read p95 is at most 120 ms and p99 at most 300 ms
    And write p95 is at most 250 ms and p99 at most 600 ms
    And DB query p95 is at most 50 ms and non-LLM run overhead is at most 2 seconds
    And a medium SOW completes within 10 minutes p95

  Scenario: [ ] E2E-237 Load, soak and office stress show no leak
    Given the k6 browse, board drag, idea burst, WS fan-out and memory search scenarios
    When they run at twice the expected load for 60 minutes
    Then memory growth stays under 10 percent and the error rate stays under 0.1 percent
    And the office sustains 16 agents and 200 animated props at 60 FPS
    And WS event to render stays at or below 250 ms p95

  Scenario: [ ] E2E-238 Rolling upgrades and clock rules are safe
    Given a running version and a backward-compatible migration
    When a rolling upgrade is performed
    Then the old and new versions interoperate for one version
    And all timestamps are server-generated and clients are never authoritative
    And a simulated deadlock is retried up to three times with jitter

  Scenario: [ ] E2E-239 SLOs, chaos and restore evidence agree
    Given the SLO dashboards, alert rules and restore drill
    When the chaos suite kills a worker, the database and Redis
    Then recovery matches the failure matrix and RTO and RPO targets
    And no alert fires falsely and no data is lost
    And the restore drill evidence is attached to AC-012 and AC-013

  Scenario: [ ] E2E-240 Documentation and release acceptance are complete
    Given the documentation manifest
    When the docs build and link check run
    Then every required document exists, links resolve and screenshots match the deterministic seed
    And the acceptance report has evidence for AC-001 to AC-015
    And no milestone regressed, no P0 or P1 is open, and the release candidate is signed off by a non-author
```

---

## 22. Amiga-Style UI Certification (E2E-241 – E2E-360)

This is the visual certification suite for the 16-bit Amiga/OCS office. It goes beyond functional coverage: it certifies the palette, the pixel grid, integer scaling, sprite timing, truthful state, the interaction model, accessibility, sound, performance, determinism and visual regression. It is designed to be reproducible: every scenario has an observable assertion, a deterministic seed, a frame hash or a pixel diff, and a captured artifact. A release is blocked if any `@p0` certification scenario fails. The target is not "a nice office"; the target is a pixel-perfect, state-truthful, accessible, deterministic, measurable Amiga-style experience with evidence that survives review.

### 22.1 Amiga visual language and palette

```gherkin
@office @visual @p0 @ux-020
Feature: Amiga visual language and palette

  Scenario: [ ] E2E-241 The office uses the fixed internal resolution and tile grid
    Given the office is loaded
    When the engine reports its render model
    Then the internal resolution is exactly 384x216 logical pixels
    And the grid is 24x13 tiles of 16 pixels with a 4-pixel top offset
    And no layout computation uses a fractional tile

  Scenario: [ ] E2E-242 The integer scale factor follows the capitolato formula
    Given viewports of 384x216, 768x432, 1440x900 and 1920x1080
    When the office computes its scale
    Then the scale is floor(min(viewport_width / 384, viewport_height / 216))
    And the scale is always an integer
    And the canvas never uses a fractional multiplier

  Scenario: [ ] E2E-243 Rendering is pixelated and nearest-neighbour
    Given the office canvas is visible
    When the browser computes its rendering style
    Then image-rendering is pixelated
    And no smoothing, bilinear or bicubic filter is applied
    And the sprite atlas is sampled with nearest-neighbour

  Scenario: [ ] E2E-244 The palette is limited to the fixed 32 colours
    Given the office renders a full room with agents and furniture
    When every visible pixel is sampled
    Then every colour belongs to the fixed Amiga palette ramp
    And no colour is introduced by an antialiasing or CSS fallback

  Scenario: [ ] E2E-245 Chrome is beveled with hard pixel edges
    Given panels, buttons and the window strip are visible
    When their computed styles are inspected
    Then borders are 1px or 2px solid
    And shadows are hard pixel offsets with no blur radius
    And radii are limited to 0, 2 or 4 pixels
    And there is no rounded SaaS corner or soft drop shadow

  Scenario: [ ] E2E-246 The scanline texture stays within the opacity budget
    Given the scanline effect is enabled
    When the overlay is measured
    Then its opacity is at most 4 percent
    And the effect is disabled when reduced motion is active
    And disabling it leaves the office legible

  Scenario: [ ] E2E-247 In-canvas typography uses the bitmap pixel font
    Given the office displays a desk plaque and a speech bubble
    When the rendered text is inspected
    Then the canvas text uses the bitmap pixel font at an integer size
    And no emoji is rendered in the office
    And glyph edges remain crisp at every integer scale

  Scenario: [ ] E2E-248 Gradients are dithered from the fixed palette
    Given the office renders a gradient surface
    When the pixels are inspected
    Then the gradient is produced by palette dithering
    And no smooth CSS linear-gradient appears inside the canvas
    And the dither pattern is deterministic

  Scenario: [ ] E2E-249 Status LEDs are colourblind-safe and never colour-only
    Given agents in idle, working, blocked, paused and offline states
    When their desk LEDs are inspected
    Then each LED uses the colourblind-safe accent for its state
    And the desk also exposes a text label and a role icon
    And an operator can distinguish every state without colour vision

  Scenario: [ ] E2E-250 Sprites and chrome have no anti-aliasing
    Given a sprite edge, an icon and a beveled border
    When the rendered pixels are sampled
    Then no intermediate blend colour appears on the edges
    And the sprite alpha is either transparent or palette-exact
    And the browser zoom is 100 percent

  Scenario: [ ] E2E-251 The amiga500 theme applies consistently
    Given the lab office theme is amiga500
    When the office and its DOM overlays render
    Then the canvas and the overlays use the same token palette
    And the window strip, clock, LEDs and sound control use the theme tokens
    And switching the theme persists to the lab settings

  Scenario: [ ] E2E-252 The high-contrast theme preserves the visual language
    Given the high-contrast theme is enabled
    When the office and overlays render
    Then text contrast is at least 4.5 to 1
    And the palette remains within the documented high-contrast ramp
    And the pixel edges, bevels and integer scaling remain intact
```

### 22.2 Pixel-perfect rendering and scaling

```gherkin
@office @visual @p0 @ux-020
Feature: Pixel-perfect rendering and scaling

  Scenario: [ ] E2E-253 Device pixel ratio never changes the logical render
    Given the same office state at DPR 1, 1.5, 2 and 3
    When each render is captured
    Then the canvas buffer is always 384x216 logical pixels
    And only the CSS size changes with the device pixel ratio
    And the logical frame hash is identical across the four captures

  Scenario: [ ] E2E-254 Integer zoom never produces fractional blur
    Given the camera at 1x, 2x, 3x and 4x
    When each zoom level renders
    Then the canvas scale is an integer
    And no pixel is resampled with a fractional offset
    And the edge of every 1-pixel line is a single crisp pixel

  Scenario: [ ] E2E-255 The canvas buffer matches the declared resolution
    Given the office canvas is mounted
    When its width and height attributes are read
    Then the buffer is exactly 384x216
    And the CSS dimensions are an integer multiple of the buffer
    And the canvas is not stretched by a non-integer transform

  Scenario: [ ] E2E-256 Static layers are cached offscreen
    Given the floor, walls and behind-furniture layers do not change
    When the engine renders consecutive frames
    Then the static layers are drawn from offscreen canvases
    And only dynamic layers are redrawn
    And disabling the cache restores the same final frame

  Scenario: [ ] E2E-257 The draw-call budget is respected
    Given the office with sixteen agents and full furniture
    When a frame is profiled
    Then the draw calls are at most 150
    And draws are batched by atlas
    And the draw-call count stays stable across ten frames

  Scenario: [ ] E2E-258 The sprite atlas stays within the memory budget
    Given the office assets are loaded
    When the atlases are decoded
    Then the compressed atlas size is at most 8 MB
    And the decoded memory is reported
    And all critical atlases are ready before the simulation starts

  Scenario: [ ] E2E-259 Every manifest frame resolves
    Given the sprite manifest
    When the engine loads every declared frame
    Then every frame id resolves to a rectangle
    And a deliberately missing frame fails fast with the frame id
    And the loading screen never starts the simulation with a missing critical atlas

  Scenario: [ ] E2E-260 The frame hash is stable for identical inputs
    Given the same event sequence and seed
    When the office reaches the same tick
    Then the frame hash is identical across two runs
    And the hash covers positions, animation frames and z-order
    And a one-event difference changes the hash

  Scenario: [ ] E2E-261 The office is deterministic across viewport changes
    Given the same events at 1280x720 and 1920x1080
    When both runs reach the same tick
    Then the logical positions and frame hash are identical
    And only the integer scale differs

  Scenario: [ ] E2E-262 Visual regression stays under the pixel threshold
    Given a current office screenshot and its golden baseline
    When the visual diff is computed
    Then the changed-pixel ratio is at most 0.1 percent
    And only documented dynamic regions are masked
    And the diff artifact is attached to the result

  Scenario: [ ] E2E-263 Resize and orientation preserve integer scaling
    Given the office at a desktop, tablet and phone width
    When I resize the window or rotate the device
    Then the office reflows without a reload
    And the scale remains an integer
    And no fractional blur or overflow appears

  Scenario: [ ] E2E-264 Fullscreen and fit preserve the pixel grid
    Given the office in windowed mode
    When I enter fullscreen and press fit
    Then the scale is an integer
    And the room is centered with letterboxing where needed
    And no sprite is cropped or stretched
```

### 22.3 Layout, furniture and desks

```gherkin
@office @visual @p1 @ux-001
Feature: Office layout, furniture and desks

  Scenario: [ ] E2E-265 Sixteen agents are visible without overlap
    Given a lab with sixteen active agents
    When the office renders
    Then every agent is visible at a desk, hot desk or standing position
    And no two agents overlap
    And no agent is hidden behind furniture

  Scenario: [ ] E2E-266 Five persistent desks expose identity and state
    Given the default office layout
    When the desks render
    Then five desks are present
    And each plaque shows name, role icon and status LED
    And each desk keeps its position across edits

  Scenario: [ ] E2E-267 Hot desks and standing positions are used when desks are full
    Given more agents than the five permanent desks
    When the office renders
    Then the additional agents occupy hot desks or standing positions
    And the layout remains navigable
    And each hot desk exposes the same inspector interaction

  Scenario: [ ] E2E-268 Furniture participates in collision and pathfinding
    Given a target tile behind a desk or a plant
    When an agent walks to it
    Then the path routes around the furniture footprint
    And the agent never passes through a non-walkable tile
    And the path is recomputed after a layout change

  Scenario: [ ] E2E-269 Every interactive object is clickable
    Given the office is open
    When I click the whiteboard, server rack, bookshelf, filing cabinet, meeting table, door, coffee machine, plant and pet
    Then each object opens or triggers its documented interaction
    And decorative objects only play a cosmetic effect
    And no interactive object is a dead zone

  Scenario: [ ] E2E-270 Y-sorting places characters correctly against furniture
    Given an agent behind a desk and an agent in front of it
    When the frame renders
    Then the agent behind is drawn before the desk
    And the agent in front is drawn after the desk
    And the z-order is stable across frames

  Scenario: [ ] E2E-271 A retired agent leaves an empty desk with a plaque
    Given a retired agent
    When the office renders
    Then the desk is empty
    And the plaque still shows the retired agent identity
    And the door opens the retired-agent list

  Scenario: [ ] E2E-272 Spawning does not teleport an existing agent
    Given a new agent is added
    When it appears in the office
    Then it spawns at a free tile
    And existing agents are not teleported
    And the only allowed instant movement is spawn or crash recovery

  Scenario: [ ] E2E-273 Furniture footprints match the manifest
    Given the furniture manifest
    When each footprint is compared with its hitbox
    Then the footprint size and collision map match
    And there is no invisible blocker outside the footprint
    And no visible furniture is walkable by mistake

  Scenario: [ ] E2E-274 Layout changes invalidate the path cache safely
    Given an agent with a cached path
    When the layout changes
    Then the path cache is invalidated
    And the agent recomputes a valid path
    And no agent remains stuck on a removed tile
```

### 22.4 Characters, sprites and animation

```gherkin
@office @visual @p0 @ux-003
Feature: Characters, sprites and animation

  Scenario: [ ] E2E-275 The character roster is finite and validated
    Given the ten documented character ids
    When an agent is created with each id
    Then every documented id is accepted
    And an unknown id is rejected
    And the office renders the matching sprite set

  Scenario: [ ] E2E-276 Characters expose four directions with mirroring
    Given an agent walking in each direction
    When the sprite frame is selected
    Then down, up, left and right frames exist
    And left may mirror right only when the manifest allows it
    And the mirror flag does not change the hitbox

  Scenario: [ ] E2E-277 Animation frame counts and rates match the manifest
    Given an idle, walking and typing agent
    When the sprite animation advances
    Then idle uses two frames at 2 FPS
    And walk uses four frames at 8 FPS
    And type uses four frames at 12 FPS

  Scenario: [ ] E2E-278 Animation timing is tick-based with no drift
    Given the fixed 60 Hz simulation
    When the office runs for ten minutes
    Then animation timing is derived from ticks, not wall-clock time
    And the frame index does not drift
    And pausing and resuming does not skip a frame incorrectly

  Scenario: [ ] E2E-279 The walk cycle is deterministic
    Given an agent walking for a known elapsed time
    When the walk frame is computed
    Then the frame index is floor(elapsed / 200) modulo 4
    And the same elapsed time always selects the same frame
    And the frame changes only at the expected boundary

  Scenario: [ ] E2E-280 Speech bubbles wrap within the documented limits
    Given a speech message of 120 characters
    When the bubble renders
    Then text wraps at 28 characters per line
    And a bubble has at most three lines
    And the content never exceeds 120 characters

  Scenario: [ ] E2E-281 Speech bubbles auto-dismiss and open details
    Given a speech bubble appears from a real event
    When six seconds elapse
    Then the bubble auto-dismisses
    And clicking it before dismissal opens the agent detail panel
    And the bubble is not replayed after dismissal

  Scenario: [ ] E2E-282 Blocked, paused and offline states are visually distinct
    Given agents that are blocked, paused and offline
    When the office renders
    Then blocked shows the red exclamation and pacing animation
    And paused shows the sleep animation and gray LED
    And offline shows an empty chair with no invented activity

  Scenario: [ ] E2E-283 A merge triggers celebration for the right agent
    Given a delivery.merge.completed event for agent A
    When the office receives the event
    Then agent A plays the celebrate animation and confetti
    And the success chime plays once
    And no other agent celebrates

  Scenario: [ ] E2E-284 A state change stops the previous animation immediately
    Given an agent typing
    When it becomes blocked
    Then the typing animation stops at the next frame
    And the blocked animation starts without a ghost frame
    And the desk LED and label update with it

  Scenario: [ ] E2E-285 Character selection never changes permissions or behaviour
    Given two agents with the same role and different characters
    When they execute the same card
    Then their permissions and tool access are identical
    And only the visual and speech style differ
    And a character/role conflict logs character_override without changing policy

  Scenario: [ ] E2E-286 Every sprite, font and sound asset is licensed
    Given the asset manifest
    When the asset linter runs
    Then every asset has author, source and redistributable license metadata
    And a missing license fails the build
    And no asset is copied from the reference product
```

### 22.5 Truthful state and realtime

```gherkin
@office @realtime @p0 @ux-005
Feature: Truthful state and realtime office mapping

  Scenario: [ ] E2E-287 Domain states map to the truthful animations
    Given agents in idle, working, blocked, paused and offline states
    When the office renders
    Then idle maps to idle, working to type, blocked to blocked and paused or offline to sleep or empty
    And no state shows an animation that is not in the domain

  Scenario: [ ] E2E-288 Ambient life yields to work within one tick
    Given an idle agent walking to the coffee machine
    When a working event arrives
    Then the ambient action yields within one tick
    And the agent returns to the desk and types
    And no work event is visually ignored

  Scenario: [ ] E2E-289 Agent status changes reach the office within the latency budget
    Given a connected office
    When agent.status_changed is emitted
    Then the office reflects the new state within 250 ms p95
    And the desk LED, label and animation agree
    And the event is not invented by the client

  Scenario: [ ] E2E-290 Run steps select the correct working sub-animation
    Given a working agent
    When a plan or llm_request step arrives
    Then the agent shows think
    When a read or search tool_call arrives
    Then the agent shows read
    When a write tool_call arrives
    Then the agent shows type

  Scenario: [ ] E2E-291 Merge completion celebrates the correct office entity
    Given two agents working and one merge event
    When the event is applied
    Then only the merging agent celebrates
    And the card owner is not confused with the merger
    And the event id is deduplicated

  Scenario: [ ] E2E-292 Budget exhaustion shows the lab-wide blocked banner
    Given an active office
    When system.budget.exhausted is emitted
    Then a red lab-wide banner appears
    And active agents show the blocked state
    And the banner clears only when the lab resumes

  Scenario: [ ] E2E-293 Reconnect backfill converges to the same frame hash
    Given a client that disconnects for several events
    When it reconnects and backfills
    Then the office reaches the same state and frame hash as a continuously connected client
    And no duplicate event is applied twice
    And the intermediate states do not flash permanently

  Scenario: [ ] E2E-294 Out-of-order ephemeral events cannot override a newer state
    Given a newer agent state has been applied
    When an older ephemeral typing event arrives
    Then the newer state wins
    And the office does not regress to the older animation
    And the ephemeral event is coalesced

  Scenario: [ ] E2E-295 Duplicate events are deduplicated by id and ordered by seq
    Given a replayed event with an already seen event_id
    When the office applies the stream
    Then the duplicate is ignored
    And the monotonic seq order is preserved
    And no animation is triggered twice

  Scenario: [ ] E2E-296 Offline mode never invents activity
    Given the API is unreachable
    When the office loads the last snapshot
    Then it shows the last truthful state in read-only mode
    And no idle animation invents work
    And a reconnect reconciles before resuming animation
```

### 22.6 Interaction and hitboxes

```gherkin
@office @interaction @p0 @ux-010
Feature: Interaction model and hitboxes

  Scenario: [ ] E2E-297 Clicking an agent opens the Live inspector
    Given the office is open
    When I click an agent
    Then the right-side inspector opens on the Live tab
    And it shows status, current card and the last 20 run steps
    And the selection is visible in the office

  Scenario: [ ] E2E-298 Clicking a desk opens the inspector with desk metadata
    Given the office is open
    When I click a desk
    Then the same inspector opens
    And the desk metadata is shown
    And the selected agent matches the desk

  Scenario: [ ] E2E-299 Inspector tabs are keyboard accessible
    Given the inspector is open
    When I use the keyboard on Live, CV, Memory and Controls
    Then roving tabindex and Arrow, Home and End work
    And Enter and Space activate the tab
    And focus is visible on every tab

  Scenario: [ ] E2E-300 The inspector can be resized and closed safely
    Given the inspector is open
    When I resize it and press Escape
    Then it slides out and closes
    And focus returns to the triggering object
    And the office remains interactive

  Scenario: [ ] E2E-301 Agent controls are functional and audited
    Given the inspector Controls tab
    When I pause, resume, reassign, message, set autonomy or retire an agent
    Then each action succeeds or reports a clear refusal
    And each state change emits an event and an audit row
    And the office reflects the change immediately

  Scenario: [ ] E2E-302 The whiteboard opens Kanban in split view
    Given the office is open
    When I click the whiteboard
    Then the Kanban drawer opens filtered to the project
    And the office remains visible in split view
    And the filter is reflected in the URL or state

  Scenario: [ ] E2E-303 The server rack opens System Health
    Given the office is open
    When I click the server rack
    Then the System Health drawer opens
    And it shows provider, database, Redis, queue, error rate and budget
    And the values match the API health endpoints

  Scenario: [ ] E2E-304 The bookshelf opens the Memory explorer
    Given the office is open
    When I click the bookshelf
    Then the Memory explorer drawer opens
    And it supports scope switching, search, recent documents and add note
    And no cross-scope result is shown

  Scenario: [ ] E2E-305 The filing cabinet opens failed runs and replay
    Given the office is open
    When I click the filing cabinet
    Then the failed-runs and dead-letter drawer opens
    And each failed run shows its error and cost
    And a replay action creates a new attempt without mutating the old run

  Scenario: [ ] E2E-306 The remaining objects open their documented surfaces
    Given the office is open
    When I click the meeting table, door, coffee machine, plant and pet
    Then the meeting table opens the handoffs timeline
    And the door opens retired agents
    And the coffee machine opens idle agents
    And the plant and pet only play a cosmetic effect

  Scenario: [ ] E2E-307 Hover feedback is immediate and pixel-styled
    Given the office is open
    When I hover an interactive object
    Then a pixel tooltip and outline appear within 100 ms
    And the tooltip does not cover the object entirely
    And leaving the object removes the feedback

  Scenario: [ ] E2E-308 Keyboard navigation reaches every office object
    Given the office is open without a mouse
    When I press Tab, Enter, Escape and the arrow keys
    Then Tab cycles through interactive objects in a stable order
    And Enter opens the focused object
    And Escape closes the active drawer
    And the arrow keys pan the camera
```

### 22.7 Camera and deep links

```gherkin
@office @navigation @p1 @ux-017
Feature: Camera and deep links

  Scenario: [ ] E2E-309 The default camera is a fixed full-room view
    Given the office loads
    When the camera initializes
    Then the full room is visible by default
    And the fit button restores the default view
    And resetting the camera does not change the frame hash

  Scenario: [ ] E2E-310 Pan and zoom are available only when needed
    Given the room is smaller than the viewport
    When I use the wheel or drag
    Then panning and zooming are constrained or disabled
    And when the room exceeds the viewport they become available
    And the camera never exposes empty space beyond the room bounds

  Scenario: [ ] E2E-311 Zoom snaps to integer factors
    Given the camera is zoomable
    When I zoom with the wheel or controls
    Then the zoom snaps to 1x, 2x, 3x or 4x
    And no fractional zoom is applied
    And the pixel grid remains crisp at every factor

  Scenario: [ ] E2E-312 The agent deep link restores selection and panel
    Given the URL contains the lab id, an agent id and panel=cv
    When the page loads
    Then the office focuses the agent
    And the inspector opens on the CV tab
    And the state matches a manual selection

  Scenario: [ ] E2E-313 Drawer deep links restore the correct panel
    Given the URL contains panel=health, panel=memory or panel=failed
    When the page loads
    Then the matching drawer opens with the correct data
    And closing it updates the URL without a full reload
    And back and forward restore the previous panel

  Scenario: [ ] E2E-314 The card deep link preserves split view
    Given a deep link with a card id
    When the board opens over the office
    Then the card is highlighted
    And the office remains visible in split view
    And an unknown card id shows a safe not-found state

  Scenario: [ ] E2E-315 Invalid deep-link ids fail safely
    Given a malformed or unknown lab, agent, panel or card id
    When the URL is opened
    Then the app shows a safe not-found or fallback state
    And it never renders a blank screen
    And the error includes a trace id without leaking internals

  Scenario: [ ] E2E-316 Every interactive object exposes a shareable URL
    Given an agent or an object is selected
    When I copy its shareable URL
    Then the URL restores the same selection and panel in a new session
    And the URL contains no secret or token
    And the URL is stable across reloads
```

### 22.8 Responsive layout and input modes

```gherkin
@office @responsive @p1 @ux-030
Feature: Responsive layout and input modes

  Scenario: [ ] E2E-317 Desktop layout expands the sidebar
    Given a viewport at or above 1280 px
    When the office renders
    Then the sidebar is expanded to 240 px
    And the office and the optional inspector fit without overlap
    And the canvas keeps integer scaling

  Scenario: [ ] E2E-318 Tablet layout collapses the sidebar
    Given a viewport between 768 and 1279 px
    When the office renders
    Then the sidebar collapses to 64 px icons
    And the inspector overlays instead of pushing the office
    And every action remains reachable

  Scenario: [ ] E2E-319 Phone layout uses a bottom sheet and full-screen inspector
    Given a viewport below 768 px
    When the office renders
    Then the sidebar becomes a bottom sheet
    And the office scales to fit the width
    And the inspector becomes a full-screen sheet
    And the Kanban becomes a single-column swipe

  Scenario: [ ] E2E-320 Orientation changes reflow without a reload
    Given a device in portrait
    When I rotate it to landscape and back
    Then the layout reflows without a page reload
    And the office state and selection are preserved
    And the scale remains an integer

  Scenario: [ ] E2E-321 Touch targets meet the minimum size
    Given a touch device
    When I inspect every interactive office control
    Then each target is at least 44 by 44 CSS pixels
    And adjacent targets do not overlap
    And the targets remain reachable at 200 percent text zoom

  Scenario: [ ] E2E-322 A keyboard-only user completes the office journey
    Given no pointer device
    When I navigate, select an agent, open the inspector and run a control
    Then every step is reachable
    And focus never disappears
    And the result matches the pointer journey

  Scenario: [ ] E2E-323 Pointer and touch produce the same result
    Given the same object and action
    When I trigger it with a mouse and with touch
    Then the same drawer, selection and state change occur
    And no double event or ghost click is produced

  Scenario: [ ] E2E-324 DOM overlays do not cover the canvas at 200 percent zoom
    Given the browser text zoom is 200 percent
    When the office and overlays render
    Then overlays reflow without covering the selected agent
    And the canvas remains visible and interactive
    And no horizontal scroll is introduced
```

### 22.9 Accessibility and reduced motion

```gherkin
@office @a11y @p0 @ux-034
Feature: Accessibility and reduced motion

  Scenario: [ ] E2E-325 Reduced motion disables walking and ambient motion
    Given prefers-reduced-motion is active
    When the office renders
    Then walking becomes instant reposition with a fade
    And ambient and parallax motion are disabled
    And state changes and speech text remain available

  Scenario: [ ] E2E-326 Reduced motion replaces typing with a static pose
    Given reduced motion and a working agent
    When the office renders
    Then typing is replaced by a static working pose
    And the status label still says working
    And the desk LED is still truthful

  Scenario: [ ] E2E-327 The screen-reader office list is ordered and complete
    Given a screen reader is active
    When I open the office
    Then a hidden but focusable office list exposes agents in a stable order
    And each entry has status, current card and available actions
    And the list matches the visual order

  Scenario: [ ] E2E-328 A live region announces state transitions
    Given the office is open
    When an agent changes state
    Then the live region announces the agent, the old state and the new state
    And the announcement is not duplicated
    And it does not steal focus

  Scenario: [ ] E2E-329 The canvas has an accessible name and fallback
    Given a screen reader or a browser without canvas support
    When the office loads
    Then the canvas has a descriptive accessible name and role
    And the fallback list provides the same information
    And no critical action is canvas-only

  Scenario: [ ] E2E-330 Focus is always visible and trapped correctly
    Given the inspector or a drawer is open
    When I tab through the controls
    Then focus is visible with at least 3 to 1 contrast
    And focus is trapped inside the modal surface
    And closing restores focus to the trigger

  Scenario: [ ] E2E-331 Status is never conveyed by colour alone
    Given every agent state
    When the desk and inspector are inspected
    Then each state has a text label and an icon
    And the LED colour is supplementary
    And a colourblind simulation still distinguishes the states

  Scenario: [ ] E2E-332 Text contrast meets WCAG 2.2 AA
    Given the office DOM overlays and the in-canvas labels
    When contrast is measured
    Then normal text is at least 4.5 to 1
    And large text is at least 3 to 1
    And the high-contrast theme improves rather than weakens contrast

  Scenario: [ ] E2E-333 Automated axe checks find no critical office issues
    Given the office route
    When axe runs with WCAG 2.2 AA rules
    Then there are zero critical or serious violations
    And keyboard and screen-reader checks pass
    And the result is attached to the certification report

  Scenario: [ ] E2E-334 Color-scheme and contrast preferences are respected
    Given prefers-color-scheme and prefers-contrast variations
    When the office renders
    Then the chosen theme remains readable
    And the palette and pixel edges are preserved
    And no preference forces an unsupported colour outside the ramp
```

### 22.10 Sound and ambient life

```gherkin
@office @audio @p1 @ux-027
Feature: Sound and ambient life

  Scenario: [ ] E2E-335 The merge chime plays once and respects mute
    Given sound is enabled
    When a merge completes
    Then the success chime plays once
    And the volume follows the configured level
    And mute prevents the sound without blocking the animation

  Scenario: [ ] E2E-336 The blocked warning respects autoplay rules
    Given a blocked agent and a browser autoplay policy
    When the warning chime would play
    Then the chime plays only after a user gesture when required
    And reduced-motion or mute settings are respected
    And no sound plays for a historical event on page load

  Scenario: [ ] E2E-337 Ambient actions happen only while idle
    Given an idle agent near the coffee machine
    When ambient life runs
    Then coffee trips, plant sway and pet or fish motion occur only for idle agents
    And a work event yields the ambient action within one tick
    And the ambient action never blocks a desk

  Scenario: [ ] E2E-338 Ambient actions never change domain state
    Given an ambient coffee trip
    When the action completes
    Then no card, run, agent status or audit row changes
    And the office returns the agent to its previous idle state
    And the frame hash returns to the expected idle sequence

  Scenario: [ ] E2E-339 The sound preference persists safely
    Given the operator changes volume or mute in settings
    When the lab reloads
    Then the preference persists for the lab
    And it is stored without a secret
    And the default respects the platform autoplay policy

  Scenario: [ ] E2E-340 Office audio and visuals contain no unlicensed asset
    Given the office sound and sprite manifests
    When the asset linter runs
    Then every sound and sprite has license metadata
    And no emoji is used in the office
    And a missing or non-redistributable asset fails the build
```

### 22.11 Performance, memory and determinism

```gherkin
@office @performance @p0 @ux-025
Feature: Office performance, memory and determinism

  Scenario: [ ] E2E-341 Desktop frame rate meets the target
    Given the office at 1440p with sixteen agents and 200 animated props
    When it runs for five minutes
    Then the FPS is at least 58
    And no frame exceeds the long-task budget repeatedly
    And the FPS samples are attached to the evidence

  Scenario: [ ] E2E-342 The low-end profile meets the minimum frame rate
    Given the low-end profile with an integrated GPU and reduced effects
    When the office runs
    Then the FPS is at least 30
    And the visual state remains truthful
    And the quality auto-tune does not change the logical frame hash

  Scenario: [ ] E2E-343 The frame budget is respected
    Given a steady office
    When frame durations are measured
    Then the p95 frame time is at most 16.6 ms
    And no recurring long task exceeds 50 ms during idle
    And the frame budget is reported per layer

  Scenario: [ ] E2E-344 Peak heap and long-run memory stay bounded
    Given a ten-minute office soak
    When memory is sampled
    Then peak JS heap is at most 150 MB
    And memory growth is under 10 percent
    And no unbounded particle or bubble allocation is observed

  Scenario: [ ] E2E-345 Draw calls and atlas batching stay stable
    Given the full office scene
    When ten consecutive frames are profiled
    Then draw calls stay at or below 150
    And the atlas batch count is stable
    And no per-frame atlas rebuild occurs

  Scenario: [ ] E2E-346 A background tab pauses simulation but keeps state in sync
    Given the office is visible
    When the tab is backgrounded
    Then the simulation pauses
    And the WebSocket state keeps synchronizing
    And returning to the tab does not replay thousands of frames

  Scenario: [ ] E2E-347 Resuming a background tab catches up without teleporting
    Given a backgrounded tab and new events
    When I return to the tab
    Then the office catches up to the latest state
    And agents do not teleport except through valid movement
    And the frame hash matches a continuously visible client

  Scenario: [ ] E2E-348 Determinism survives repeated runs
    Given the same seed and event sequence
    When the office runs three times
    Then the final positions, animation frames and z-order are identical
    And the frame hash is identical
    And a single event difference changes the hash

  Scenario: [ ] E2E-349 The optional WebGL2 backend is pixel-identical
    Given the WebGL2 backend is enabled
    When the same frame renders in Canvas 2D and WebGL2
    Then the pixel diff is zero or within the declared tolerance
    And the palette, edges and integer scaling match
    And disabling WebGL2 restores the Canvas 2D reference

  Scenario: [ ] E2E-350 The office stress test holds for one hour
    Given sixteen agents and 200 animated props
    When the office runs for 60 minutes
    Then the FPS target is sustained
    And memory growth is under 10 percent
    And no visual state diverges
    And the evidence is attached to the certification report
```

### 22.12 Visual regression and certification evidence

```gherkin
@office @visual @certification @p0 @ux-027
Feature: Visual regression and certification evidence

  Scenario: [ ] E2E-351 Golden screenshots cover the visual matrix
    Given the deterministic office seed
    When I capture 1x, 2x and 3x at DPR 1 and 2, with reduced motion on and off
    Then every capture exists in the golden set
    And each baseline is generated from the seed, not edited by hand
    And the capture metadata records viewport, DPR and theme

  Scenario: [ ] E2E-352 Pixel diffs respect the threshold and masks
    Given a current capture and its golden baseline
    When the diff is computed
    Then the changed-pixel ratio is at most 0.1 percent
    And only documented dynamic regions are masked
    And the diff image and the mask definition are attached

  Scenario: [ ] E2E-353 The palette audit proves the 32-colour limit
    Given a full office capture
    When every pixel is sampled
    Then every colour belongs to the fixed palette ramp
    And the distinct-colour count is at most 32 per atlas
    And any off-palette colour fails the certification

  Scenario: [ ] E2E-354 The anti-aliasing audit proves crisp edges
    Given sprite, icon and bevel captures
    When edge pixels are sampled
    Then there is no partial blend colour
    And alpha is transparent or palette-exact
    And the audit report lists the sampled edges

  Scenario: [ ] E2E-355 The scanline audit proves the opacity budget
    Given the scanline effect on and off
    When the overlay pixels are measured
    Then the effect opacity is at most 4 percent
    And it is disabled under reduced motion
    And the underlying palette remains unchanged

  Scenario: [ ] E2E-356 Z-order golden frames match the expected composition
    Given agents behind, in front of and beside furniture
    When the golden frame is compared
    Then the y-sorted z-order matches the expected composition
    And no agent is drawn through a desk or wall
    And the frame hash is recorded

  Scenario: [ ] E2E-357 The visual baseline is reproducible from the seed
    Given a clean environment and the deterministic seed
    When the baseline is regenerated
    Then it is byte-identical to the checked-in golden set
    And no manual image edit is required
    And a changed seed produces a different but explainable baseline

  Scenario: [ ] E2E-358 Every office scenario attaches evidence
    Given an executed office scenario
    When it completes
    Then it attaches a frame hash, a screenshot, an FPS sample and the relevant logs
    And the artifact paths are referenced by the E2E id
    And secrets and personal data are redacted

  Scenario: [ ] E2E-359 The certification report aggregates every dimension
    Given the office certification run
    When the report is generated
    Then it includes palette, scaling, anti-aliasing, scanline, z-order, a11y, FPS, memory, determinism and visual diff
    And every dimension has a pass or fail with evidence
    And a single failed dimension blocks certification

  Scenario: [ ] E2E-360 The Amiga-style certification suite is release-blocking
    Given the complete office certification suite
    When the release candidate is evaluated
    Then every `@p0` certification scenario passes
    And visual, accessibility, performance and determinism evidence is attached
    And any regression blocks the release with the failing E2E ids
    And the certified result is recorded as a reproducible artifact
```

---

## 23. Inventory and Lab Management UI Certification (E2E-361 – E2E-480)

This section certifies the management surface of the product: the inventory of agents, projects and providers, and the lab settings, team and policy management. The bar is not "it works"; the bar is clarity, hierarchy, calm, responsiveness, accessibility and delight — the quality of a product designed with the focus and taste of Steve Jobs in his prime. Every scenario asserts an observable quality that a reviewer can see, measure or reproduce, and the final Jobs-quality certification is release-blocking.

### 23.1 Design quality and visual hierarchy

```gherkin
@inventory @lab @ui @visual @p0 @ux-060
Feature: Management design quality and visual hierarchy

  Scenario: [ ] E2E-361 The management shell uses the pixel design tokens consistently
    Given I open the agents, projects, providers and settings routes
    When the computed styles are inspected
    Then colours, spacing, borders, radii, shadows and typography come from the design tokens
    And no feature component hardcodes a colour or spacing value
    And the same primitive looks identical on every management route

  Scenario: [ ] E2E-362 Every management screen has one clear primary action
    Given the agents, projects, providers and settings screens
    When the visual hierarchy is inspected
    Then each screen has exactly one primary action
    And secondary actions are visually subordinate
    And the page title, subtitle and primary action form a clear reading order

  Scenario: [ ] E2E-363 The chrome rejects the generic SaaS look
    Given any management surface
    When borders, radii and shadows are measured
    Then radii are limited to 0, 2 or 4 pixels
    And shadows are hard pixel offsets with no blur
    And borders are crisp 1px or 2px lines
    And there is no soft rounded card or floating glass effect

  Scenario: [ ] E2E-364 The spacing rhythm is consistent
    Given a management screen with forms, tables and drawers
    When the vertical and horizontal gaps are measured
    Then every spacing value is a multiple of the 4px scale
    And related elements are closer than unrelated groups
    And no gap looks accidental or cramped

  Scenario: [ ] E2E-365 Icons and labels are consistent and crisp
    Given buttons, tables, menus and tabs
    When their icons are inspected
    Then every icon is from the 16x16 pixel set
    And icons use crisp edges without anti-aliasing
    And every icon has a label or an accessible name that matches its meaning

  Scenario: [ ] E2E-366 Status is never conveyed by colour alone
    Given agents, projects and providers in every status
    When the badges and rows are inspected
    Then each status has a text label, an icon and a tone
    And a colourblind simulation still distinguishes the status
    And the same status looks the same across every list

  Scenario: [ ] E2E-367 Focus is visible, consistent and never lost
    Given I navigate a management screen with the keyboard
    When focus moves through links, inputs, buttons, tabs and rows
    Then every focused element has a visible focus ring
    And the ring meets contrast requirements
    And focus is never hidden behind a sticky element

  Scenario: [ ] E2E-368 Text contrast meets WCAG 2.2 AA
    Given the management routes with normal and large text
    When contrast is measured
    Then normal text is at least 4.5 to 1
    And large text is at least 3 to 1
    And disabled text remains readable and is not mistaken for enabled text

  Scenario: [ ] E2E-369 Loading does not cause layout shift
    Given a slow management list load
    When data is replaced by skeletons and then by rows
    Then the skeleton reserves the final height
    And the cumulative layout shift is at most 0.1
    And the page does not jump under the pointer

  Scenario: [ ] E2E-370 Motion is restrained and purposeful
    Given drawers, toasts, tabs and menus
    When they open and close
    Then durations use the 80, 160 or 240 ms tokens
    And pixel effects use steps instead of smooth easing
    And reduced motion removes non-essential animation
    And no animation delays an action by more than 240 ms

  Scenario: [ ] E2E-371 Every action exposes hover, press, disabled and loading states
    Given every interactive control on the management routes
    When it is hovered, pressed, disabled or loading
    Then the state is visually distinct
    And the control remains readable
    And a loading control prevents a second submit

  Scenario: [ ] E2E-372 Destructive actions look and behave as destructive
    Given archive, delete, retire and reset controls
    When they are rendered and triggered
    Then they use the danger tone and a consequence-focused explanation
    And a typed confirmation is required where the specification demands it
    And the cancel path is safe and always visible
```

### 23.2 Management shell, navigation and search

```gherkin
@inventory @lab @navigation @p1 @ux-060
Feature: Management shell and navigation

  Scenario: [ ] E2E-373 Every management route is reachable from the shell
    Given the application shell
    When I use the sidebar and the in-page navigation
    Then agents, projects, providers and settings are reachable
    And the active route is highlighted
    And no route ends in a dead end

  Scenario: [ ] E2E-374 Search finds the entity by its visible fields
    Given agents, projects and providers with known names
    When I type a name, slug, role or repo into the list search
    Then the matching rows remain
    And the result count is visible
    And clearing the search restores the full list

  Scenario: [ ] E2E-375 Breadcrumbs reflect the hierarchy
    Given an entity detail open from a list
    When the breadcrumbs render
    Then they show the route from the management area to the entity
    And the current item is marked with aria-current
    And each parent crumb is keyboard reachable

  Scenario: [ ] E2E-376 Back and forward restore the list state
    Given a list with a search, a filter and a page cursor
    When I open a detail and then use browser back and forward
    Then the search, filter, cursor and drawer state are restored
    And no full page reload is required

  Scenario: [ ] E2E-377 Deep-linkable UI state is stable
    Given a management URL with a selected agent, panel or card query state
    When I open it in a new session
    Then the same entity and drawer are restored
    And the URL contains no secret or personal data
    And an invalid state falls back safely

  Scenario: [ ] E2E-378 Empty management states always offer a next step
    Given an empty agents, projects or providers inventory
    When the screen renders
    Then an illustrated empty state explains what is missing
    And exactly one clear CTA creates the first entity
    And there is no dead end or disabled primary action

  Scenario: [ ] E2E-379 Loading and error are never fake data
    Given a slow or failing management API
    When the screen renders
    Then it shows a loading skeleton or an error card with retry and trace id
    And it never shows zeros or stale rows as if they were current
    And retry restores the ready state

  Scenario: [ ] E2E-380 Invalid or missing entities fail safely
    Given an invalid or deleted agent, project, provider or lab id
    When I open the route or drawer
    Then a safe not-found state appears
    And the user can return to the list
    And the screen never goes blank

  Scenario: [ ] E2E-381 View in office works from the inventory
    Given an agent, project or lab row
    When I trigger view in office
    Then the office opens focused on the entity
    And the selection is shareable
    And returning to inventory restores the list state

  Scenario: [ ] E2E-382 Navigation is consistent across pointer and keyboard
    Given the same management journey with a mouse and with the keyboard
    When I navigate, search, filter, open a drawer and submit a form
    Then the same result occurs
    And no action is pointer-only
    And focus is restored to the trigger after closing a drawer
```

### 23.3 Agents inventory

```gherkin
@inventory @agents @ui @p0 @ux-060
Feature: Agents inventory and lifecycle UI

  Scenario: [ ] E2E-383 The agents list shows the information needed to decide
    Given a lab with several agents
    When the agents list renders
    Then each row shows name, role, character avatar, status, capacity, CV version and last activity
    And the columns are aligned and readable
    And the primary create action is visible

  Scenario: [ ] E2E-384 Agents can be searched by name, role and character
    Given agents with distinct names, roles and characters
    When I search by each field
    Then only the matching agents remain
    And the search is case-insensitive and debounced
    And the result count updates without a full reload

  Scenario: [ ] E2E-385 Agents can be filtered by role and status
    Given agents in every role and status
    When I filter by role and by status
    Then the list contains only matching agents
    And the active filters are visible
    And clearing a filter restores the matching rows

  Scenario: [ ] E2E-386 Agents can be sorted deterministically
    Given a list of agents
    When I sort by name, role, status or last activity
    Then the order is stable and correct
    And the active sort is indicated with aria-sort
    And reloading preserves the sort

  Scenario: [ ] E2E-387 Agent pagination is clear and reversible
    Given more agents than one page
    When I move to the next and previous page
    Then the boundary controls are enabled and disabled correctly
    And the page cursor is reflected in the URL
    And returning to the first page restores the first rows

  Scenario: [ ] E2E-388 The agent detail drawer explains the agent
    Given an agent row
    When I open its detail
    Then the drawer shows description, role, character, model preference, autonomy, capacity and CV summary
    And the edit and lifecycle actions are clear
    And the drawer traps focus and closes with Escape

  Scenario: [ ] E2E-389 The agent creation wizard follows the documented steps
    Given the create-agent action
    When I open the wizard
    Then the steps are role, character, name, model, autonomy and review
    And Back and Next preserve the entered data
    And the review step summarises every choice before creation

  Scenario: [ ] E2E-390 The character picker is visual and valid
    Given the character step
    When I inspect the roster
    Then each character shows an animated preview and its display name
    And selecting one highlights it
    And an unknown character id cannot be submitted

  Scenario: [ ] E2E-391 Agent names are unique and validated inline
    Given the name step
    When I submit an empty name or a duplicate name within the lab
    Then an inline error explains the problem and keeps my input
    And a valid unique name advances the wizard

  Scenario: [ ] E2E-392 The agent description teaches the limit
    Given the description field
    When I type fewer than 120 or more than 600 characters
    Then a live counter and an inline message explain the range
    And the submit is blocked with a clear reason
    And a valid description is accepted

  Scenario: [ ] E2E-393 Agent capacity, model and autonomy are constrained
    Given the model and autonomy steps
    When I set capacity outside 1 to 4 or choose an unsupported model
    Then the control rejects the value with an explanation
    And valid choices are preserved in the review

  Scenario: [ ] E2E-394 A created agent appears everywhere immediately
    Given a completed agent wizard
    When I submit it
    Then the agent appears in the list and the lab team
    And a success toast confirms the creation
    And an inventory event and audit row are recorded
    And view in office resolves the new agent

  Scenario: [ ] E2E-395 Editing an agent is safe and reversible
    Given an existing agent
    When I edit its description, model, autonomy or capacity and save
    Then the detail and list reflect the change
    And the form validates before saving
    And cancel discards the change without altering the agent

  Scenario: [ ] E2E-396 Pause, resume and retire are clear and safe
    Given an active agent
    When I pause, resume and retire it from the detail or list
    Then each action shows its consequence and requires confirmation where needed
    And a retired agent stays visible with its CV and desk
    And a retired agent cannot receive new cards
    And every action is audited
```

### 23.4 Projects inventory and GitHub connection

```gherkin
@inventory @projects @ui @p0 @ux-060
Feature: Projects inventory and GitHub connection UI

  Scenario: [ ] E2E-397 The projects list explains each project at a glance
    Given several projects
    When the list renders
    Then each row shows name, slug, key prefix, repository, default branch, status and lab
    And the visual hierarchy makes the project name the anchor
    And the create action is obvious

  Scenario: [ ] E2E-398 Projects can be searched, filtered and sorted
    Given projects with distinct names, statuses and labs
    When I search, filter by status or lab, and sort
    Then the list updates deterministically
    And the active state is visible and reflected in the URL
    And an empty filter result offers a clear reset

  Scenario: [ ] E2E-399 The project creation wizard is complete
    Given the create-project action
    When I open the wizard
    Then the steps cover name, slug, key prefix, lab, SCM credential, repository and default branch
    And the review step summarises the choices
    And I cannot submit an incomplete project

  Scenario: [ ] E2E-400 Slug and key prefix are validated inline
    Given the slug and prefix fields
    When I enter a duplicate slug, a duplicate prefix, a malformed prefix or an invalid character
    Then an inline error explains the rule
    And a valid slug and prefix are accepted
    And the uniqueness check is scoped to the lab

  Scenario: [ ] E2E-401 The SCM credential is selected and verified
    Given the repository step
    When I choose an SCM credential
    Then the token is never displayed
    And the connection can be verified
    And an invalid or expired credential produces an actionable error

  Scenario: [ ] E2E-402 The project detail is a control panel
    Given a project detail
    When it renders
    Then it shows repository metadata, sync status, last sync, board, SOW and plan links
    And the primary next action is clear
    And the page does not overwhelm with secondary actions

  Scenario: [ ] E2E-403 Project sync reports progress and recovers
    Given a connected project
    When I trigger sync
    Then progress and result are visible
    And a failure shows the reason and a retry
    And a successful sync updates the repository metadata

  Scenario: [ ] E2E-404 Archiving a project is deliberate and reversible in the UI
    Given an active project
    When I archive it with the typed confirmation
    Then it leaves the active list and remains available in the archived filter
    And new cards are blocked with a clear message
    And the action is audited

  Scenario: [ ] E2E-405 Repository and credential errors never become dead ends
    Given an archived, empty or inaccessible repository or a bad credential
    When the project step validates it
    Then the error explains the exact problem and how to fix it
    And the user can change the repository or credential without losing the form
    And no silent failure occurs

  Scenario: [ ] E2E-406 A project links to its board, SOW and plan
    Given a project detail
    When I follow the board, SOW and plan links
    Then the matching route opens with the project context
    And returning to the project preserves the shell state
    And the links are keyboard reachable
```

### 23.5 Providers inventory and write-only key

```gherkin
@inventory @providers @security @ui @p0 @ux-060
Feature: Providers inventory and write-only key UI

  Scenario: [ ] E2E-407 The providers list is readable and operational
    Given several providers
    When the list renders
    Then each row shows name, kind, base URL host, status, last4, last health and rate limits
    And the status and last4 are immediately legible
    And the create action is obvious

  Scenario: [ ] E2E-408 The provider creation wizard covers the contract
    Given the create-provider action
    When I open the wizard
    Then the fields cover name, kind, base URL, key, rate limits and timeout
    And the key is a write-only input
    And I cannot submit an invalid provider

  Scenario: [ ] E2E-409 The API key is never echoed or recoverable
    Given the provider key field and a saved provider
    When I type, save and reopen the provider
    Then the full key is never shown in the DOM, network response, toast or console
    And only the last four characters are displayed
    And the field is cleared after submit

  Scenario: [ ] E2E-410 Provider fields are validated inline
    Given the provider form
    When I submit a non-HTTPS URL, a duplicate name, a non-positive limit or a key outside the allowed length
    Then an inline error explains the rule
    And a valid provider is accepted
    And the validation happens before any external call

  Scenario: [ ] E2E-411 Provider verification shows actionable states
    Given a provider
    When I verify it and the upstream returns success, degraded, timeout or unauthorized
    Then each state is visible with a clear message
    And the provider status updates accordingly
    And a failure offers a retry or an edit action

  Scenario: [ ] E2E-412 Key rotation is hidden, confirmed and audited
    Given an existing provider
    When I rotate its key
    Then the new key is entered through a hidden prompt
    And the old last4 is replaced only after a successful save
    And the audit row records the rotation without the key

  Scenario: [ ] E2E-413 Disabling and enabling a provider is deliberate
    Given an active provider
    When I disable and then enable it
    Then the status changes and is visible in the list and detail
    And a confirmation explains the impact on runs
    And the action is audited

  Scenario: [ ] E2E-414 Provider health detail is understandable
    Given a provider with a health history
    When I open its detail
    Then the last health time and detail are shown in plain language
    And the difference between degraded and disabled is explained
    And the next action is clear

  Scenario: [ ] E2E-415 Provider errors are actionable, not alarming
    Given an invalid key, a 401, a degraded endpoint or a timeout
    When the error is displayed
    Then the message states what happened and what to do next
    And it uses the precise operational tone with no exclamation marks
    And the provider form remains editable

  Scenario: [ ] E2E-416 No secret leaks through any management surface
    Given a provider key was entered
    When I inspect the DOM, network, logs, toasts, errors and traces
    Then no plaintext key appears
    And the redactor covers the key shape
    And the browser history and URL contain no key
```

### 23.6 Lab settings, team and policy

```gherkin
@lab @settings @ui @p0 @ux-060
Feature: Lab settings, team and policy UI

  Scenario: [ ] E2E-417 The settings page is grouped into clear sections
    Given the lab settings route
    When the page renders
    Then the sections cover lab, autonomy, budget, scheduler, office and danger zone
    And each section has a clear heading and description
    And the primary save action is obvious

  Scenario: [ ] E2E-418 Lab identity is editable and validated
    Given the lab section
    When I change the name, slug or description
    Then the fields validate inline
    And saving updates the lab and its shell label
    And cancel discards the change

  Scenario: [ ] E2E-419 Autonomy levels are explained before selection
    Given the autonomy section
    When I inspect A0, A1, A2 and A3
    Then each level explains what agents may do and what requires a human
    And the current level is selected
    And a change is confirmed with its impact

  Scenario: [ ] E2E-420 A3 requires an explicit opt-in and a hard budget
    Given the autonomy section
    When I select A3 without an explicit opt-in or a hard monthly budget
    Then the save is blocked with a clear explanation
    And the opt-in and budget fields are revealed
    And a valid A3 configuration saves and is audited

  Scenario: [ ] E2E-421 Budget limits are coherent
    Given the budget section
    When I set a per-card budget greater than the per-day budget or a negative value
    Then an inline error explains the relationship
    And a valid pair with a hard stop saves
    And the effective limits are visible after saving

  Scenario: [ ] E2E-422 Scheduler settings are bounded and understandable
    Given the scheduler section
    When I edit the tick, lease TTL or retry count outside the allowed range
    Then the control rejects the value with an explanation
    And valid values save
    And the defaults are one click away

  Scenario: [ ] E2E-423 Office settings preview their effect
    Given the office section
    When I change the theme, sound or reduced motion
    Then a preview reflects the choice immediately
    And saving persists it to the lab
    And reduced motion removes non-essential animation

  Scenario: [ ] E2E-424 WIP limits are validated as a pair
    Given the WIP section
    When I set the DOING limit or the REVIEW limit outside its range
    Then an inline error explains the limit
    And valid limits save
    And the current limits are visible in the scheduler context

  Scenario: [ ] E2E-425 The team always has exactly one active Tech Lead
    Given the team section
    When I add, remove or reassign roles
    Then the one-Tech-Lead rule is enforced with a clear message
    And an empty Tech Lead cannot be saved
    And a valid team composition saves

  Scenario: [ ] E2E-426 Removing the last Tech Lead is blocked
    Given a lab with one Tech Lead
    When I try to remove or retire that agent
    Then the action is refused with an explanation
    And the team remains valid
    And the UI suggests adding a replacement

  Scenario: [ ] E2E-427 Adding an agent from settings reuses the wizard
    Given the team section
    When I add an agent
    Then the same validated agent wizard opens
    And the role is compatible with the lab
    And the new agent appears in the team and inventory

  Scenario: [ ] E2E-428 Saving settings is optimistic and recoverable
    Given a valid settings change
    When I save
    Then the UI applies the change immediately
    And a server validation error rolls it back with the field error
    And a success toast confirms the save

  Scenario: [ ] E2E-429 Resetting settings requires confirmation
    Given changed settings
    When I reset to defaults
    Then a confirmation explains what will be restored
    And cancelling leaves the values unchanged
    And confirming restores and audits the defaults

  Scenario: [ ] E2E-430 The danger zone is explicit and safe
    Given the danger zone
    When I archive, export or delete the lab
    Then each action explains its consequence
    And archive and delete require typed confirmation
    And the result is audited and the UI returns to a safe state
```

### 23.7 Wizards, forms and validation

```gherkin
@inventory @lab @ui @forms @p1 @ux-061
Feature: Management wizards, forms and validation

  Scenario: [ ] E2E-431 Wizard progress is visible and honest
    Given any management wizard
    When I advance through the steps
    Then the current step, completed steps and remaining steps are visible
    And the progress indicator is not clickable ahead of validation
    And the final review is reachable only when the steps are valid

  Scenario: [ ] E2E-432 Back and cancel preserve or discard deliberately
    Given a partially completed wizard
    When I go back
    Then my entries are preserved
    When I cancel with unsaved changes
    Then I am asked to confirm the discard
    And confirming returns me to the list unchanged

  Scenario: [ ] E2E-433 Validation is inline and summarised
    Given a form with an invalid field
    When I blur the field and submit
    Then an inline message is linked with aria-describedby
    And a summary lists the errors and focuses the first invalid field
    And the submit is blocked with a clear reason

  Scenario: [ ] E2E-434 Required and optional fields are unambiguous
    Given every management form
    When I inspect the labels
    Then required fields are marked in text, not colour alone
    And optional fields are clearly optional
    And placeholders never replace a label

  Scenario: [ ] E2E-435 Every wizard is completable by keyboard
    Given a management wizard
    When I use only the keyboard
    Then I can move through every step, select, type and submit
    And Enter submits and Escape cancels with confirmation
    And focus is never trapped outside the active surface

  Scenario: [ ] E2E-436 Drawers and dialogs manage focus correctly
    Given a drawer or dialog
    When it opens and closes
    Then focus moves inside it, is trapped, and returns to the trigger
    And the surface has aria-modal and an accessible name
    And Escape closes it without losing data unexpectedly

  Scenario: [ ] E2E-437 Leaving a dirty form is guarded
    Given a form with unsaved changes
    When I navigate away or close the drawer
    Then a confirmation explains the unsaved change
    And cancelling keeps me in the form
    And confirming discards and navigates

  Scenario: [ ] E2E-438 Submit loading prevents duplicates
    Given a valid form
    When I submit it
    Then the primary action enters a loading state and is disabled
    And a second submit is ignored
    And success or error resolves the state with a toast

  Scenario: [ ] E2E-439 Server violations map to the fields
    Given a server 422 response with violations
    When the form receives it
    Then each violation is attached to its field
    And a summary names the problem in plain language
    And the entered data is preserved

  Scenario: [ ] E2E-440 Long content never breaks the layout
    Given very long names, descriptions, URLs and labels
    When the list, detail and form render
    Then text wraps or truncates with an accessible full value
    And no action is pushed off screen
    And the layout remains aligned at every breakpoint
```

### 23.8 Tables, filters, pagination and states

```gherkin
@inventory @lab @ui @tables @p1 @fe-008
Feature: Management tables, filters and states

  Scenario: [ ] E2E-441 Tables are semantic and accessible
    Given the agents, projects and providers tables
    When they render
    Then each table has a caption and column headers
    And sortable headers expose aria-sort
    And row actions have accessible names

  Scenario: [ ] E2E-442 Columns adapt without losing meaning
    Given a management table at desktop, tablet and phone widths
    When the viewport changes
    Then non-critical columns collapse or move into a row detail
    And the primary identity and action remain visible
    And no critical value is hidden without access

  Scenario: [ ] E2E-443 Filters are visible, resettable and persistent
    Given a list with search and filters
    When I apply filters and reload
    Then the filters are visible as chips or controls
    And a reset action clears them
    And the URL preserves the filter state

  Scenario: [ ] E2E-444 Empty filter results are recoverable
    Given a filter that matches nothing
    When the result renders
    Then an empty result state explains the filter
    And a clear-filters action is present
    And the user can recover without retyping

  Scenario: [ ] E2E-445 Loading skeletons preserve the final layout
    Given a slow list request
    When the skeleton renders
    Then rows and columns reserve the final space
    And the layout shift is minimal
    And the ready state does not jump

  Scenario: [ ] E2E-446 List errors are actionable
    Given a failing list request
    When the error state renders
    Then it shows a plain-language message and a retry
    And the trace id is available
    And the rest of the shell remains usable

  Scenario: [ ] E2E-447 Pagination boundaries are correct
    Given the first, middle and last page
    When I navigate
    Then previous and next are disabled at the correct boundaries
    And the page size is respected
    And the cursor is stable across reloads

  Scenario: [ ] E2E-448 Row actions are keyboard and pointer equivalent
    Given a table row
    When I open its actions with a mouse, keyboard or touch
    Then the same menu opens
    And arrow, Home, End and Escape work
    And the action result is identical

  Scenario: [ ] E2E-449 Narrow screens have an explicit table strategy
    Given a phone width
    When a management table renders
    Then it becomes cards or an intentional scroll region
    And the affordance is visible
    And no action is unreachable

  Scenario: [ ] E2E-450 Filters and sorting never lie
    Given a filtered and sorted list
    When I inspect the rows
    Then every row satisfies the active filter
    And the order matches the active sort
    And the result count matches the rendered rows
```

### 23.9 Responsive management and touch

```gherkin
@inventory @lab @responsive @p1 @ux-030
Feature: Responsive management and touch

  Scenario: [ ] E2E-451 Desktop management layout is calm and readable
    Given a viewport at or above 1280 px
    When a management screen renders
    Then the sidebar is expanded
    And the content has a comfortable line length and clear gutters
    And the primary action is not lost in the chrome

  Scenario: [ ] E2E-452 Tablet management uses collapsible navigation
    Given a viewport between 768 and 1279 px
    When a management screen renders
    Then the sidebar collapses to icons
    And drawers overlay the content
    And every action remains reachable

  Scenario: [ ] E2E-453 Phone management becomes a focused single column
    Given a viewport below 768 px
    When a management screen renders
    Then the sidebar becomes a bottom sheet
    And lists become single-column cards or an intentional scroll region
    And drawers become full-screen sheets
    And the primary action stays visible

  Scenario: [ ] E2E-454 Orientation changes preserve management state
    Given a completed filter and an open detail
    When I rotate the device
    Then the layout reflows without a reload
    And the filter and detail are preserved
    And focus is not lost

  Scenario: [ ] E2E-455 Touch targets are comfortable
    Given a touch device
    When I inspect management controls
    Then each target is at least 44 by 44 CSS pixels
    And adjacent targets do not overlap
    And no action depends on hover

  Scenario: [ ] E2E-456 Text zoom does not break the management UI
    Given the browser text zoom is 200 percent
    When a management screen renders
    Then text does not overlap or clip
    And critical actions remain visible
    And the layout remains usable without horizontal scrolling

  Scenario: [ ] E2E-457 Long values remain accessible on small screens
    Given long names, repository URLs and descriptions
    When the phone layout renders
    Then values wrap or truncate with an accessible full value
    And no row action is pushed off screen
    And the visual hierarchy remains intact

  Scenario: [ ] E2E-458 Responsive management has no dead ends
    Given every management route at desktop, tablet and phone widths
    When I navigate and act
    Then every screen has a reachable primary action
    And empty, loading and error states remain actionable
    And no breakpoint hides the only path forward
```

### 23.10 Usability, onboarding and microcopy

```gherkin
@inventory @lab @usability @p0 @ux-041
Feature: Usability, onboarding and microcopy

  Scenario: [ ] E2E-459 The first-run onboarding checklist is complete
    Given a fresh installation
    When the dashboard renders
    Then the checklist covers provider, key, lab, Tech Lead, GitHub, project and idea
    And each step shows its state
    And the next incomplete step is the primary action

  Scenario: [ ] E2E-460 Every onboarding step deep-links to its task
    Given an onboarding step
    When I open it
    Then the exact form or route opens
    And completing the task returns me to the checklist with the step marked done
    And I never have to hunt for the next step

  Scenario: [ ] E2E-461 Microcopy is concise, precise and human
    Given management forms, empty states and errors
    When I read the copy
    Then it states what happened and what to do next
    And errors use the operational tone with no exclamation marks
    And no marketing fluff appears in operational panels

  Scenario: [ ] E2E-462 Empty states have one unmistakable next action
    Given every management empty state
    When it renders
    Then it explains the missing entity in one sentence
    And exactly one primary CTA is offered
    And the illustration supports the action instead of competing with it

  Scenario: [ ] E2E-463 Toasts confirm without stealing the flow
    Given a successful or failed management action
    When the toast appears
    Then it is announced to assistive technology
    And it auto-dismisses at a readable pace
    And it never covers the primary action or the focused field

  Scenario: [ ] E2E-464 Destructive confirmations explain the consequence
    Given archive, delete, retire or reset
    When the confirmation opens
    Then it names the entity and the consequence
    And the destructive action is visually distinct
    And the safe action is the easy default

  Scenario: [ ] E2E-465 The UI teaches the domain without a manual
    Given autonomy, budget, DoR, DoD and provider health
    When I inspect their labels and help text
    Then each term is explained in place with a short sentence
    And the explanation links to the deeper document where useful
    And no undefined acronym is left unexplained

  Scenario: [ ] E2E-466 The management experience feels calm under failure
    Given a server error, a validation failure and a timeout
    When each occurs
    Then the UI stays composed, explains the state and offers a next step
    And no raw stack, JSON blob or internal code is shown to the operator
    And the retry path is always available
```

### 23.11 Management accessibility and keyboard

```gherkin
@inventory @lab @a11y @p0 @fe-010
Feature: Management accessibility and keyboard

  Scenario: [ ] E2E-467 Every management route has landmarks and a heading order
    Given the agents, projects, providers and settings routes
    When the accessibility tree is inspected
    Then each route has a main landmark and a logical heading order
    And the skip link is the first focusable element
    And there is exactly one h1 per route

  Scenario: [ ] E2E-468 Focus order follows the visual order
    Given a management screen with a sidebar, header, form and actions
    When I tab through it
    Then focus follows the reading order
    And it never jumps into an off-screen or hidden region
    And the focused element is scrolled into view

  Scenario: [ ] E2E-469 Form errors are programmatically linked
    Given an invalid management form
    When the error appears
    Then the input has aria-invalid and aria-describedby
    And the error is announced
    And the error summary is focusable

  Scenario: [ ] E2E-470 Tabs, menus and lists use roving tabindex
    Given tabs, menus and row action menus
    When I use Arrow, Home, End, Enter and Escape
    Then the documented keyboard model works
    And focus stays within the component
    And the selection is announced

  Scenario: [ ] E2E-471 Drawers and dialogs are accessible modals
    Given an open drawer or dialog
    When it renders
    Then it has role dialog, aria-modal and an accessible name
    And focus is trapped and restored
    And the background is inert to assistive technology

  Scenario: [ ] E2E-472 Status and validation are never colour-only
    Given statuses and validation errors
    When they render
    Then text and icons accompany the colour
    And a colourblind simulation still communicates the meaning
    And contrast remains sufficient

  Scenario: [ ] E2E-473 Axe finds no critical management issues
    Given the agents, projects, providers and settings routes
    When axe runs with WCAG 2.2 AA rules
    Then there are zero critical or serious violations
    And the result is attached to the certification evidence
    And keyboard and screen-reader checks pass

  Scenario: [ ] E2E-474 Table, filter and pagination changes are announced
    Given a screen reader on a management list
    When I sort, filter or change page
    Then a live region announces the new result count and state
    And focus remains on the triggering control
    And no announcement is duplicated
```

### 23.12 Management performance and Jobs-quality certification

```gherkin
@inventory @lab @performance @certification @p0 @ux-040
Feature: Management performance and Jobs-quality certification

  Scenario: [ ] E2E-475 Management interactions feel immediate
    Given the agents, projects, providers and settings routes
    When I search, filter, open a drawer or submit a form
    Then visual feedback starts within 100 ms
    And the list load p95 is within the route budget
    And no interaction blocks the main thread with a long task

  Scenario: [ ] E2E-476 Management routes meet the performance budget
    Given the management routes in a production build
    When Lighthouse and the bundle budgets run
    Then performance is at least 90 on desktop
    And the initial JS and CSS budgets hold
    And the route lazy-loads without a blank frame

  Scenario: [ ] E2E-477 Management visual baselines cover the responsive matrix
    Given the deterministic seed
    When I capture the agents, projects, providers and settings routes at desktop, tablet and phone widths
    Then every baseline exists
    And the pixel diff is at most 0.1 percent with documented masks
    And the baseline is generated from the seed, not hand-edited

  Scenario: [ ] E2E-478 Management pages do not shift or jank
    Given a slow network and a large list
    When the page loads and a drawer opens
    Then cumulative layout shift is at most 0.1
    And no frame exceeds the long-task budget during interaction
    And the content remains stable under the pointer

  Scenario: [ ] E2E-479 Every management scenario attaches evidence
    Given an executed management scenario
    When it completes
    Then it attaches a screenshot, trace, axe result, performance sample and the relevant logs
    And the artifact paths are referenced by the E2E id
    And secrets and personal data are redacted

  Scenario: [ ] E2E-480 The Jobs-quality management certification is release-blocking
    Given the complete management certification suite
    When the release candidate is evaluated
    Then clarity, hierarchy, consistency, responsiveness, usability, accessibility and performance all pass
    And the visual baselines and evidence are current
    And a single `@p0` failure blocks the release with the failing E2E ids
    And the certified result is recorded as a reproducible artifact
```

---

## Implementation Notes

- **Suggested Playwright layout.** One spec file per area (`health.spec.ts`, `providers.spec.ts`, `kanban.spec.ts`, `sow.spec.ts`, `runs.spec.ts`, `memory.spec.ts`, `office.spec.ts`, `office-certification.spec.ts`, `management-ui.spec.ts`, `security.spec.ts`, `journeys.spec.ts`). Give each test the stable id from its scenario title (for example `test('E2E-057 ...')`) so reporting maps directly to this checklist.
- **Amiga-style certification.** Section 22 is implemented as a visual certification spec with a fixed seed, canvas pixel sampling, frame hashes, FPS and draw-call probes, axe, keyboard, screen-reader and reduced-motion emulation, and golden screenshots per viewport, DPR, theme and motion setting. Baselines live in `web/e2e-bdd/visual-baselines/` and are updated only by an intentional reviewed change. A single `@p0` failure blocks release.
- **Jobs-quality management certification.** Section 23 is implemented as a management-UI certification spec covering the inventory and lab settings: tokens and hierarchy, wizard usability, validation, tables and states, responsive breakpoints, touch and keyboard, onboarding, microcopy, a11y and performance. Golden screenshots cover desktop, tablet and phone widths, and a single `@p0` failure blocks release.
- **Fixtures.** Build API fixtures in `web/e2e/fixtures/` and reuse `scripts/seed-demo.sh` for journey and visual tests. Never share mutable state between scenarios; reset via the seed or a per-test transaction.
- **Priority.** Run `@p0` on every pull request, `@p1` nightly, and `@p2` weekly or on release candidates.
- **Evidence.** Each completed scenario links to its CI run, its artifact (trace, screenshot or report) and the owning work-package row. A scenario without evidence is not "covered".

## References

- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §3, §6–§19, §23
- `.docs/technical-specification/APPENDIX-B-CONTRACTS.md`
- `.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md`
- `.docs/product/acceptance-report.md`, `.docs/product/testing.md`, `.docs/product/demo-script.md`, `.docs/product/ui.md`
