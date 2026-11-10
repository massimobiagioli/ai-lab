@office @visual @p0 @ux-020
Feature: Amiga visual language and palette

  @E2E-241 @suite-office
  Scenario: E2E-241 The office uses the fixed internal resolution and tile grid
    Given the office is loaded
    When the engine reports its render model
    Then the internal resolution is exactly 384x216 logical pixels
    Then the grid is 24x13 tiles of 16 pixels with a 4-pixel top offset
    Then no layout computation uses a fractional tile

  @E2E-242 @suite-office
  Scenario: E2E-242 The integer scale factor follows the capitolato formula
    Given viewports of 384x216, 768x432, 1440x900 and 1920x1080
    When the office computes its scale
    Then the scale is floor(min(viewport_width / 384, viewport_height / 216))
    Then the scale is always an integer
    Then the canvas never uses a fractional multiplier

  @E2E-243 @suite-office
  Scenario: E2E-243 Rendering is pixelated and nearest-neighbour
    Given the office canvas is visible
    When the browser computes its rendering style
    Then image-rendering is pixelated
    Then no smoothing, bilinear or bicubic filter is applied
    Then the sprite atlas is sampled with nearest-neighbour

  @E2E-244 @suite-office
  Scenario: E2E-244 The palette is limited to the fixed 32 colours
    Given the office renders a full room with agents and furniture
    When every visible pixel is sampled
    Then every colour belongs to the fixed Amiga palette ramp
    Then no colour is introduced by an antialiasing or CSS fallback

  @E2E-245 @suite-office
  Scenario: E2E-245 Chrome is beveled with hard pixel edges
    Given panels, buttons and the window strip are visible
    When their computed styles are inspected
    Then borders are 1px or 2px solid
    Then shadows are hard pixel offsets with no blur radius
    Then radii are limited to 0, 2 or 4 pixels
    Then there is no rounded SaaS corner or soft drop shadow

  @E2E-246 @suite-office
  Scenario: E2E-246 The scanline texture stays within the opacity budget
    Given the scanline effect is enabled
    When the overlay is measured
    Then its opacity is at most 4 percent
    Then the effect is disabled when reduced motion is active
    Then disabling it leaves the office legible

  @E2E-247 @suite-office
  Scenario: E2E-247 In-canvas typography uses the bitmap pixel font
    Given the office displays a desk plaque and a speech bubble
    When the rendered text is inspected
    Then the canvas text uses the bitmap pixel font at an integer size
    Then no emoji is rendered in the office
    Then glyph edges remain crisp at every integer scale

  @E2E-248 @suite-office
  Scenario: E2E-248 Gradients are dithered from the fixed palette
    Given the office renders a gradient surface
    When the pixels are inspected
    Then the gradient is produced by palette dithering
    Then no smooth CSS linear-gradient appears inside the canvas
    Then the dither pattern is deterministic

  @E2E-249 @suite-office
  Scenario: E2E-249 Status LEDs are colourblind-safe and never colour-only
    Given agents in idle, working, blocked, paused and offline states
    When their desk LEDs are inspected
    Then each LED uses the colourblind-safe accent for its state
    Then the desk also exposes a text label and a role icon
    Then an operator can distinguish every state without colour vision

  @E2E-250 @suite-office
  Scenario: E2E-250 Sprites and chrome have no anti-aliasing
    Given a sprite edge, an icon and a beveled border
    When the rendered pixels are sampled
    Then no intermediate blend colour appears on the edges
    Then the sprite alpha is either transparent or palette-exact
    Then the browser zoom is 100 percent

  @E2E-251 @suite-office
  Scenario: E2E-251 The amiga500 theme applies consistently
    Given the lab office theme is amiga500
    When the office and its DOM overlays render
    Then the canvas and the overlays use the same token palette
    Then the window strip, clock, LEDs and sound control use the theme tokens
    Then switching the theme persists to the lab settings

  @E2E-252 @suite-office
  Scenario: E2E-252 The high-contrast theme preserves the visual language
    Given the high-contrast theme is enabled
    When the office and overlays render
    Then text contrast is at least 4.5 to 1
    Then the palette remains within the documented high-contrast ramp
    Then the pixel edges, bevels and integer scaling remain intact
