# WP-03 — Infrastructure, Docker & Runtime Platform Review Plan

| Field | Value |
|---|---|
| WP | `WP-03` |
| Contract | `.docs/sow/WP-03-INFRASTRUCTURE.md` |
| Review branch | `review/wp-03-infrastructure` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_review` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present and consistent.
- [x] Requirement traceability cited in §9 maps to the specification (§4, §5, §16, §18, §20) and Appendix D §D.1/§D.4.

## 2. Specification conformance

- [x] Specification §4/§5 topology and §16 sandbox — evidence: `infra/compose/docker-compose.yml`, `infra/docker/*.Dockerfile`, `.tools/test-compose-isolation.sh`.
- [x] Appendix D §D.1 (M0) and §D.4 (environment) — evidence: `scripts/doctor.sh`, `.env.example`.
- [x] ADR-0020 (Compose deployment) and ADR-0023 (runner control plane) — evidence: `.docs/adr/`, compose, runner image.

## 3. Deliverables

- [x] Five Dockerfiles — evidence: `infra/docker/{api,worker,web,runner,migrate}.Dockerfile`.
- [x] Base + `dev`, `observability`, `tls`, `egress` overrides — evidence: `make compose-validate` green for every profile.
- [x] Nginx SPA/WS/security config — evidence: `infra/nginx/nginx.conf`.
- [x] Caddy TLS config — evidence: `infra/caddy/Caddyfile`.
- [x] Egress allowlist + Squid — evidence: `infra/egress/{allowlist.txt,squid.conf}`.
- [x] Socket proxy minimal allowlist — evidence: compose service + isolation test.
- [x] Healthcheck subcommands — evidence: `crates/infrastructure/src/health.rs`, service health.
- [x] `.env.example` aligned (40/40, CR-0004) — evidence: WP-02 review.
- [x] `doctor.sh` — evidence: script categories; host port-conflict fails actionably.
- [x] Backup/restore scripts — evidence: `scripts/backup.sh`, `scripts/restore-drill.sh`; drill after WP-04.
- [x] Image build/scan/SBOM/sign hooks — evidence: `.github/workflows/images.yml`.
- [x] Compose validation + isolation tests wired in CI — evidence: `ci.yml` `docs-build`.
- [x] Operations docs — evidence: `.docs/product/operations.md`.
- [x] Resource-limit and network-isolation tests — evidence: `.tools/test-compose-isolation.sh` (8 services).

## 4. Acceptance criteria

- [x] AC1 `docker compose config` clean; only `web` publishes a host port (`caddy` in TLS) — evidence: `make compose-validate`; rendered config shows one `127.0.0.1:8080` port.
- [x] AC2 `make up` healthy ≤ 10 min cold / ≤ 90 s warm — DEFERRED to WP-04 for migrations/seed; the stack is currently running healthy.
- [x] AC3 failing migration blocks api/worker and exits non-zero — DEFERRED to WP-04 (owns migration content); compose ordering and `service_completed_successfully` dependency inspected.
- [x] AC4 Postgres/Redis unreachable from `edge`/sandbox — evidence: `.tools/test-compose-isolation.sh`.
- [x] AC5 runner restricted to the proxy allowlist; sandbox no host mounts, no socket, read-only, cap drop, non-root — evidence: compose + isolation test + runner Dockerfile.
- [x] AC6 egress allowlist reachable, arbitrary domain denied — evidence: `infra/egress/*`; runtime enforcement owned by WP-14.
- [x] AC7 image budgets, Trivy clean, SBOM/sign per image — DEFERRED to WP-22 for SBOM/sign; build/scan hooks present.
- [x] AC8 backup dump/manifest + restore drill — DEFERRED to WP-04 for the schema.
- [x] AC9 SIGTERM exits ≤ 20 s without losing an in-flight run — DEFERRED to WP-13 (run lifecycle); compose `stop_grace_period` inspected.
- [x] AC10 `make doctor` actionable on stopped Docker, busy port or missing key — evidence: doctor categories; busy-port path reproduced on this host.

## 5. Implementation and tests

- [x] Compose validation — `make compose-validate` green.
- [x] Isolation/resource limits — `.tools/test-compose-isolation.sh` 8 services.
- [x] No real external API in tests.
- [x] Coverage thresholds met (infrastructure WP; scripts covered by the isolation/validation tests).

## 6. Security and quality gates

- [x] Sandbox isolation, socket-proxy allowlist and no host mounts verified.
- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `make compose-validate` and `.tools/test-compose-isolation.sh` green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/operations.md` updated.
- [x] `.docs/adr/registry.md` consistent (ADR-0020, ADR-0023).
- [x] `AGENTS.md` indexes updated for this review plan.
- [x] `.docs/sow/SOW.md` and WP-03 status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED`.
- [ ] QA evidence: reproduced gates (harness 16, sow-lint 31, doc links 141, compose-validate all profiles, isolation 8 services, rendered config).

## 9. Deferred items and residual risks

- [x] AC2/AC3/AC8 startup, migration failure and backup drill — owner: Dev, WP-04.
- [x] AC7 SBOM/sign and AC9 graceful run termination — owners: WP-22 and WP-13.
- [x] No unexplained deferral remains.

## 10. Decisions

- [x] ADR-0020 Compose deployment and ADR-0023 runner control plane — recorded.
