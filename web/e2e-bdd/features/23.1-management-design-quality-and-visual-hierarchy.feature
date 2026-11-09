@inventory @lab @ui @visual @p0 @ux-060
Feature: Management design quality and visual hierarchy

  @E2E-361
  Scenario: E2E-361 The management shell uses the pixel design tokens consistently
    Given I open the agents, projects, providers and settings routes
    When the computed styles are inspected
    Then colours, spacing, borders, radii, shadows and typography come from the design tokens
    Then no feature component hardcodes a colour or spacing value
    Then the same primitive looks identical on every management route

  @E2E-362
  Scenario: E2E-362 Every management screen has one clear primary action
    Given the agents, projects, providers and settings screens
    When the visual hierarchy is inspected
    Then each screen has exactly one primary action
    Then secondary actions are visually subordinate
    Then the page title, subtitle and primary action form a clear reading order

  @E2E-363
  Scenario: E2E-363 The chrome rejects the generic SaaS look
    Given any management surface
    When borders, radii and shadows are measured
    Then radii are limited to 0, 2 or 4 pixels
    Then shadows are hard pixel offsets with no blur
    Then borders are crisp 1px or 2px lines
    Then there is no soft rounded card or floating glass effect

  @E2E-364
  Scenario: E2E-364 The spacing rhythm is consistent
    Given a management screen with forms, tables and drawers
    When the vertical and horizontal gaps are measured
    Then every spacing value is a multiple of the 4px scale
    Then related elements are closer than unrelated groups
    Then no gap looks accidental or cramped

  @E2E-365
  Scenario: E2E-365 Icons and labels are consistent and crisp
    Given buttons, tables, menus and tabs
    When their icons are inspected
    Then every icon is from the 16x16 pixel set
    Then icons use crisp edges without anti-aliasing
    Then every icon has a label or an accessible name that matches its meaning

  @E2E-366
  Scenario: E2E-366 Status is never conveyed by colour alone
    Given agents, projects and providers in every status
    When the badges and rows are inspected
    Then each status has a text label, an icon and a tone
    Then a colourblind simulation still distinguishes the status
    Then the same status looks the same across every list

  @E2E-367
  Scenario: E2E-367 Focus is visible, consistent and never lost
    Given I navigate a management screen with the keyboard
    When focus moves through links, inputs, buttons, tabs and rows
    Then every focused element has a visible focus ring
    Then the ring meets contrast requirements
    Then focus is never hidden behind a sticky element

  @E2E-368
  Scenario: E2E-368 Text contrast meets WCAG 2.2 AA
    Given the management routes with normal and large text
    When contrast is measured
    Then normal text is at least 4.5 to 1
    Then large text is at least 3 to 1
    Then disabled text remains readable and is not mistaken for enabled text

  @E2E-369
  Scenario: E2E-369 Loading does not cause layout shift
    Given a slow management list load
    When data is replaced by skeletons and then by rows
    Then the skeleton reserves the final height
    Then the cumulative layout shift is at most 0.1
    Then the page does not jump under the pointer

  @E2E-370
  Scenario: E2E-370 Motion is restrained and purposeful
    Given drawers, toasts, tabs and menus
    When they open and close
    Then durations use the 80, 160 or 240 ms tokens
    Then pixel effects use steps instead of smooth easing
    Then reduced motion removes non-essential animation
    Then no animation delays an action by more than 240 ms

  @E2E-371
  Scenario: E2E-371 Every action exposes hover, press, disabled and loading states
    Given every interactive control on the management routes
    When it is hovered, pressed, disabled or loading
    Then the state is visually distinct
    Then the control remains readable
    Then a loading control prevents a second submit

  @E2E-372
  Scenario: E2E-372 Destructive actions look and behave as destructive
    Given archive, delete, retire and reset controls
    When they are rendered and triggered
    Then they use the danger tone and a consequence-focused explanation
    Then a typed confirmation is required where the specification demands it
    Then the cancel path is safe and always visible
