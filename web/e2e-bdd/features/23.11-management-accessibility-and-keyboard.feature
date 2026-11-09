@inventory @lab @a11y @p0 @fe-010
Feature: Management accessibility and keyboard

  @E2E-467
  Scenario: E2E-467 Every management route has landmarks and a heading order
    Given the agents, projects, providers and settings routes
    When the accessibility tree is inspected
    Then each route has a main landmark and a logical heading order
    Then the skip link is the first focusable element
    Then there is exactly one h1 per route

  @E2E-468
  Scenario: E2E-468 Focus order follows the visual order
    Given a management screen with a sidebar, header, form and actions
    When I tab through it
    Then focus follows the reading order
    Then it never jumps into an off-screen or hidden region
    Then the focused element is scrolled into view

  @E2E-469
  Scenario: E2E-469 Form errors are programmatically linked
    Given an invalid management form
    When the error appears
    Then the input has aria-invalid and aria-describedby
    Then the error is announced
    Then the error summary is focusable

  @E2E-470
  Scenario: E2E-470 Tabs, menus and lists use roving tabindex
    Given tabs, menus and row action menus
    When I use Arrow, Home, End, Enter and Escape
    Then the documented keyboard model works
    Then focus stays within the component
    Then the selection is announced

  @E2E-471
  Scenario: E2E-471 Drawers and dialogs are accessible modals
    Given an open drawer or dialog
    When it renders
    Then it has role dialog, aria-modal and an accessible name
    Then focus is trapped and restored
    Then the background is inert to assistive technology

  @E2E-472
  Scenario: E2E-472 Status and validation are never colour-only
    Given statuses and validation errors
    When they render
    Then text and icons accompany the colour
    Then a colourblind simulation still communicates the meaning
    Then contrast remains sufficient

  @E2E-473
  Scenario: E2E-473 Axe finds no critical management issues
    Given the agents, projects, providers and settings routes
    When axe runs with WCAG 2.2 AA rules
    Then there are zero critical or serious violations
    Then the result is attached to the certification evidence
    Then keyboard and screen-reader checks pass

  @E2E-474
  Scenario: E2E-474 Table, filter and pagination changes are announced
    Given a screen reader on a management list
    When I sort, filter or change page
    Then a live region announces the new result count and state
    Then focus remains on the triggering control
    Then no announcement is duplicated
