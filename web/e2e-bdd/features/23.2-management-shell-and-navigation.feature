@inventory @lab @navigation @p1 @ux-060
Feature: Management shell and navigation

  @E2E-373
  Scenario: E2E-373 Every management route is reachable from the shell
    Given the application shell
    When I use the sidebar and the in-page navigation
    Then agents, projects, providers and settings are reachable
    Then the active route is highlighted
    Then no route ends in a dead end

  @E2E-374
  Scenario: E2E-374 Search finds the entity by its visible fields
    Given agents, projects and providers with known names
    When I type a name, slug, role or repo into the list search
    Then the matching rows remain
    Then the result count is visible
    Then clearing the search restores the full list

  @E2E-375
  Scenario: E2E-375 Breadcrumbs reflect the hierarchy
    Given an entity detail open from a list
    When the breadcrumbs render
    Then they show the route from the management area to the entity
    Then the current item is marked with aria-current
    Then each parent crumb is keyboard reachable

  @E2E-376
  Scenario: E2E-376 Back and forward restore the list state
    Given a list with a search, a filter and a page cursor
    When I open a detail and then use browser back and forward
    Then the search, filter, cursor and drawer state are restored
    Then no full page reload is required

  @E2E-377
  Scenario: E2E-377 Deep-linkable UI state is stable
    Given a management URL with a selected agent, panel or card query state
    When I open it in a new session
    Then the same entity and drawer are restored
    Then the URL contains no secret or personal data
    Then an invalid state falls back safely

  @E2E-378
  Scenario: E2E-378 Empty management states always offer a next step
    Given an empty agents, projects or providers inventory
    When the screen renders
    Then an illustrated empty state explains what is missing
    Then exactly one clear CTA creates the first entity
    Then there is no dead end or disabled primary action

  @E2E-379
  Scenario: E2E-379 Loading and error are never fake data
    Given a slow or failing management API
    When the screen renders
    Then it shows a loading skeleton or an error card with retry and trace id
    Then it never shows zeros or stale rows as if they were current
    Then retry restores the ready state

  @E2E-380
  Scenario: E2E-380 Invalid or missing entities fail safely
    Given an invalid or deleted agent, project, provider or lab id
    When I open the route or drawer
    Then a safe not-found state appears
    Then the user can return to the list
    Then the screen never goes blank

  @E2E-381
  Scenario: E2E-381 View in office works from the inventory
    Given an agent, project or lab row
    When I trigger view in office
    Then the office opens focused on the entity
    Then the selection is shareable
    Then returning to inventory restores the list state

  @E2E-382
  Scenario: E2E-382 Navigation is consistent across pointer and keyboard
    Given the same management journey with a mouse and with the keyboard
    When I navigate, search, filter, open a drawer and submit a form
    Then the same result occurs
    Then no action is pointer-only
    Then focus is restored to the trigger after closing a drawer
