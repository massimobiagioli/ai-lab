@delivery @orchestration @p1 @fr-orc-010
Feature: Delivery and orchestration gap closure

  @E2E-214 @suite-delivery
  Scenario: E2E-214 Agent commits and pull requests are traceable
    Given a Dev agent commit
    When the commit and PR are created
    Then the commit uses a Conventional Commit type and scope
    Then the commit footer carries AI-Lab-Run, AI-Lab-Card and AI-Lab-Agent
    Then the PR title is "[{CARD_KEY}] {TITLE}"
    Then the PR body includes the full AI-Lab trace block

  @E2E-215 @suite-delivery
  Scenario: E2E-215 Context build is complete and bounded
    Given a READY card
    When the run builds context
    Then it includes the card, repo map, memory hits, relevant files, conventions and previous review feedback
    Then the initial file payload does not exceed 120 KB or 30k tokens
    Then the task packet is always present

  @E2E-216 @suite-delivery
  Scenario: E2E-216 The agent loop is bounded and plans before mutating
    Given a run that reaches the implement stage
    When the loop runs
    Then it makes at most 40 tool calls and 25 iterations within 45 minutes
    Then a plan RunStep is persisted and streamed before any file is written
    Then a response never exceeds 8192 output tokens

  @E2E-217 @suite-delivery
  Scenario: E2E-217 Verification runs before commit and feeds failures back
    Given a run ready to commit
    When verification runs in the sandbox
    Then formatter, linter, type check, unit tests and secret scan execute in order
    Then a failure returns the raw output, truncated to 20 KB, to the agent
    Then a run cannot commit while verification is red

  @E2E-218 @suite-delivery
  Scenario: E2E-218 Run records and step traces are complete
    Given a completed run
    When I read the run and its steps
    Then the run records card, agent, attempt, status, lease, sandbox, tokens, cost, error and result summary
    Then each step records type, arguments hash, duration, exit status and capped output
    Then all steps share one trace_id

  @E2E-219 @suite-delivery
  Scenario: E2E-219 Handoffs, escalations and human decisions are structured
    Given an agent that cannot finish its card
    When it hands off or escalates
    Then the Handoff object contains from, to, card, summary, state, open questions, artifacts and acceptance status
    Then free-form chat is not accepted as a handoff
    Then escalation includes the last 20 log lines, the failing command and a proposed next action
    Then a human intervention is recorded as a Decision visible to the next agent
