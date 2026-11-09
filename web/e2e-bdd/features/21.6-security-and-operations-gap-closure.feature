@security @operations @p1 @sec-003
Feature: Security and operations gap closure

  @E2E-231
  Scenario: E2E-231 TLS is terminated only by an explicit proxy profile
    Given the optional TLS profile
    When I start the stack with it
    Then TLS terminates at the proxy and the app does not generate self-signed certificates silently
    Then the API security headers and HSTS are present when TLS is active

  @E2E-232
  Scenario: E2E-232 Sandbox shell and credentials are least-privilege
    Given a run in the sandbox
    When the agent requests a command outside the allowlist, such as curl piped to sh
    Then the command is blocked
    Then package installation requires the egress allowlist and is logged
    Then only a short-lived repo-scoped token is injected, then revoked after the run
    Then no production credential is available inside the sandbox

  @E2E-233
  Scenario: E2E-233 The audit log covers sensitive actions and is exportable
    Given provider key create, rotate and delete, agent create and retire, autonomy and budget changes, merges, project connect and disconnect, manual overrides, memory deletion and settings changes
    When each action occurs
    Then an append-only audit row records actor, action, target, safe before and after, trace id and timestamp
    Then the audit log can be exported without exposing secrets

  @E2E-234
  Scenario: E2E-234 Data export, deletion and retention are honored
    Given a lab with cards, runs, memory vectors, chunks and artifacts
    When the operator exports all data and then deletes the lab with typed confirmation
    Then the export contains the relational data and artifact references
    Then deletion removes rows, vectors, chunks and artifacts, is audited and is asynchronous
    Then a prompt or response deletion request completes within 24 hours including embeddings

  @E2E-235
  Scenario: E2E-235 Configuration and images are hardened and fail fast
    Given an invalid AILAB_* value or a missing master key
    When the process starts
    Then it exits with a clear message and never logs the secret
    Then every Compose service has CPU, memory and pid limits, a healthcheck and a pinned image digest
    Then the release image has an SBOM and a signature
