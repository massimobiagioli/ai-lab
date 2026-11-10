@realtime @p0 @ws-001
Feature: WebSocket realtime and event replay

  @E2E-139 @suite-platform
  Scenario: E2E-139 The WebSocket handshake returns hello metadata
    Given an authenticated realtime client
    When I connect to GET /api/v1/ws
    Then the server sends hello with server_time, seq and topics
    Then the envelope format matches the AsyncAPI contract

  @E2E-140 @suite-platform
  Scenario: E2E-140 Subscribe and unsubscribe are acknowledged
    Given a connected client
    When I subscribe to lab:{id} and card:{id}
    Then the server acknowledges the subscription
    When I unsubscribe from card:{id}
    Then no further card events are delivered to that client

  @E2E-141 @suite-platform
  Scenario: E2E-141 State events are persisted and ephemeral events are not
    Given a connected client
    When a card moves and an agent types
    Then kanban.card.moved is persisted and replayable
    Then agent.typing is coalesced and never persisted

  @E2E-142 @suite-platform
  Scenario: E2E-142 Reconnect resumes from the last applied sequence
    Given a client that disconnects after seq 100
    When it reconnects with since=100
    Then only events with seq greater than 100 are delivered
    Then duplicate event ids are removed and order is by seq

  @E2E-143 @suite-platform
  Scenario: E2E-143 A gap larger than the buffer triggers HTTP backfill
    Given a reconnect whose gap exceeds the in-memory buffer
    When the client resumes
    Then it calls GET /api/v1/events?after={seq}
    Then the events are applied in order without loss

  @E2E-144 @suite-platform
  Scenario: E2E-144 Backpressure and heartbeat are enforced
    Given a slow client with a full 256-message buffer
    When more events arrive
    Then ephemeral events are dropped first
    Then the connection closes with code 1013 if state events cannot be delivered
    Then the server pings every 20 seconds and expects a pong within 10 seconds
