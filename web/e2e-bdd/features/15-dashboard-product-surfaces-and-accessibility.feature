@ui @ux-040 @p0
Feature: Dashboard, product surfaces and accessibility

  @E2E-157 @suite-management
  Scenario: E2E-157 The dashboard shows real KPIs and never fake zeros
    Given the demo seed is loaded
    When I open /
    Then the KPI strip shows active agents, cards by column, spend today and success rate
    Then while loading it shows a loading state, and on error an error card with retry
    Then it never reports zeros as if they were real data

  @E2E-158 @suite-management
  Scenario: E2E-158 The activity feed and health pill reflect live state
    Given recent domain events
    When the dashboard loads
    Then the activity feed shows the last 50 events
    Then the system health pill reflects the current health status

  @E2E-159 @suite-management
  Scenario: E2E-159 The onboarding checklist guides first-run setup
    Given a fresh installation with no provider
    When I open the dashboard
    Then the checklist lists provider, key, lab, Tech Lead, GitHub, project and idea
    Then each completed step is marked done
    Then the next incomplete step is actionable

  @E2E-160 @suite-management
  Scenario: E2E-160 Empty states are illustrated and actionable
    Given a lab with no projects
    When I open the projects inventory
    Then an illustrated empty state with a single clear CTA is shown
    Then there is no dead end

  @E2E-161 @suite-management
  Scenario: E2E-161 Board manipulation works from the keyboard
    Given a focused TODO card
    When I press M and the arrow keys and Enter
    Then the card moves according to the transition guards
    Then Escape cancels the move
    Then focus remains visible throughout

  @E2E-162 @suite-management
  Scenario: E2E-162 The shell is responsive across breakpoints
    Given widths at or above 1280, between 768 and 1279, and below 768
    When I load the app at each width
    Then the sidebar is expanded, collapsed to icons and a bottom sheet respectively
    Then the Kanban becomes a single-column swipe on narrow screens
    Then orientation changes reflow without a reload

  @E2E-163 @suite-management
  Scenario: E2E-163 Automated accessibility checks find no serious issues
    Given the routes /, /settings, /memory and /design
    When axe runs with WCAG 2.2 AA rules
    Then there are no critical or serious violations
    Then focus is visible and meaning is never conveyed by color alone

  @E2E-164 @suite-management
  Scenario: E2E-164 The global error boundary degrades gracefully
    Given the API returns an unexpected error
    When I trigger a failing action
    Then a pixel-styled toast with a trace id and retry appears
    Then the screen is not blank
    Then the error is reported to /api/v1/telemetry/frontend with the build hash
