# Security Policy

## Supported versions

AI-Lab is pre-release until v1.0.0. Security fixes are applied to `main`; no long-lived release branches are supported before v1.0.0.

## Reporting a vulnerability

Do not open a public issue for a security vulnerability. Use a private security advisory on the repository or contact the Human Operator through the private channel defined at deployment. Include:

- affected component and version;
- reproduction steps or a proof of concept;
- impact and any known exploitation;
- suggested mitigation.

The team acknowledges within 24 hours and provides a remediation plan within 72 hours for P0/P1.

## Security requirements

- Secrets are encrypted at rest with AES-256-GCM and are write-only.
- No secret is logged, returned by an API, emitted in an event, placed in a prompt or made available to a sandbox.
- Sandboxes run non-root with a read-only root filesystem, dropped capabilities, no host mounts and allowlisted egress.
- Webhooks are verified with HMAC and deduplicated.
- Dependencies, images and lockfiles are pinned; `cargo-deny`, `npm audit`, gitleaks and Trivy run in CI.
- A secret found in history is a P0: revoke, rotate, purge, then reopen the change.

## Incident response

P0 and P1 incidents follow `.docs/product/failure-modes.md` and the runbooks in `.docs/product/runbooks/`. A post-incident report is due within 48 hours.
