@delivery @orchestration @p0 @fr-orc-002
Feature: Scheduler, leases and run pipeline

  @E2E-103
  Scenario: E2E-103 Scheduler eligibility explains every rejected card
    Given READY cards that are inactive, over capacity, over budget or already leased
    When the scheduler evaluates them
    Then each ineligible card reports an explicit reason
    Then an eligible READY card with an active assignee is selected

  @E2E-104
  Scenario: E2E-104 A card has at most one lease
    Given a READY card leased by worker A
    When worker B attempts to lease the same card
    Then worker B receives Conflict or ALREADY_LEASED
    Then no second run is created

  @E2E-105
  Scenario: E2E-105 Lease TTL, heartbeat and reaping behave as specified
    Given a lease with TTL 900 seconds and a heartbeat every 15 seconds
    When the run heartbeats
    Then the lease is extended
    When the heartbeat stops and the TTL elapses
    Then reaping removes the lease
    Then the run is timed out

  @E2E-106
  Scenario: E2E-106 The run state machine rejects invalid transitions
    Given a run in Succeeded
    When I attempt to move it to Running
    Then the transition is rejected
    Then the valid transitions Queued to Running to Succeeded, Failed or TimedOut and Cancelled are accepted

  @E2E-107
  Scenario: E2E-107 Starting a run on an eligible card moves it to DOING
    Given an eligible READY card
    When I call POST /api/v1/cards/{card_id}/runs
    Then the response status is 202
    Then a queued run is created
    Then the card becomes DOING when the run starts

  @E2E-108
  Scenario: E2E-108 Starting a run on an ineligible card is rejected
    Given a card that is not READY or an agent that is paused
    When I call POST /api/v1/cards/{card_id}/runs
    Then the response status is 422
    Then the reason is reported

  @E2E-109
  Scenario: E2E-109 Run steps are typed, ordered and streamed
    Given a running card
    When the run emits plan, tool_call, command, test, commit, pr and review steps
    Then the steps are persisted in order with arguments hash, duration and capped output
    Then the UI receives them through the realtime stream
    Then no step output exceeds 20 KB

  @E2E-110
  Scenario: E2E-110 Transient failures retry with bounded backoff
    Given a run that fails with a transient network error
    When the scheduler retries
    Then the attempts back off at 30 seconds, 2 minutes and 10 minutes with jitter
    Then after the third retry the card is dead-lettered to ICEBOX with category risk

  @E2E-111
  Scenario: E2E-111 Deterministic failures return to DOING with feedback
    Given a run that fails because tests fail deterministically
    When the failure is classified
    Then the card returns to DOING with the failing output as feedback
    Then the attempt counter increments

  @E2E-112
  Scenario: E2E-112 Policy failures dead-letter the card and escalate
    Given a run that violates a hard stop
    When the failure is classified as policy
    Then the card moves to ICEBOX
    Then an escalation reaches the Tech Lead and optionally the operator
    Then system.escalation is emitted

  @E2E-113
  Scenario: E2E-113 Budget exhaustion stops the run safely
    Given a run whose spend would exceed the card or daily budget
    When the next charge is evaluated
    Then the run stops before the charge is applied
    Then BUDGET_EXCEEDED is persisted
    Then the lab is paused and system.budget.exhausted is emitted

  @E2E-114
  Scenario: E2E-114 Missed heartbeats cancel and requeue the run
    Given a running card whose agent misses three heartbeats
    When the scheduler reconciles
    Then the run is cancelled or timed out
    Then the card is requeued with backoff

  @E2E-115
  Scenario: E2E-115 A worker crash releases the lease after the TTL
    Given a worker holding a lease crashes
    When the lease TTL elapses
    Then the lease is released and the run is marked timed_out
    Then the card becomes eligible for a new attempt

  @E2E-116
  Scenario: E2E-116 Restart reconciliation handles merged, open and branch-only states
    Given in-flight runs when the worker restarts
    When reconciliation runs
    Then a merged PR moves the card to DONE
    Then an open PR resumes review
    Then an existing branch resumes the push
    Then a run with no evidence is retried

  @E2E-117
  Scenario: E2E-117 External side effects are idempotent
    Given a run that opens a PR, comments and merges
    When the operation is replayed with the same run_id, operation and target
    Then the external side effect is not duplicated
    Then the replayed response matches the original

  @E2E-118
  Scenario: E2E-118 Failed runs are visible and replayable
    Given a dead-lettered run
    When I call GET /api/v1/labs/{lab_id}/runs?status=failed
    Then the failed run is returned with its error and cost
    Then the office filing cabinet opens the same list
    Then a replay action creates a new attempt without mutating the old run
