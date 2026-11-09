@office @responsive @p1 @ux-030
Feature: Responsive layout and input modes

  @E2E-317
  Scenario: E2E-317 Desktop layout expands the sidebar
    Given a viewport at or above 1280 px
    When the office renders
    Then the sidebar is expanded to 240 px
    Then the office and the optional inspector fit without overlap
    Then the canvas keeps integer scaling

  @E2E-318
  Scenario: E2E-318 Tablet layout collapses the sidebar
    Given a viewport between 768 and 1279 px
    When the office renders
    Then the sidebar collapses to 64 px icons
    Then the inspector overlays instead of pushing the office
    Then every action remains reachable

  @E2E-319
  Scenario: E2E-319 Phone layout uses a bottom sheet and full-screen inspector
    Given a viewport below 768 px
    When the office renders
    Then the sidebar becomes a bottom sheet
    Then the office scales to fit the width
    Then the inspector becomes a full-screen sheet
    Then the Kanban becomes a single-column swipe

  @E2E-320
  Scenario: E2E-320 Orientation changes reflow without a reload
    Given a device in portrait
    When I rotate it to landscape and back
    Then the layout reflows without a page reload
    Then the office state and selection are preserved
    Then the scale remains an integer

  @E2E-321
  Scenario: E2E-321 Touch targets meet the minimum size
    Given a touch device
    When I inspect every interactive office control
    Then each target is at least 44 by 44 CSS pixels
    Then adjacent targets do not overlap
    Then the targets remain reachable at 200 percent text zoom

  @E2E-322
  Scenario: E2E-322 A keyboard-only user completes the office journey
    Given no pointer device
    When I navigate, select an agent, open the inspector and run a control
    Then every step is reachable
    Then focus never disappears
    Then the result matches the pointer journey

  @E2E-323
  Scenario: E2E-323 Pointer and touch produce the same result
    Given the same object and action
    When I trigger it with a mouse and with touch
    Then the same drawer, selection and state change occur
    Then no double event or ghost click is produced

  @E2E-324
  Scenario: E2E-324 DOM overlays do not cover the canvas at 200 percent zoom
    Given the browser text zoom is 200 percent
    When the office and overlays render
    Then overlays reflow without covering the selected agent
    Then the canvas remains visible and interactive
    Then no horizontal scroll is introduced
