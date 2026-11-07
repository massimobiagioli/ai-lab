---
name: Docker & Runtime Operations
description: Use when changing Dockerfiles, Compose profiles, networks, volumes, healthchecks, runner, sandbox, proxies or local bootstrap.
---

# Docker & Runtime Operations

## When to use

- A Dockerfile, image, Compose service, profile, network, volume or port changes.
- The runner, sandbox, socket proxy or egress proxy changes.
- A healthcheck, resource limit, restart policy or logging configuration changes.
- Local bootstrap or `make up` behavior changes.

## Preconditions

- Read `.docs/sow/WP-03-INFRASTRUCTURE.md` and `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §20.
- Read `.docs/sow/WP-14-SANDBOX-RUNNER.md` for any runner or sandbox change.
- Docker Engine 27 or later and Compose 2.30 or later are available.
- No secret is present in a Dockerfile, Compose file or build argument.

## Workflow

1. Write a failing check: a Compose validation test, a healthcheck test, an isolation test or a smoke test.
2. Change the Dockerfile or Compose file.
3. Validate:
   - `docker compose config --quiet`
   - `docker compose build`
   - `docker compose up -d --wait`
   - `make doctor`
4. Verify isolation:
   - only `web` publishes a port in the production profile;
   - Postgres and Redis are not on the edge network;
   - no service uses `privileged`, host network or host mounts;
   - the sandbox has no Docker socket and no host mount.
5. Verify resource limits and healthchecks match the specification.
6. Verify image budgets, Trivy scan, SBOM and signature.
7. Verify graceful shutdown: `docker compose stop` completes within 20 seconds.
8. Verify backup and restore commands still work.
9. Record the commands and results in the WP progress line.

## Checklist

- [ ] Compose config valid.
- [ ] Images build within size budgets.
- [ ] Non-root user and read-only root filesystem where required.
- [ ] Capabilities dropped; no new privileges.
- [ ] Only the documented host ports are published.
- [ ] Networks isolate edge, backbone, runner and sandbox.
- [ ] Healthchecks and `depends_on` conditions correct.
- [ ] Resource limits set for every service.
- [ ] No secrets in build args, images or logs.
- [ ] Trivy, SBOM and signature green.
- [ ] Graceful shutdown verified.
- [ ] Backup and restore verified.

## Anti-patterns

- Using `latest` tags instead of pinned tags and digests.
- Mounting the Docker socket into a workload.
- Publishing Postgres or Redis to the host in production.
- Running a container as root for convenience.
- Adding `privileged: true`.
- Adding an unbounded log configuration.
- Silently changing a volume name or path.

## References

- `../../../.docs/sow/WP-03-INFRASTRUCTURE.md`
- `../../../.docs/sow/WP-14-SANDBOX-RUNNER.md`
- `../../../.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §20
- `../../../.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §16.3

## Supporting files

- None.
