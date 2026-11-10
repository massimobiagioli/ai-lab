@office @visual @p1 @ux-001
Feature: Office layout, furniture and desks

  @E2E-265 @suite-office
  Scenario: E2E-265 Sixteen agents are visible without overlap
    Given a lab with sixteen active agents
    When the office renders
    Then every agent is visible at a desk, hot desk or standing position
    Then no two agents overlap
    Then no agent is hidden behind furniture

  @E2E-266 @suite-office
  Scenario: E2E-266 Five persistent desks expose identity and state
    Given the default office layout
    When the desks render
    Then five desks are present
    Then each plaque shows name, role icon and status LED
    Then each desk keeps its position across edits

  @E2E-267 @suite-office
  Scenario: E2E-267 Hot desks and standing positions are used when desks are full
    Given more agents than the five permanent desks
    When the office renders
    Then the additional agents occupy hot desks or standing positions
    Then the layout remains navigable
    Then each hot desk exposes the same inspector interaction

  @E2E-268 @suite-office
  Scenario: E2E-268 Furniture participates in collision and pathfinding
    Given a target tile behind a desk or a plant
    When an agent walks to it
    Then the path routes around the furniture footprint
    Then the agent never passes through a non-walkable tile
    Then the path is recomputed after a layout change

  @E2E-269 @suite-office
  Scenario: E2E-269 Every interactive object is clickable
    Given the office is open
    When I click the whiteboard, server rack, bookshelf, filing cabinet, meeting table, door, coffee machine, plant and pet
    Then each object opens or triggers its documented interaction
    Then decorative objects only play a cosmetic effect
    Then no interactive object is a dead zone

  @E2E-270 @suite-office
  Scenario: E2E-270 Y-sorting places characters correctly against furniture
    Given an agent behind a desk and an agent in front of it
    When the frame renders
    Then the agent behind is drawn before the desk
    Then the agent in front is drawn after the desk
    Then the z-order is stable across frames

  @E2E-271 @suite-office
  Scenario: E2E-271 A retired agent leaves an empty desk with a plaque
    Given a retired agent
    When the office renders
    Then the desk is empty
    Then the plaque still shows the retired agent identity
    Then the door opens the retired-agent list

  @E2E-272 @suite-office
  Scenario: E2E-272 Spawning does not teleport an existing agent
    Given a new agent is added
    When it appears in the office
    Then it spawns at a free tile
    Then existing agents are not teleported
    Then the only allowed instant movement is spawn or crash recovery

  @E2E-273 @suite-office
  Scenario: E2E-273 Furniture footprints match the manifest
    Given the furniture manifest
    When each footprint is compared with its hitbox
    Then the footprint size and collision map match
    Then there is no invisible blocker outside the footprint
    Then no visible furniture is walkable by mistake

  @E2E-274 @suite-office
  Scenario: E2E-274 Layout changes invalidate the path cache safely
    Given an agent with a cached path
    When the layout changes
    Then the path cache is invalidated
    Then the agent recomputes a valid path
    Then no agent remains stuck on a removed tile
