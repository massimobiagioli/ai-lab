@inventory @team @p1 @fr-inv-001
Feature: Inventory and team gap closure

  @E2E-201 @suite-core
  Scenario: E2E-201 Provider status lifecycle is controllable and audited
    Given an active provider with a valid key
    When I disable it, request a completion and then enable it again
    Then the disabled provider is not selected for new runs
    Then a failed health probe marks it degraded with a visible status
    Then enabling it restores active status
    Then every status change is written to the audit log with actor and reason

  @E2E-202 @suite-core
  Scenario: E2E-202 Model fields are validated and the catalog stays consistent
    Given the model create or edit form
    When I submit a non-positive context window or a negative input or output cost
    Then the response status is 422
    Then a valid model with positive windows and non-negative costs is accepted
    Then the catalog never contains a third v1 model by accident

  @E2E-203 @suite-core
  Scenario: E2E-203 Provider and lab token buckets queue work on exhaustion
    Given a provider rate limit and a lab rate limit are configured
    When both buckets are exhausted for the current window
    Then new completion requests are queued rather than failed
    Then the queue drains when the window resets
    Then the per-provider and per-lab counters are independent

  @E2E-204 @suite-core
  Scenario: E2E-204 SCM credential verification and PAT class are explicit
    Given a fine-grained GitHub PAT
    When I create and verify the SCM credential
    Then the verification state is recorded without exposing the token
    Then a classic PAT is accepted with a visible warning
    Then an invalid or expired token sets the credential to unverified with a reason

  @E2E-205 @suite-core
  Scenario: E2E-205 Unknown agent roles are rejected
    Given the agent create form
    When I submit role "manager" or any value outside tech_lead, analyst, dev and qa
    Then the response status is 422
    Then the role violation is reported

  @E2E-206 @suite-core
  Scenario: E2E-206 The agent state machine is explicit and observable
    Given an offline agent
    When it starts, receives a lease, becomes blocked, resolves the block, pauses and resumes
    Then the status sequence follows offline, idle, working, blocked, working, paused, idle
    Then each change emits agent.status_changed
    Then the office reflects the new state within 250 ms p95

  @E2E-207 @suite-core
  Scenario: E2E-207 Effective autonomy and self-modification are enforced
    Given a lab at A2, a card at A1 and an agent at A3
    When a run computes its effective autonomy
    Then the effective level is the minimum of the three, A1
    Then a lab cannot enable A3 without an explicit opt-in and a hard monthly budget
    Then an agent attempt to change its own charter, CV, autonomy, budget or scheduler settings is rejected
    Then the rejection is audited
