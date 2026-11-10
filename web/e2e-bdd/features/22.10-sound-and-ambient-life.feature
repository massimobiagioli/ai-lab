@office @audio @p1 @ux-027
Feature: Sound and ambient life

  @E2E-335 @suite-office
  Scenario: E2E-335 The merge chime plays once and respects mute
    Given sound is enabled
    When a merge completes
    Then the success chime plays once
    Then the volume follows the configured level
    Then mute prevents the sound without blocking the animation

  @E2E-336 @suite-office
  Scenario: E2E-336 The blocked warning respects autoplay rules
    Given a blocked agent and a browser autoplay policy
    When the warning chime would play
    Then the chime plays only after a user gesture when required
    Then reduced-motion or mute settings are respected
    Then no sound plays for a historical event on page load

  @E2E-337 @suite-office
  Scenario: E2E-337 Ambient actions happen only while idle
    Given an idle agent near the coffee machine
    When ambient life runs
    Then coffee trips, plant sway and pet or fish motion occur only for idle agents
    Then a work event yields the ambient action within one tick
    Then the ambient action never blocks a desk

  @E2E-338 @suite-office
  Scenario: E2E-338 Ambient actions never change domain state
    Given an ambient coffee trip
    When the action completes
    Then no card, run, agent status or audit row changes
    Then the office returns the agent to its previous idle state
    Then the frame hash returns to the expected idle sequence

  @E2E-339 @suite-office
  Scenario: E2E-339 The sound preference persists safely
    Given the operator changes volume or mute in settings
    When the lab reloads
    Then the preference persists for the lab
    Then it is stored without a secret
    Then the default respects the platform autoplay policy

  @E2E-340 @suite-office
  Scenario: E2E-340 Office audio and visuals contain no unlicensed asset
    Given the office sound and sprite manifests
    When the asset linter runs
    Then every sound and sprite has license metadata
    Then no emoji is used in the office
    Then a missing or non-redistributable asset fails the build
