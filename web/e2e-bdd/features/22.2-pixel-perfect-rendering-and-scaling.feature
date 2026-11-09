@office @visual @p0 @ux-020
Feature: Pixel-perfect rendering and scaling

  @E2E-253
  Scenario: E2E-253 Device pixel ratio never changes the logical render
    Given the same office state at DPR 1, 1.5, 2 and 3
    When each render is captured
    Then the canvas buffer is always 384x216 logical pixels
    Then only the CSS size changes with the device pixel ratio
    Then the logical frame hash is identical across the four captures

  @E2E-254
  Scenario: E2E-254 Integer zoom never produces fractional blur
    Given the camera at 1x, 2x, 3x and 4x
    When each zoom level renders
    Then the canvas scale is an integer
    Then no pixel is resampled with a fractional offset
    Then the edge of every 1-pixel line is a single crisp pixel

  @E2E-255
  Scenario: E2E-255 The canvas buffer matches the declared resolution
    Given the office canvas is mounted
    When its width and height attributes are read
    Then the buffer is exactly 384x216
    Then the CSS dimensions are an integer multiple of the buffer
    Then the canvas is not stretched by a non-integer transform

  @E2E-256
  Scenario: E2E-256 Static layers are cached offscreen
    Given the floor, walls and behind-furniture layers do not change
    When the engine renders consecutive frames
    Then the static layers are drawn from offscreen canvases
    Then only dynamic layers are redrawn
    Then disabling the cache restores the same final frame

  @E2E-257
  Scenario: E2E-257 The draw-call budget is respected
    Given the office with sixteen agents and full furniture
    When a frame is profiled
    Then the draw calls are at most 150
    Then draws are batched by atlas
    Then the draw-call count stays stable across ten frames

  @E2E-258
  Scenario: E2E-258 The sprite atlas stays within the memory budget
    Given the office assets are loaded
    When the atlases are decoded
    Then the compressed atlas size is at most 8 MB
    Then the decoded memory is reported
    Then all critical atlases are ready before the simulation starts

  @E2E-259
  Scenario: E2E-259 Every manifest frame resolves
    Given the sprite manifest
    When the engine loads every declared frame
    Then every frame id resolves to a rectangle
    Then a deliberately missing frame fails fast with the frame id
    Then the loading screen never starts the simulation with a missing critical atlas

  @E2E-260
  Scenario: E2E-260 The frame hash is stable for identical inputs
    Given the same event sequence and seed
    When the office reaches the same tick
    Then the frame hash is identical across two runs
    Then the hash covers positions, animation frames and z-order
    Then a one-event difference changes the hash

  @E2E-261
  Scenario: E2E-261 The office is deterministic across viewport changes
    Given the same events at 1280x720 and 1920x1080
    When both runs reach the same tick
    Then the logical positions and frame hash are identical
    Then only the integer scale differs

  @E2E-262
  Scenario: E2E-262 Visual regression stays under the pixel threshold
    Given a current office screenshot and its golden baseline
    When the visual diff is computed
    Then the changed-pixel ratio is at most 0.1 percent
    Then only documented dynamic regions are masked
    Then the diff artifact is attached to the result

  @E2E-263
  Scenario: E2E-263 Resize and orientation preserve integer scaling
    Given the office at a desktop, tablet and phone width
    When I resize the window or rotate the device
    Then the office reflows without a reload
    Then the scale remains an integer
    Then no fractional blur or overflow appears

  @E2E-264
  Scenario: E2E-264 Fullscreen and fit preserve the pixel grid
    Given the office in windowed mode
    When I enter fullscreen and press fit
    Then the scale is an integer
    Then the room is centered with letterboxing where needed
    Then no sprite is cropped or stretched
