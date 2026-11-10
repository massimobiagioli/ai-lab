@office @navigation @p1 @ux-017
Feature: Camera and deep links

  @E2E-309 @suite-office
  Scenario: E2E-309 The default camera is a fixed full-room view
    When the office loads
    When the camera initializes
    Then the full room is visible by default
    Then the fit button restores the default view
    Then resetting the camera does not change the frame hash

  @E2E-310 @suite-office
  Scenario: E2E-310 Pan and zoom are available only when needed
    Given the room is smaller than the viewport
    When I use the wheel or drag
    Then panning and zooming are constrained or disabled
    Then when the room exceeds the viewport they become available
    Then the camera never exposes empty space beyond the room bounds

  @E2E-311 @suite-office
  Scenario: E2E-311 Zoom snaps to integer factors
    Given the camera is zoomable
    When I zoom with the wheel or controls
    Then the zoom snaps to 1x, 2x, 3x or 4x
    Then no fractional zoom is applied
    Then the pixel grid remains crisp at every factor

  @E2E-312 @suite-office
  Scenario: E2E-312 The agent deep link restores selection and panel
    Given the URL contains the lab id, an agent id and panel=cv
    When the page loads
    Then the office focuses the agent
    Then the inspector opens on the CV tab
    Then the state matches a manual selection

  @E2E-313 @suite-office
  Scenario: E2E-313 Drawer deep links restore the correct panel
    Given the URL contains panel=health, panel=memory or panel=failed
    When the page loads
    Then the matching drawer opens with the correct data
    Then closing it updates the URL without a full reload
    Then back and forward restore the previous panel

  @E2E-314 @suite-office
  Scenario: E2E-314 The card deep link preserves split view
    Given a deep link with a card id
    When the board opens over the office
    Then the card is highlighted
    Then the office remains visible in split view
    Then an unknown card id shows a safe not-found state

  @E2E-315 @suite-office
  Scenario: E2E-315 Invalid deep-link ids fail safely
    Given a malformed or unknown lab, agent, panel or card id
    When the URL is opened
    Then the app shows a safe not-found or fallback state
    Then it never renders a blank screen
    Then the error includes a trace id without leaking internals

  @E2E-316 @suite-office
  Scenario: E2E-316 Every interactive object exposes a shareable URL
    Given an agent or an object is selected
    When I copy its shareable URL
    Then the URL restores the same selection and panel in a new session
    Then the URL contains no secret or token
    Then the URL is stable across reloads
