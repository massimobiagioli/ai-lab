@office @visual @certification @p0 @ux-027
Feature: Visual regression and certification evidence

  @E2E-351
  Scenario: E2E-351 Golden screenshots cover the visual matrix
    Given the deterministic office seed
    When I capture 1x, 2x and 3x at DPR 1 and 2, with reduced motion on and off
    Then every capture exists in the golden set
    Then each baseline is generated from the seed, not edited by hand
    Then the capture metadata records viewport, DPR and theme

  @E2E-352
  Scenario: E2E-352 Pixel diffs respect the threshold and masks
    Given a current capture and its golden baseline
    When the diff is computed
    Then the changed-pixel ratio is at most 0.1 percent
    Then only documented dynamic regions are masked
    Then the diff image and the mask definition are attached

  @E2E-353
  Scenario: E2E-353 The palette audit proves the 32-colour limit
    Given a full office capture
    When every pixel is sampled
    Then every colour belongs to the fixed palette ramp
    Then the distinct-colour count is at most 32 per atlas
    Then any off-palette colour fails the certification

  @E2E-354
  Scenario: E2E-354 The anti-aliasing audit proves crisp edges
    Given sprite, icon and bevel captures
    When edge pixels are sampled
    Then there is no partial blend colour
    Then alpha is transparent or palette-exact
    Then the audit report lists the sampled edges

  @E2E-355
  Scenario: E2E-355 The scanline audit proves the opacity budget
    Given the scanline effect on and off
    When the overlay pixels are measured
    Then the effect opacity is at most 4 percent
    Then it is disabled under reduced motion
    Then the underlying palette remains unchanged

  @E2E-356
  Scenario: E2E-356 Z-order golden frames match the expected composition
    Given agents behind, in front of and beside furniture
    When the golden frame is compared
    Then the y-sorted z-order matches the expected composition
    Then no agent is drawn through a desk or wall
    Then the frame hash is recorded

  @E2E-357
  Scenario: E2E-357 The visual baseline is reproducible from the seed
    Given a clean environment and the deterministic seed
    When the baseline is regenerated
    Then it is byte-identical to the checked-in golden set
    Then no manual image edit is required
    Then a changed seed produces a different but explainable baseline

  @E2E-358
  Scenario: E2E-358 Every office scenario attaches evidence
    Given an executed office scenario
    When it completes
    Then it attaches a frame hash, a screenshot, an FPS sample and the relevant logs
    Then the artifact paths are referenced by the E2E id
    Then secrets and personal data are redacted

  @E2E-359
  Scenario: E2E-359 The certification report aggregates every dimension
    Given the office certification run
    When the report is generated
    Then it includes palette, scaling, anti-aliasing, scanline, z-order, a11y, FPS, memory, determinism and visual diff
    Then every dimension has a pass or fail with evidence
    Then a single failed dimension blocks certification

  @E2E-360
  Scenario: E2E-360 The Amiga-style certification suite is release-blocking
    Given the complete office certification suite
    When the release candidate is evaluated
    Then every `@p0` certification scenario passes
    Then visual, accessibility, performance and determinism evidence is attached
    Then any regression blocks the release with the failing E2E ids
    Then the certified result is recorded as a reproducible artifact
