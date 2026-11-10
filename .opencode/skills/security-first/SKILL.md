---
name: Security-First Engineering
description: Use for any change touching authentication, secrets, sandbox, LLM prompts, SCM, webhooks, data exposure, dependencies or external input.
---

# Security-First Engineering

## When to use

- A change touches auth, exposure mode, secrets, encryption, key rotation or audit.
- A change touches the sandbox, runner, tool policy, egress or shell execution.
- A change touches prompts, retrieval, tool schemas or model output handling.
- A change touches GitHub, webhooks, tokens or repository writes.
- A change adds a dependency, image, script or external host.

## Preconditions

- Read the threat model in `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §16 and `.docs/sow/WP-05-SECURITY.md`.
- Identify the trust boundaries crossed by the change.
- Identify the assets involved: secrets, repository content, prompts, memory, user data.
- Confirm the change does not widen exposure beyond the approved mode.

## Workflow

1. Write the abuse cases: who can attack, what they control, what they gain.
2. Map each abuse case to a control. If a control is missing, add it or block the change.
3. Implement with secure defaults:
   - loopback bind unless a bearer token and TLS are configured;
   - AES-256-GCM with AAD for secrets;
   - constant-time comparison for tokens and signatures;
   - deny-by-default tool policy and egress allowlist;
   - non-root, read-only, capability-dropped sandbox.
4. Add tests for the control, including the negative case.
5. Run the security gates:
   - `gitleaks detect --source . --no-git`
   - `cargo deny check`
   - `scripts/run-web.sh npm audit --audit-level=high`
   - Trivy on changed images
   - the sandbox escape suite
   - the prompt-injection fixtures
   - the cross-scope memory leakage test
6. Update the security documentation and the audit coverage matrix.
7. If a security exception is required, create an ADR with an expiry date and obtain QA and Human Operator approval.

## Checklist

- [ ] Threat model and abuse cases written for the change.
- [ ] No secret in code, logs, events, errors, prompts, fixtures or sandbox.
- [ ] Secrets encrypted with AES-256-GCM and AAD.
- [ ] Auth and exposure mode enforced server-side.
- [ ] Input validated; SQL parameterized; output encoded.
- [ ] Tool policy and egress allowlist enforced server-side.
- [ ] Sandbox isolation controls verified.
- [ ] Webhook signature and replay protection verified where applicable.
- [ ] Supply-chain scans green; SBOM updated when images change.
- [ ] Audit events cover every sensitive action.
- [ ] Negative security tests added.
- [ ] Documentation and runbooks updated.

## Anti-patterns

- Relying on the model or the UI to enforce policy.
- Logging a request body that may contain a secret.
- Storing a token in `localStorage`, a URL or a commit.
- Mounting the Docker socket into a workload container.
- Adding an allow-all egress rule for convenience.
- Disabling a security test to unblock a build.
- Creating a security exception without an owner and an expiry.

## References

- `../../../.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §16
- `../../../.docs/sow/WP-05-SECURITY.md`
- `../../../.docs/sow/WP-14-SANDBOX-RUNNER.md`
- `../../../.docs/sow/WP-12-SCM-GITHUB.md`
- `../../../.docs/PRINCIPLES.md` §2

## Supporting files

- None.
