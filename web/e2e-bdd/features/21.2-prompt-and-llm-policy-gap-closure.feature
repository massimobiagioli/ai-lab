@llm @agents @p1 @fr-llm-010
Feature: Prompt and LLM policy gap closure

  @E2E-208 @suite-platform
  Scenario: E2E-208 Prompt assembly is deterministic and reproducible
    Given the same role, character, CV, hard rules, tools, task packet and untrusted context
    When the prompt is assembled twice
    Then the rendered sections appear in the fixed Appendix C order
    Then prompt_hash is identical for identical inputs and differs when an input changes
    Then the character section is capped at 400 tokens, with role rules winning and character_override logged on conflict

  @E2E-209 @suite-platform
  Scenario: E2E-209 Role tool allowlists deny and log policy violations
    Given a Dev, an Analyst, a QA and a Tech Lead
    When each calls a tool outside its role allowlist
    Then the call returns POLICY_DENIED
    Then the denial is logged with run id, agent id and tool name
    Then every allowed tool call is recorded in the run steps

  @E2E-210 @suite-platform
  Scenario: E2E-210 Model routing justifies escalation
    Given a default classification request
    When it is routed
    Then it uses deepseek-flash
    Then given the SOW architecture section, a third debugging attempt, a security review or a Tech Lead conflict
    Then deepseek-v4-pro may be used with a recorded justification
    Then a pro use without an allowed reason is rejected

  @E2E-211 @suite-platform
  Scenario: E2E-211 Structured outputs validate and repair exactly once
    Given a structured call against a JSON Schema
    When the model returns invalid JSON
    Then one repair request is sent
    Then a second invalid response fails with InvalidOutput
    Then a valid response is parsed into the typed value before use

  @E2E-212 @suite-platform
  Scenario: E2E-212 Token budgets allocate context and summarize overflow
    Given a model context window with system, memory, repo, task and reserve budgets
    When the context exceeds a section budget
    Then system is capped at 20 percent, memory at 25, repo at 35, task at 15 and reserve at 5
    Then overflow is summarized or omitted by priority
    Then the task packet is never silently dropped

  @E2E-213 @suite-platform
  Scenario: E2E-213 Streaming, retries and untrusted content are safe
    Given a UI-visible SOW or plan generation
    When tokens stream to the client
    Then deltas are coalesced at 50 ms
    Then 429 and 5xx responses retry with exponential backoff and Retry-After
    Then the circuit opens after five consecutive failures and half-opens after 60 seconds
    Then untrusted content is wrapped and cannot break out of its wrapper
    Then prompts and responses are persisted redacted with the configured retention
