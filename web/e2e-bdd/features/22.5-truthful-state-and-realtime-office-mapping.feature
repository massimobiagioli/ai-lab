@office @realtime @p0 @ux-005
Feature: Truthful state and realtime office mapping

  @E2E-287
  Scenario: E2E-287 Domain states map to the truthful animations
    Given agents in idle, working, blocked, paused and offline states
    When the office renders
    Then idle maps to idle, working to type, blocked to blocked and paused or offline to sleep or empty
    Then no state shows an animation that is not in the domain

  @E2E-288
  Scenario: E2E-288 Ambient life yields to work within one tick
    Given an idle agent walking to the coffee machine
    When a working event arrives
    Then the ambient action yields within one tick
    Then the agent returns to the desk and types
    Then no work event is visually ignored

  @E2E-289
  Scenario: E2E-289 Agent status changes reach the office within the latency budget
    Given a connected office
    When agent.status_changed is emitted
    Then the office reflects the new state within 250 ms p95
    Then the desk LED, label and animation agree
    Then the event is not invented by the client

  @E2E-290
  Scenario: E2E-290 Run steps select the correct working sub-animation
    Given a working agent
    When a plan or llm_request step arrives
    Then the agent shows think
    When a read or search tool_call arrives
    Then the agent shows read
    When a write tool_call arrives
    Then the agent shows type

  @E2E-291
  Scenario: E2E-291 Merge completion celebrates the correct office entity
    Given two agents working and one merge event
    When the event is applied
    Then only the merging agent celebrates
    Then the card owner is not confused with the merger
    Then the event id is deduplicated

  @E2E-292
  Scenario: E2E-292 Budget exhaustion shows the lab-wide blocked banner
    Given an active office
    Then system.budget.exhausted is emitted
    Then a red lab-wide banner appears
    Then active agents show the blocked state
    Then the banner clears only when the lab resumes

  @E2E-293
  Scenario: E2E-293 Reconnect backfill converges to the same frame hash
    Given a client that disconnects for several events
    When it reconnects and backfills
    Then the office reaches the same state and frame hash as a continuously connected client
    Then no duplicate event is applied twice
    Then the intermediate states do not flash permanently

  @E2E-294
  Scenario: E2E-294 Out-of-order ephemeral events cannot override a newer state
    Given a newer agent state has been applied
    When an older ephemeral typing event arrives
    Then the newer state wins
    Then the office does not regress to the older animation
    Then the ephemeral event is coalesced

  @E2E-295
  Scenario: E2E-295 Duplicate events are deduplicated by id and ordered by seq
    Given a replayed event with an already seen event_id
    When the office applies the stream
    Then the duplicate is ignored
    Then the monotonic seq order is preserved
    Then no animation is triggered twice

  @E2E-296
  Scenario: E2E-296 Offline mode never invents activity
    Given the API is unreachable
    When the office loads the last snapshot
    Then it shows the last truthful state in read-only mode
    Then no idle animation invents work
    Then a reconnect reconciles before resuming animation
