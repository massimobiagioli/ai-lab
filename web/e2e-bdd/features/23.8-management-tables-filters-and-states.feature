@inventory @lab @ui @tables @p1 @fe-008
Feature: Management tables, filters and states

  @E2E-441
  Scenario: E2E-441 Tables are semantic and accessible
    Given the agents, projects and providers tables
    When they render
    Then each table has a caption and column headers
    Then sortable headers expose aria-sort
    Then row actions have accessible names

  @E2E-442
  Scenario: E2E-442 Columns adapt without losing meaning
    Given a management table at desktop, tablet and phone widths
    When the viewport changes
    Then non-critical columns collapse or move into a row detail
    Then the primary identity and action remain visible
    Then no critical value is hidden without access

  @E2E-443
  Scenario: E2E-443 Filters are visible, resettable and persistent
    Given a list with search and filters
    When I apply filters and reload
    Then the filters are visible as chips or controls
    Then a reset action clears them
    Then the URL preserves the filter state

  @E2E-444
  Scenario: E2E-444 Empty filter results are recoverable
    Given a filter that matches nothing
    When the result renders
    Then an empty result state explains the filter
    Then a clear-filters action is present
    Then the user can recover without retyping

  @E2E-445
  Scenario: E2E-445 Loading skeletons preserve the final layout
    Given a slow list request
    When the skeleton renders
    Then rows and columns reserve the final space
    Then the layout shift is minimal
    Then the ready state does not jump

  @E2E-446
  Scenario: E2E-446 List errors are actionable
    Given a failing list request
    When the error state renders
    Then it shows a plain-language message and a retry
    Then the trace id is available
    Then the rest of the shell remains usable

  @E2E-447
  Scenario: E2E-447 Pagination boundaries are correct
    Given the first, middle and last page
    When I navigate
    Then previous and next are disabled at the correct boundaries
    Then the page size is respected
    Then the cursor is stable across reloads

  @E2E-448
  Scenario: E2E-448 Row actions are keyboard and pointer equivalent
    Given a table row
    When I open its actions with a mouse, keyboard or touch
    Then the same menu opens
    Then arrow, Home, End and Escape work
    Then the action result is identical

  @E2E-449
  Scenario: E2E-449 Narrow screens have an explicit table strategy
    Given a phone width
    When a management table renders
    Then it becomes cards or an intentional scroll region
    Then the affordance is visible
    Then no action is unreachable

  @E2E-450
  Scenario: E2E-450 Filters and sorting never lie
    Given a filtered and sorted list
    When I inspect the rows
    Then every row satisfies the active filter
    Then the order matches the active sort
    Then the result count matches the rendered rows
