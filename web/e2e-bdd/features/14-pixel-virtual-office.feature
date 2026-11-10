@office @ux-001 @p0
Feature: Pixel virtual office

  @E2E-145 @suite-office
  Scenario: E2E-145 Every active agent is rendered with a persistent desk
    Given a lab with four agents
    When I open /lab/{labId}
    Then each agent appears at a desk
    Then each desk plaque shows name, role icon and a status LED

  @E2E-146 @suite-office
  Scenario: E2E-146 Animation is derived truthfully from domain state
    Given agents that are idle, working, blocked and paused
    When the office renders
    Then idle shows idle, working shows type, blocked shows blocked and paused shows sleep
    Then no agent invents activity that is not in the domain

  @E2E-147 @suite-office
  Scenario: E2E-147 Pathfinding is deterministic and avoids corner cutting
    Given an agent that must walk around furniture
    When it computes a path
    Then the path is the shortest octile path
    Then diagonals do not cut blocked corners
    Then the same grid and seed produce the same positions

  @E2E-148 @suite-office
  Scenario: E2E-148 Clicking an agent opens the inspector
    Given the office is open
    When I click an agent
    Then the Agent Inspector opens with Live, CV, Memory and Controls tabs

  @E2E-149 @suite-office
  Scenario: E2E-149 Deep links restore selection and panel
    Given the URL /lab/{labId}?agent={agentId}&panel=cv
    When the page loads
    Then the inspector opens on the CV tab for that agent
    Then the object exposes a shareable URL

  @E2E-150 @suite-office
  Scenario: E2E-150 Office objects open their bound surfaces
    Given the office is open
    When I click the whiteboard, server rack, bookshelf and filing cabinet
    Then they open Kanban, System Health, Memory explorer and Failed Runs respectively

  @E2E-151 @suite-office
  Scenario: E2E-151 Speech bubbles come from real events
    Given a domain event that produces an agent message
    When the event arrives
    Then a bubble of at most 120 characters appears
    Then it auto-dismisses after 6 seconds
    Then clicking it opens the detail panel

  @E2E-152 @suite-office
  Scenario: E2E-152 Reduced motion is honored
    Given prefers-reduced-motion is set
    When the office renders
    Then walking and ambient motion are replaced with instant or static poses
    Then state changes and speech text remain visible

  @E2E-153 @suite-office
  Scenario: E2E-153 The screen-reader office list announces transitions
    Given a screen reader is active
    When agent states change
    Then the hidden office list exposes agent, status, current card and actions
    Then a live region announces the transition

  @E2E-154 @suite-office
  Scenario: E2E-154 The office meets its frame-rate budget
    Given 16 agents and 200 animated props
    When the office runs at 1440p on the reference host
    Then desktop FPS is at least 58
    Then the low-end profile sustains at least 30 FPS

  @E2E-155 @suite-office
  Scenario: E2E-155 Scaling is integer and pixelated
    Given a viewport of any size
    When the office renders
    Then the internal resolution is 384x216
    Then the scale factor is an integer
    Then the canvas uses image-rendering pixelated with no fractional blur

  @E2E-156 @suite-office
  Scenario: E2E-156 Retired agents keep a desk and the door opens the retired list
    Given a retired agent
    When I open the office and click the exit door
    Then the retired agent appears as an empty desk with a plaque
    Then the door opens the retired agents list
