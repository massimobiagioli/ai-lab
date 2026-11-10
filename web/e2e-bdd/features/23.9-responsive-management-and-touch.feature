@inventory @lab @responsive @p1 @ux-030
Feature: Responsive management and touch

  @E2E-451 @suite-management
  Scenario: E2E-451 Desktop management layout is calm and readable
    Given a viewport at or above 1280 px
    When a management screen renders
    Then the sidebar is expanded
    Then the content has a comfortable line length and clear gutters
    Then the primary action is not lost in the chrome

  @E2E-452 @suite-management
  Scenario: E2E-452 Tablet management uses collapsible navigation
    Given a viewport between 768 and 1279 px
    When a management screen renders
    Then the sidebar collapses to icons
    Then drawers overlay the content
    Then every action remains reachable

  @E2E-453 @suite-management
  Scenario: E2E-453 Phone management becomes a focused single column
    Given a viewport below 768 px
    When a management screen renders
    Then the sidebar becomes a bottom sheet
    Then lists become single-column cards or an intentional scroll region
    Then drawers become full-screen sheets
    Then the primary action stays visible

  @E2E-454 @suite-management
  Scenario: E2E-454 Orientation changes preserve management state
    Given a completed filter and an open detail
    When I rotate the device
    Then the layout reflows without a reload
    Then the filter and detail are preserved
    Then focus is not lost

  @E2E-455 @suite-management
  Scenario: E2E-455 Touch targets are comfortable
    Given a touch device
    When I inspect management controls
    Then each target is at least 44 by 44 CSS pixels
    Then adjacent targets do not overlap
    Then no action depends on hover

  @E2E-456 @suite-management
  Scenario: E2E-456 Text zoom does not break the management UI
    Given the browser text zoom is 200 percent
    When a management screen renders
    Then text does not overlap or clip
    Then critical actions remain visible
    Then the layout remains usable without horizontal scrolling

  @E2E-457 @suite-management
  Scenario: E2E-457 Long values remain accessible on small screens
    Given long names, repository URLs and descriptions
    When the phone layout renders
    Then values wrap or truncate with an accessible full value
    Then no row action is pushed off screen
    Then the visual hierarchy remains intact

  @E2E-458 @suite-management
  Scenario: E2E-458 Responsive management has no dead ends
    Given every management route at desktop, tablet and phone widths
    When I navigate and act
    Then every screen has a reachable primary action
    Then empty, loading and error states remain actionable
    Then no breakpoint hides the only path forward
