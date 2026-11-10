@office @frontend @p1 @ux-016
Feature: Office and frontend gap closure

  @E2E-226 @suite-office
  Scenario: E2E-226 Office camera and object navigation are complete
    Given the office is open
    When I hover an object
    Then a pixel tooltip and outline appear
    When I press Tab, Enter and Escape
    Then focus cycles through interactive objects, Enter opens and Escape closes
    Then wheel and drag pan and zoom snap to integer factors 1x to 4x
    Then the fit button resets the camera

  @E2E-227 @suite-office
  Scenario: E2E-227 The office scales to sixteen agents without overlap
    Given a lab with sixteen agents
    When the office renders
    Then every agent is visible at a desk, hot desk or standing position
    Then no two agents or desks overlap
    Then the layout remains navigable

  @E2E-228 @suite-office
  Scenario: E2E-228 Office assets are licensed and original
    Given the sprite, font and sound manifest
    When the asset linter runs
    Then every asset has author, source and redistributable license metadata
    Then a missing license fails the build
    Then no asset is copied from the reference product and no emoji is used in the office

  @E2E-229 @suite-office
  Scenario: E2E-229 Sound and background-tab behaviour respect the operator
    Given the office is running
    When a card completes or is blocked
    Then the matching chime plays at the configured volume
    Then mute disables it and prefers-reduced-motion suppresses non-essential motion
    When the tab is backgrounded
    Then simulation pauses but WebSocket state continues to synchronize

  @E2E-230 @suite-office
  Scenario: E2E-230 Frontend budgets, offline mode and event dedupe hold
    Given a production build
    When the bundle and Lighthouse budgets are measured
    Then initial JS is at most 350 KB gzip, office at most 180 KB, CSS at most 60 KB and performance is at least 90
    Then a lost connection shows the last snapshot in read-only mode and queues no mutation
    Then replayed state events are deduplicated by event_id and applied by monotonic seq
