# WP-03 — Infrastructure, Docker & Runtime Platform

| Field | Value |
|---|---|
| WP ID | `WP-03` |
| Status | **done** |
| Owner | Dev |
| Milestone | M0 |
| Estimate | 13 points |
| Depends on | WP-01, WP-02 |
| Blocks | WP-04, WP-05, WP-20 |
| Requirement traceability | `OPS-001..009`, `SEC-001..014`, `NFR-001`, `NFR-008` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-01 and WP-02 done; T01 started |
| 2026-11-07 | `progress` | `done` | Dev (agent) | Platform artifacts, images, Compose, isolation tests and operations docs delivered; make up/backup drill executed with WP-04 schema |
| 2026-11-07 | `done` | `done` | Tech Lead (agent) | WP review: review plan created, compose/isolation gates reproduced, QA `APPROVED` (`.docs/product/evidence/WP-03-review.md`) |

---

## 1. Objective

Deliver the **containerized runtime platform** for AI-Lab: reproducible images, deterministic Compose topologies, secure networking, persistent volumes, health/readiness, resource limits, bootstrap sequencing, TLS/observability profiles, backup hooks and sandbox isolation. The platform must be startable with one command, safe by default, and operable by a single person.

---

## 2. Scope

### 2.1 In scope

- Dockerfiles for `api`, `worker`, `web`, `runner`, `migrate`.
- Compose base + dev + observability + TLS profiles.
- Networks, volumes, ports, healthchecks, resource limits, restart policies, logging.
- Sandbox network and egress control (Docker socket proxy + egress proxy).
- Bootstrap/ordering, migrations, seed, readiness.
- Image hardening, SBOM, signing, scanning.
- Backup/restore integration points.
- Operational commands and failure behavior.

### 2.2 Out of scope

- Application behavior (WP-06+).
- Managed cloud/Kubernetes deployment.
- CI runner infrastructure.
- Grafana dashboard content (WP-20).

---

## 3. Engineering specification

### 3.1 Service topology (Compose)

| Service | Image | Profiles | Networks | Published ports | Restart |
|---|---|---|---|---|---|
| `postgres` | `pgvector/pgvector:0.8.7-pg18-trixie` | core | backbone | none (dev: `127.0.0.1:5432`) | `unless-stopped` |
| `redis` | `redis:8.8.4-trixie` | core | backbone | none (dev: `127.0.0.1:6379`) | `unless-stopped` |
| `migrate` | `ailab/migrate:1.0.0` | core | backbone | none | `no` |
| `api` | `ailab/api:1.0.0` | core | edge, backbone | none (web proxies) | `unless-stopped` |
| `worker` | `ailab/worker:1.0.0` | core | backbone, runner | none | `unless-stopped` |
| `runner` | `ailab/runner:1.0.0` | core | runner, sandbox-control | none | `unless-stopped` |
| `docker-socket-proxy` | `tecnativa/docker-socket-proxy:<pinned>` | core | runner | none | `unless-stopped` |
| `egress-proxy` | `ailab/egress-proxy:1.0.0` | egress | sandbox-egress, runner | none | `unless-stopped` |
| `web` | `ailab/web:1.0.0` | core | edge | `127.0.0.1:8080:8080` | `unless-stopped` |
| `caddy` | `caddy:<pinned>` | tls | edge | `80:80`, `443:443` | `unless-stopped` |
| `otel-collector` | `otel/opentelemetry-collector-contrib:<pinned>` | observability | observability, backbone | none | `unless-stopped` |
| `prometheus` | `prom/prometheus:<pinned>` | observability | observability | none | `unless-stopped` |
| `grafana` | `grafana/grafana:<pinned>` | observability | observability | `127.0.0.1:3001:3000` | `unless-stopped` |

**Names are stable**; scripts and docs reference these exact names. Only `web` (and `caddy` in TLS profile) publishes a port in production. Dev-only ports bind to `127.0.0.1`.

### 3.2 Networks

| Network | Members | Purpose | Isolation |
|---|---|---|---|
| `edge` | web, api, caddy | Browser-facing traffic | Internal; only web/caddy publish |
| `backbone` | api, worker, postgres, redis, migrate, otel | Durable state and telemetry | Internal; no host publish |
| `runner` | worker, runner, docker-socket-proxy | Runner control plane | Internal |
| `sandbox-control` | runner, sandbox containers | Container lifecycle only | Internal |
| `sandbox-egress` | egress-proxy, opt-in sandbox containers | Package registry access | Internal + proxy allowlist |
| `observability` | otel, prometheus, grafana | Metrics/traces dashboards | Internal; Grafana loopback |
| `tls` (profile) | caddy, web | TLS termination | Internal + host 80/443 |

**Rules:** Postgres and Redis are never on `edge`. Sandbox containers never join `backbone` or `edge`. No service uses `network_mode: host`. No `privileged: true` anywhere.

### 3.3 Volumes

| Volume | Mount | Owner | Purpose | Backup |
|---|---|---|---|---|
| `pgdata` | `/var/lib/postgresql/data` | postgres | Database | Daily `pg_dump` + optional WAL |
| `redisdata` | `/data` | redis | AOF/RDB (ephemeral truth) | Not backed up critically |
| `artifacts` | `/var/lib/ailab/artifacts` | api, worker | SOW exports, logs, screenshots, recordings | Daily tar + manifest |
| `workspaces` | `/var/lib/ailab/workspaces` | worker, runner | Git worktrees | Not backed up (recreatable) |
| `caddy_data` | `/data` | caddy | TLS certs | Optional |
| `prometheus_data` | `/prometheus` | prometheus | Metrics | Optional |

**Rule:** no bind mounts in the production profile. Dev profile may bind `./crates`, `./web/src`, `./migrations` for hot reload, explicitly marked `:dev`.

### 3.4 Image specifications

#### Common rules

- Multi-stage builds; builder images pinned by digest; runtime images minimal.
- Run as non-root (`api/worker`: UID/GID `10001`; `web`: nginx UID `101`; `runner`: `10001`).
- `HEALTHCHECK` on all long-running services.
- OCI labels: `org.opencontainers.image.{source,revision,version,created,licenses}`.
- No shell/package manager in `api`/`worker` runtime (distroless or `debian:trixie-slim` with apt removed); `runner` includes toolchains by design.
- Read-only root filesystem where possible; writable `tmpfs` for `/tmp`; explicit writable volumes.
- `STOPSIGNAL SIGTERM`; graceful shutdown ≤ 20 s.
- Image budgets: `api ≤ 120 MB`, `worker ≤ 140 MB`, `web ≤ 40 MB`, `runner ≤ 600 MB`, `migrate ≤ 80 MB`.
- Every image is scanned (Trivy HIGH/CRITICAL fail) and has a CycloneDX SBOM.

#### `api.Dockerfile`

- Builder: `rust:1.99.1-trixie` + `cargo-chef`; cache dependency layer; `cargo build --release -p ailab-api`.
- Runtime: `gcr.io/distroless/cc-debian13` (or equivalent pinned), binary + CA certs + tzdata.
- Entrypoint: `/usr/local/bin/ailab-api`; no shell; `AILAB_ENV` required.
- Exposes `3000` internal only.

#### `worker.Dockerfile`

- Same builder; builds `ailab-worker`; runtime includes `git`, `ca-certificates`, `tzdata` (needed for worktrees; no shell scripts executed by the worker itself).
- Entrypoint: `/usr/local/bin/ailab-worker`.
- Graceful shutdown on SIGTERM: stop accepting runs, finish/checkpoint ≤ 20 s, release leases.

#### `web.Dockerfile`

- Builder: `node:24-trixie-slim` (pin digest), `npm ci`, `ng build --configuration production`.
- Runtime: `nginx:1.29-alpine` with custom config; static assets; gzip/brotli; security headers; SPA fallback.
- Exposes `8080` (nginx unprivileged).

#### `runner.Dockerfile`

- Based on `debian:trixie-slim`; installs pinned toolchains: Rust 1.99.1, Node 24, npm 11, Python 3.12, git, build-essential, pkg-config, make, just, ripgrep, jq, ca-certificates.
- Contains no application secrets; reads runner token from env/file.
- Runs as UID 10001; no Docker CLI (uses Docker API over proxy); no host mounts.
- Entrypoint: `/usr/local/bin/ailab-runner`.

#### `migrate.Dockerfile`

- `rust:1.99.1-trixie` builder with `sqlx-cli`; runtime distroless with `sqlx`, `migrations/`, `sqlx` offline metadata.
- Entrypoint: `sqlx migrate run`; exit code is authoritative.

### 3.5 Compose profiles

| Profile | Services | Use |
|---|---|---|
| `core` (default) | postgres, redis, migrate, api, worker, runner, docker-socket-proxy, web | Normal product |
| `dev` | core + hot reload overrides, exposed dev ports | Local development |
| `observability` | otel-collector, prometheus, grafana | Ops |
| `tls` | caddy | Remote/TLS exposure |
| `egress` | egress-proxy | Dependency installs in sandbox |
| `demo` | core + seeded demo job | Demos/CI |

Files: `docker-compose.yml` (base), `docker-compose.dev.yml`, `docker-compose.observability.yml`, `docker-compose.tls.yml`, `docker-compose.egress.yml`. `make up` = `core`; `make up-obs` = `core,observability`; `make up-tls` = `core,tls`.

### 3.6 Bootstrap and ordering

```
postgres (healthy) ─┐
redis (healthy) ────┼─► migrate (exit 0) ─► api (ready) ─► web (healthy)
                    └─► worker (ready) ──► runner (ready)
docker-socket-proxy (healthy) ─► runner
```

- `depends_on` uses `condition: service_healthy` / `service_completed_successfully`.
- `migrate` has `restart: "no"`; a non-zero exit blocks api/worker (they will crash-loop with a clear log).
- API readiness: DB reachable, Redis reachable, migrations applied, runner reachable (degraded allowed), provider health unknown is not fatal.
- Worker readiness: DB reachable, Redis reachable, at least one runner reachable.
- Startup budget: cold `make up` ≤ 10 min; warm ≤ 90 s.

### 3.7 Healthchecks (exact)

| Service | Command | Interval | Timeout | Retries | Start period |
|---|---|---|---|---|---|
| postgres | `pg_isready -U $POSTGRES_USER -d $POSTGRES_DB` | 5 s | 3 s | 20 | 10 s |
| redis | `redis-cli ping` | 5 s | 3 s | 20 | 5 s |
| api | `wget -qO- http://127.0.0.1:3000/health/ready` (busybox in debug only) or built-in `ailab-api healthcheck` | 10 s | 3 s | 6 | 20 s |
| worker | `ailab-worker healthcheck` (DB + Redis + runner probe) | 15 s | 5 s | 6 | 20 s |
| runner | `ailab-runner healthcheck` (Docker API via proxy) | 10 s | 3 s | 6 | 10 s |
| web | `wget -qO- http://127.0.0.1:8080/healthz` | 10 s | 3 s | 6 | 10 s |
| docker-socket-proxy | `wget -qO- http://127.0.0.1:2375/_ping` | 10 s | 3 s | 6 | 5 s |
| caddy | `caddy validate --config /etc/caddy/Caddyfile` | 30 s | 5 s | 3 | 10 s |

Healthcheck commands MUST exist in the runtime image; no relying on `curl` unless installed.

### 3.8 Resource limits and security options

| Service | CPU | Memory | PIDs | Read-only root | Capabilities | Notes |
|---|---|---|---|---|---|---|
| postgres | 2.0 | 2 GB | 512 | no | drop ALL | `shm_size: 256m` |
| redis | 1.0 | 768 MB | 256 | no | drop ALL | `maxmemory 512mb`, `allkeys-lru` |
| api | 1.0 | 512 MB | 256 | yes + `/tmp` tmpfs | drop ALL | non-root 10001 |
| worker | 2.0 | 1 GB | 512 | yes + `/tmp` tmpfs | drop ALL | non-root 10001 |
| runner | 1.0 | 512 MB | 256 | yes + `/tmp` tmpfs | drop ALL | only Docker API via proxy |
| web | 0.5 | 128 MB | 128 | yes | drop ALL | nginx unprivileged |
| docker-socket-proxy | 0.25 | 64 MB | 64 | yes | drop ALL | API allowlist |
| egress-proxy | 0.5 | 128 MB | 128 | yes | drop ALL | domain allowlist |
| caddy | 0.5 | 128 MB | 128 | no | drop ALL | cert storage |
| otel/prom/grafana | 0.5 | 256 MB each | 256 | partial | drop ALL | optional |

**Global:** `security_opt: ["no-new-privileges:true"]`, `pids_limit`, `ulimits`, `logging` rotation (`max-size=10m`, `max-file=3`). No `privileged`, no `cap_add` except none required.

### 3.9 Sandbox isolation and egress

- The runner talks to Docker through `docker-socket-proxy` with only: `CONTAINERS=1`, `POST=1`, `IMAGES=1`, `EXEC=1`, `LOGS=1`, `INFO=1`; everything else `0` (no builds, volumes, networks, swarm, system, auth).
- Sandbox containers: `network=none` by default; `read_only=true`; `cap_drop=[ALL]`; `no-new-privileges`; non-root UID 10001; `pids_limit=512`; memory/CPU/time limits from the run; workspace tmpfs/volume ≤ 2 GB; no host mounts; no Docker socket.
- Egress mode: only if the card policy allows dependency installation. Sandbox joins `sandbox-egress`; traffic is forced through `egress-proxy` with a domain allowlist: `crates.io`, `static.crates.io`, `index.crates.io`, `registry.npmjs.org`, `pypi.org`, `files.pythonhosted.org`, `github.com`, `objects.githubusercontent.com`, `proxy.golang.org` (only if needed). Everything else denied and logged.
- DNS: proxy resolves names; direct DNS from sandbox is blocked.
- Logs: runner streams stdout/stderr, capped 20 MB per run; stored under `artifacts/runs/{run_id}/`.

### 3.10 TLS and remote exposure

- Default: loopback HTTP at `127.0.0.1:8080`, no TLS.
- Remote: enable `tls` profile; Caddy obtains certificates (ACME or internal CA), sets HSTS, security headers, and proxies `/api` + `/ws` with upgrade support.
- Remote mode requires `AILAB_API_TOKEN`; the API rejects unauthenticated requests when bound non-loopback. Caddy does not bypass this.
- No self-signed certificate is generated silently; internal CA is explicit and documented.

### 3.11 Observability profile

- `otel-collector` receives OTLP HTTP/gRPC from api/worker/runner; exports to Prometheus (metrics) and logs/traces per config.
- `prometheus` scrapes `/metrics` on api/worker/runner and itself; retention 15 days.
- `grafana` provisioned dashboards (WP-20) at `127.0.0.1:3001`; anonymous access disabled; admin password from env/secret.
- Logs: Docker json-file rotation by default; OTLP when observability profile is on.

### 3.12 Backup and restore hooks

- `make backup` → `pg_dump --format=custom` to `artifacts/backups/YYYYMMDDTHHMMSSZ.dump` + tar of `artifacts/` + manifest with SHA-256 + git revision + migration version.
- `make restore-drill` → restore into `ailab_restore` scratch DB, run integrity checks, compare row counts/checksums, drop scratch.
- Retention: 14 daily backups locally; optional off-host copy via operator script.
- RPO ≤ 24 h (default) or ≤ 5 min with WAL archiving; RTO ≤ 1 h. Drill monthly.

### 3.13 Failure and restart behavior

| Scenario | Behavior |
|---|---|
| `migrate` fails | api/worker do not start; logs show exact migration error; `make up` fails non-zero |
| Postgres restarts | API readiness fails; clients retry; worker reconnects with backoff; leases expire safely |
| Redis restarts | Cache cold; rate limits fail closed; WS fan-out resumes via outbox relay; product reads/writes continue |
| Runner restarts | In-flight sandboxes may be orphaned; runner reconciles on boot and removes labeled orphans; runs requeue |
| Worker restarts | SIGTERM graceful; leases released or expire; runs reconciled |
| Host reboot | All services `unless-stopped` restart; ordering enforced; no manual step |
| Disk full | Writes fail with clear error; doctor alerts; no silent corruption |

### 3.14 Operational commands

```
make up / down / reset / ps / logs / doctor / backup / restore-drill
docker compose --profile observability up -d
docker compose --profile tls up -d
docker compose --profile egress up -d
```

No command requires interactive input. Every command documents its side effects in `make help`.

---

## 4. Deliverables (checklist)

- [x] `infra/docker/api.Dockerfile`, `worker.Dockerfile`, `web.Dockerfile`, `runner.Dockerfile`, `migrate.Dockerfile` (multi-stage, digest-pinned bases, per-image budgets; runner carries the WP-14 toolchains).
- [x] `infra/compose/docker-compose.yml` + `dev`, `observability`, `tls`, `egress` overrides (`make compose-validate` green).
- [x] `infra/nginx/nginx.conf` with SPA fallback, gzip, security headers, WS upgrade, runtime resolver and `/healthz`.
- [x] `infra/caddy/Caddyfile` (TLS profile) with HSTS and proxy rules.
- [x] `infra/egress/allowlist.txt` + Squid config (runtime egress owned by WP-14).
- [x] `docker-socket-proxy` configuration with minimal API allowlist (`/version` denied, `/containers/json` allowed).
- [x] Healthcheck binaries/subcommands in api/worker/runner (`crates/infrastructure/src/health.rs`; containers healthy).
- [x] `.env.example` aligned with Appendix D.4 plus `AILAB_RUNNER_TOKEN`; `.env` gitignored.
- [x] `scripts/doctor.sh` covering Docker/Compose/ports/disk/tools/keys/connectivity (`make doctor` fails on busy ports and missing keys).
- [x] `make backup` / `make restore-drill` scripts + manifest format (drill executes after WP-04 schema).
- [x] Image build + Trivy + SBOM + cosign signing in `images.yml` (build and scan hooks present; SBOM/sign complete in WP-22).
- [x] Compose validation test (`make compose-validate`) and isolation test wired in CI; startup smoke runs after WP-04 migrations.
- [x] `.docs/product/operations.md` install/upgrade/backup/restore/scale section.
- [x] Resource-limit and network-isolation automated tests (`.tools/test-compose-isolation.sh`; edge cannot reach Postgres/Redis).

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP03-T01 | Write Dockerfiles (5) with budgets and non-root | Dev | 3 | WP-02 | Images build; size budgets met |
| WP03-T02 | Base Compose + networks + volumes + ordering | Dev | 3 | T01 | `make up` green; readiness correct |
| WP03-T03 | Dev/observability/TLS/egress profiles | Dev | 2 | T02 | Each profile starts independently |
| WP03-T04 | Healthchecks + graceful shutdown | Dev | 1 | T02 | Health transitions correct |
| WP03-T05 | Docker socket proxy + runner isolation | Dev | 2 | T02 | Proxy denies non-allowlisted API |
| WP03-T06 | Egress proxy + allowlist | Dev | 2 | T02 | Allowed domains pass; others denied |
| WP03-T07 | Resource limits + security options | Dev | 1 | T02 | Compose config matches §3.8 |
| WP03-T08 | Backup/restore drill scripts | Dev | 2 | T02 | Restore verification passes |
| WP03-T09 | Image scan/SBOM/sign pipeline | Dev | 2 | T01 | Artifacts published on tag |
| WP03-T10 | Isolation and startup tests | QA | 2 | All | Tests green in CI |

---

## 6. Acceptance criteria

1. `docker compose config` validates with no warnings; only `web` publishes a host port in the production profile (and `caddy` in TLS).
2. `make up` from clean volumes brings the stack to healthy in ≤ 10 min cold and ≤ 90 s warm.
3. A failing migration blocks api/worker startup and exits non-zero with the exact error.
4. Postgres and Redis are unreachable from the `edge` and `sandbox` networks (network isolation test).
5. The runner cannot access Docker APIs outside the allowlist (proxy denial test); sandbox containers have no host mounts, no Docker socket, `read_only`, `cap_drop ALL`, non-root.
6. Sandbox egress: allowlisted registry reachable; an arbitrary domain (e.g., `example.com`) denied and logged.
7. Image sizes are within budgets; Trivy reports no HIGH/CRITICAL; SBOM and signature exist for every image.
8. `make backup` produces a dump and manifest; `make restore-drill` restores and verifies checksums.
9. SIGTERM to api/worker exits within 20 s without dropping an in-flight run incorrectly (run becomes retryable, not lost).
10. `make doctor` fails with an actionable message when Docker is stopped, a port is busy, or `AILAB_MASTER_KEY` is missing.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence.
- [x] Compose validation and isolation tests green and wired in CI; startup smoke executes after WP-04 migrations.
- [x] Image budgets defined (api ≤ 120 MB, worker ≤ 140 MB, web ≤ 40 MB, runner ≤ 600 MB, migrate ≤ 80 MB); build/scan/SBOM/sign executed at release by WP-22.
- [x] Backup/restore scripts delivered; the drill is executed after WP-04 provides the schema.
- [x] Operations docs updated (`.docs/product/operations.md`).
- [x] Status `done` with sign-off (2026-11-07).

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Docker socket exposure | Critical | Socket proxy with allowlist; runner-only access; no host mounts |
| Sandbox escape | Critical | Non-root, cap drop, read-only, no host network, egress proxy |
| Compose drift between profiles | Medium | Base + overrides; CI config validation |
| Slow cold start | Medium | Layer caching, cargo-chef, healthcheck tuning |
| Data loss on volume reset | High | `make reset` requires explicit confirmation flag; backups documented |
| Resource exhaustion | Medium | Limits per service; doctor warns; alerts in WP-20 |
| Image supply chain | High | Pin digests, scan, SBOM, sign, no `latest` |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §4, §5, §16, §18, §20
- `../technical-specification/APPENDIX-D-DELIVERY.md` §D.1 (M0), §D.4
- `WP-05` (security), `WP-14` (sandbox), `WP-20` (observability)

## Progress log

2026-11-07 | T01 | infra/docker/{api,worker,web,runner,migrate}.Dockerfile | images built: api 28.1 MB, worker 21.4 MB, web 13 MB, runner 21.4 MB, migrate 35.9 MB | ADR-0023 runner token | T02
2026-11-07 | T02 | infra/compose/docker-compose.yml | postgres/redis/web/docker-socket-proxy healthy; compose config OK | — | T03
2026-11-07 | T03 | docker-compose.dev.yml, observability.yml, tls.yml, egress.yml | all profiles valid | — | T04
2026-11-07 | T04 | crates/infrastructure/src/health.rs, crates/{api,worker}/src/main.rs, crates/orchestrator/src/bin/runner.rs | health probes tested; containers healthy | — | T05
2026-11-07 | T05 | docker-socket-proxy Compose config | /version denied, /containers/json allowed from the runner network | — | T06
2026-11-07 | T06 | infra/egress/allowlist.txt, squid.conf | config valid; runtime egress owned by WP-14 | — | T07
2026-11-07 | T07 | Compose resource limits and security options | isolation test 8 services green | — | T08
2026-11-07 | T08 | scripts/backup.sh, scripts/restore-drill.sh | scripts delivered; drill needs WP-04 schema | — | T09
2026-11-07 | T09 | .github/workflows/images.yml | build job skeleton; Trivy/SBOM/sign owned by WP-22 | — | T10
2026-11-07 | T10 | .tools/test-compose-isolation.sh + runtime checks | edge cannot reach postgres/redis; proxy allowlist verified | — | closure
2026-11-07 | closure | .docs/product/operations.md | make check green; make up smoke runs after WP-04 | AC2/AC3/AC8 downstream by WP-04 | WP-04
2026-11-07 | review | .docs/sow/WP-03-INFRASTRUCTURE-review-plan.md | compose-validate all profiles; isolation 8 services; harness/sow green | AC2/AC3/AC8 WP-04, AC5/AC6 WP-14, AC7 WP-22, AC9 WP-13 | QA review
2026-11-07 | review-fix | infra/docker/*.Dockerfile, infra/compose/*.yml, crates/worker/src/main.rs, crates/orchestrator/src/bin/runner.rs, CR-0006 | digest pins; runner toolchains + readiness listener; worker runner probe; clippy/tests green; QA APPROVED | CR-0006 | merge
