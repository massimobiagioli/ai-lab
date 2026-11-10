@office @performance @p0 @ux-025
Feature: Office performance, memory and determinism

  @E2E-341 @suite-office
  Scenario: E2E-341 Desktop frame rate meets the target
    Given the office at 1440p with sixteen agents and 200 animated props
    When it runs for five minutes
    Then the FPS is at least 58
    Then no frame exceeds the long-task budget repeatedly
    Then the FPS samples are attached to the evidence

  @E2E-342 @suite-office
  Scenario: E2E-342 The low-end profile meets the minimum frame rate
    Given the low-end profile with an integrated GPU and reduced effects
    When the office runs
    Then the FPS is at least 30
    Then the visual state remains truthful
    Then the quality auto-tune does not change the logical frame hash

  @E2E-343 @suite-office
  Scenario: E2E-343 The frame budget is respected
    Given a steady office
    When frame durations are measured
    Then the p95 frame time is at most 16.6 ms
    Then no recurring long task exceeds 50 ms during idle
    Then the frame budget is reported per layer

  @E2E-344 @suite-office
  Scenario: E2E-344 Peak heap and long-run memory stay bounded
    Given a ten-minute office soak
    When memory is sampled
    Then peak JS heap is at most 150 MB
    Then memory growth is under 10 percent
    Then no unbounded particle or bubble allocation is observed

  @E2E-345 @suite-office
  Scenario: E2E-345 Draw calls and atlas batching stay stable
    Given the full office scene
    When ten consecutive frames are profiled
    Then draw calls stay at or below 150
    Then the atlas batch count is stable
    Then no per-frame atlas rebuild occurs

  @E2E-346 @suite-office
  Scenario: E2E-346 A background tab pauses simulation but keeps state in sync
    Given the office is visible
    When the tab is backgrounded
    Then the simulation pauses
    Then the WebSocket state keeps synchronizing
    Then returning to the tab does not replay thousands of frames

  @E2E-347 @suite-office
  Scenario: E2E-347 Resuming a background tab catches up without teleporting
    Given a backgrounded tab and new events
    When I return to the tab
    Then the office catches up to the latest state
    Then agents do not teleport except through valid movement
    Then the frame hash matches a continuously visible client

  @E2E-348 @suite-office
  Scenario: E2E-348 Determinism survives repeated runs
    Given the same seed and event sequence
    When the office runs three times
    Then the final positions, animation frames and z-order are identical
    Then the frame hash is identical
    Then a single event difference changes the hash

  @E2E-349 @suite-office
  Scenario: E2E-349 The optional WebGL2 backend is pixel-identical
    Given the WebGL2 backend is enabled
    When the same frame renders in Canvas 2D and WebGL2
    Then the pixel diff is zero or within the declared tolerance
    Then the palette, edges and integer scaling match
    Then disabling WebGL2 restores the Canvas 2D reference

  @E2E-350 @suite-office
  Scenario: E2E-350 The office stress test holds for one hour
    Given sixteen agents and 200 animated props
    When the office runs for 60 minutes
    Then the FPS target is sustained
    Then memory growth is under 10 percent
    Then no visual state diverges
    Then the evidence is attached to the certification report
