@security @p0 @sec-001
Feature: Security, exposure and guardrails

  @E2E-165 @suite-platform
  Scenario: E2E-165 The default exposure is loopback with no login
    Given the default configuration
    When the api starts
    Then it binds to 127.0.0.1
    Then no bearer token is required for local requests

  @E2E-166 @suite-platform
  Scenario: E2E-166 Non-loopback binding requires a strong token
    Given AILAB_BIND is not loopback and AILAB_API_TOKEN is missing or shorter than 32 bytes
    When the api starts
    Then it refuses to start with exit code 78
    Then with a valid token every /api and /ws request except /health/live requires it
    Then the UI stores the token in memory and sessionStorage only

  @E2E-167 @suite-platform
  Scenario: E2E-167 Security headers are present
    Given any web response
    When I inspect the headers
    Then CSP, nosniff, Referrer-Policy, Permissions-Policy and COOP are present
    Then no inline or third-party script is allowed by the CSP

  @E2E-168 @suite-platform
  Scenario: E2E-168 Mutations require JSON and an allowed origin
    Given a mutation request with a non-JSON content type or a disallowed Origin
    When I submit it
    Then it is rejected
    Then a same-origin JSON mutation is accepted

  @E2E-169 @suite-platform
  Scenario: E2E-169 Secrets are encrypted with authenticated encryption
    Given a provider key encrypted with AES-256-GCM and an AAD of table:column:row:key_version
    When the ciphertext is moved to another row or column
    Then decryption fails
    Then only last4 is ever returned by the API

  @E2E-170 @suite-platform
  Scenario: E2E-170 Logs and errors never leak secrets
    Given a request containing a bearer token or an API key
    When the request is logged or fails
    Then the log line and the Problem Details are redacted
    Then no secret appears in any response, log or trace

  @E2E-171 @suite-platform
  Scenario: E2E-171 GitHub webhooks are verified and replay-protected
    Given the webhook secret
    When a delivery has a valid X-Hub-Signature-256
    Then it is accepted
    Then an invalid or stale signature is rejected with 401
    Then a duplicate X-GitHub-Delivery is acknowledged once with 202

  @E2E-172 @suite-platform
  Scenario: E2E-172 The sandbox blocks escape and unapproved egress
    Given a run executing in the sandbox
    When the workload tries to access the host filesystem, the Docker socket, a non-allowlisted host, escalate privileges, fork-bomb or fill the disk
    Then each attempt is denied
    Then the container is non-root, read-only, without host mounts and with network none by default
