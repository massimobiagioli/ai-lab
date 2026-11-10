@office @a11y @p0 @ux-034
Feature: Accessibility and reduced motion

  @E2E-325 @suite-office
  Scenario: E2E-325 Reduced motion disables walking and ambient motion
    Given prefers-reduced-motion is active
    When the office renders
    Then walking becomes instant reposition with a fade
    Then ambient and parallax motion are disabled
    Then state changes and speech text remain available

  @E2E-326 @suite-office
  Scenario: E2E-326 Reduced motion replaces typing with a static pose
    Given reduced motion and a working agent
    When the office renders
    Then typing is replaced by a static working pose
    Then the status label still says working
    Then the desk LED is still truthful

  @E2E-327 @suite-office
  Scenario: E2E-327 The screen-reader office list is ordered and complete
    Given a screen reader is active
    When I open the office
    Then a hidden but focusable office list exposes agents in a stable order
    Then each entry has status, current card and available actions
    Then the list matches the visual order

  @E2E-328 @suite-office
  Scenario: E2E-328 A live region announces state transitions
    Given the office is open
    When an agent changes state
    Then the live region announces the agent, the old state and the new state
    Then the announcement is not duplicated
    Then it does not steal focus

  @E2E-329 @suite-office
  Scenario: E2E-329 The canvas has an accessible name and fallback
    Given a screen reader or a browser without canvas support
    When the office loads
    Then the canvas has a descriptive accessible name and role
    Then the fallback list provides the same information
    Then no critical action is canvas-only

  @E2E-330 @suite-office
  Scenario: E2E-330 Focus is always visible and trapped correctly
    Given the inspector or a drawer is open
    When I tab through the controls
    Then focus is visible with at least 3 to 1 contrast
    Then focus is trapped inside the modal surface
    Then closing restores focus to the trigger

  @E2E-331 @suite-office
  Scenario: E2E-331 Status is never conveyed by colour alone
    Given every agent state
    When the desk and inspector are inspected
    Then each state has a text label and an icon
    Then the LED colour is supplementary
    Then a colourblind simulation still distinguishes the states

  @E2E-332 @suite-office
  Scenario: E2E-332 Text contrast meets WCAG 2.2 AA
    Given the office DOM overlays and the in-canvas labels
    When contrast is measured
    Then normal text is at least 4.5 to 1
    Then large text is at least 3 to 1
    Then the high-contrast theme improves rather than weakens contrast

  @E2E-333 @suite-office
  Scenario: E2E-333 Automated axe checks find no critical office issues
    Given the office route
    When axe runs with WCAG 2.2 AA rules
    Then there are zero critical or serious violations
    Then keyboard and screen-reader checks pass
    Then the result is attached to the certification report

  @E2E-334 @suite-office
  Scenario: E2E-334 Color-scheme and contrast preferences are respected
    Given prefers-color-scheme and prefers-contrast variations
    When the office renders
    Then the chosen theme remains readable
    Then the palette and pixel edges are preserved
    Then no preference forces an unsupported colour outside the ramp
