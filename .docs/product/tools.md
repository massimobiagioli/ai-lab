# AI-Lab Tools

| Field | Value |
|---|---|
| Document | `.docs/product/tools.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/sandbox.md`, `.docs/product/security.md` |

Tool calls are denied by default. The policy engine authorizes by role and validates the command, the working directory and every written path before execution.

## Rules

| Rule | Meaning |
|---|---|
| `EMPTY_COMMAND` | No argv. |
| `FORBIDDEN_COMMAND` | Command is on the global deny list. |
| `COMMAND_NOT_ALLOWED` | Command is not in the role allowlist. |
| `FORCE_PUSH` | `git push --force`/`-f`. |
| `PUBLISH_FORBIDDEN` | `cargo publish`/`npm publish`. |
| `PATH_TRAVERSAL` | Absolute path or `..`. |
| `PROTECTED_PATH` | `.github/workflows`, `CODEOWNERS`, `.env`, `secrets`, `.git/config`. |

The full 20+ tool registry with JSON Schemas and the 8-stage evaluation is delivered in the WP-14 follow-up; this document is the policy contract.
