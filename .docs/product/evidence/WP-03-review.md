# WP-03 Independent Review

| Field | Value |
|---|---|
| Reviewer | review-agent (independent subagent) |
| Author | Dev (agent) |
| Date | 2026-11-07 |
| Scope | Dockerfiles, Compose topology, networks, volumes, healthchecks, proxies, backup/restore, operations docs |
| Verdict | `APPROVED` |

The review was iterative: the first pass returned `CHANGES_REQUESTED` for four blocking defects; the author fixed them and the final pass approved.

## Final acceptance results

| AC | Result | Evidence |
|---|---|---|
| AC1 Compose config clean; only `web` publishes a host port | PASS | `make compose-validate` all profiles; rendered core config publishes only `web:127.0.0.1:8080`; TLS adds `caddy`. |
| AC2 `make up` healthy ≤ 10 min cold | DEFERRED | WP-04 owns migrations/seed and startup smoke. |
| AC3 failing migration blocks api/worker | DEFERRED | Compose ordering verified (`migrate restart:"no"`, `service_completed_successfully`); migration content is WP-04. |
| AC4 Postgres/Redis unreachable from edge/sandbox | PASS | `.tools/test-compose-isolation.sh` 8 services. |
| AC5 runner proxy allowlist and sandbox hardening | DEFERRED | Proxy allowlist and socket `:ro` verified; sandbox-container hardening is WP-14. |
| AC6 egress allowlist / arbitrary domain denied | DEFERRED | Static Squid/allowlist verified; runtime enforcement is WP-14. |
| AC7 image budgets, Trivy, SBOM, signature | DEFERRED | Budgets defined, all bases digest-pinned; build/scan/sign are WP-22. |
| AC8 backup dump/manifest and restore drill | DEFERRED | Scripts delivered; drill needs the WP-04 schema. |
| AC9 SIGTERM ≤ 20 s without losing a run | DEFERRED | `stop_grace_period` set; run lifecycle is WP-13. |
| AC10 `make doctor` actionable | PASS | Busy-port path reproduced; stopped-Docker and missing-key paths inspected. |

## Blocking defects found and resolved

1. Runner image omitted the required toolchains. Resolved: rebuilt on `debian:trixie-slim` with Rust 1.99.1, Node 24, Python 3.12, build-essential, pkg-config, make, just, ripgrep and jq.
2. No image digest pins. Resolved: every third-party Compose image and Dockerfile base is `tag@sha256`.
3. `.tools/test-compose-isolation.sh` was used but unregistered. Resolved: added to the AGENTS tool registry.
4. The worker readiness probe omitted the runner. Resolved: the runner exposes a readiness listener (`AILAB_RUNNER_BIND`), the worker probes `AILAB_RUNNER_URL`, and CR-0006 records both variables.

## Reproduced evidence

- `scripts/harness-lint.sh` → 16 checks.
- `scripts/sow-lint.sh` → 31 checks.
- `.tools/check-doc-links.sh` → 142 links.
- `make compose-validate` → all profiles valid.
- `.tools/test-compose-isolation.sh` → 8 services.
- `cargo fmt --all --check`, `cargo clippy -D warnings`, `cargo test --workspace` → 131 passed.

## Reviewer statement

The WP-03 artifacts satisfy the infrastructure contract. The deferred criteria are downstream verifications owned by WP-04, WP-13, WP-14 and WP-22; the artifacts they verify are delivered. Approved for WP closure.
