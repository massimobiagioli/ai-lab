@office @visual @p0 @ux-003
Feature: Characters, sprites and animation

  @E2E-275 @suite-office
  Scenario: E2E-275 The character roster is finite and validated
    Given the ten documented character ids
    When an agent is created with each id
    Then every documented id is accepted
    Then an unknown id is rejected
    Then the office renders the matching sprite set

  @E2E-276 @suite-office
  Scenario: E2E-276 Characters expose four directions with mirroring
    Given an agent walking in each direction
    When the sprite frame is selected
    Then down, up, left and right frames exist
    Then left may mirror right only when the manifest allows it
    Then the mirror flag does not change the hitbox

  @E2E-277 @suite-office
  Scenario: E2E-277 Animation frame counts and rates match the manifest
    Given an idle, walking and typing agent
    When the sprite animation advances
    Then idle uses two frames at 2 FPS
    Then walk uses four frames at 8 FPS
    Then type uses four frames at 12 FPS

  @E2E-278 @suite-office
  Scenario: E2E-278 Animation timing is tick-based with no drift
    Given the fixed 60 Hz simulation
    When the office runs for ten minutes
    Then animation timing is derived from ticks, not wall-clock time
    Then the frame index does not drift
    Then pausing and resuming does not skip a frame incorrectly

  @E2E-279 @suite-office
  Scenario: E2E-279 The walk cycle is deterministic
    Given an agent walking for a known elapsed time
    When the walk frame is computed
    Then the frame index is floor(elapsed / 200) modulo 4
    Then the same elapsed time always selects the same frame
    Then the frame changes only at the expected boundary

  @E2E-280 @suite-office
  Scenario: E2E-280 Speech bubbles wrap within the documented limits
    Given a speech message of 120 characters
    When the bubble renders
    Then text wraps at 28 characters per line
    Then a bubble has at most three lines
    Then the content never exceeds 120 characters

  @E2E-281 @suite-office
  Scenario: E2E-281 Speech bubbles auto-dismiss and open details
    Given a speech bubble appears from a real event
    When six seconds elapse
    Then the bubble auto-dismisses
    Then clicking it before dismissal opens the agent detail panel
    Then the bubble is not replayed after dismissal

  @E2E-282 @suite-office
  Scenario: E2E-282 Blocked, paused and offline states are visually distinct
    Given agents that are blocked, paused and offline
    When the office renders
    Then blocked shows the red exclamation and pacing animation
    Then paused shows the sleep animation and gray LED
    Then offline shows an empty chair with no invented activity

  @E2E-283 @suite-office
  Scenario: E2E-283 A merge triggers celebration for the right agent
    Given a delivery.merge.completed event for agent A
    When the office receives the event
    Then agent A plays the celebrate animation and confetti
    Then the success chime plays once
    Then no other agent celebrates

  @E2E-284 @suite-office
  Scenario: E2E-284 A state change stops the previous animation immediately
    Given an agent typing
    When it becomes blocked
    Then the typing animation stops at the next frame
    Then the blocked animation starts without a ghost frame
    Then the desk LED and label update with it

  @E2E-285 @suite-office
  Scenario: E2E-285 Character selection never changes permissions or behaviour
    Given two agents with the same role and different characters
    When they execute the same card
    Then their permissions and tool access are identical
    Then only the visual and speech style differ
    Then a character/role conflict logs character_override without changing policy

  @E2E-286 @suite-office
  Scenario: E2E-286 Every sprite, font and sound asset is licensed
    Given the asset manifest
    When the asset linter runs
    Then every asset has author, source and redistributable license metadata
    Then a missing license fails the build
    Then no asset is copied from the reference product
