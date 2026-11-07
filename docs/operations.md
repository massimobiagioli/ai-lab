# AI-Lab Operations

| Field | Value |
|---|---|
| Document | `docs/operations.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `docs/service-catalog.md`, `docs/failure-modes.md`, `infra/README.md`, `.docs/sow/WP-03-INFRASTRUCTURE.md` |

This document covers install, run, upgrade, backup, restore, scale and troubleshooting for the Docker Compose runtime. It is the operator-facing companion to the Compose files in `infra/compose/`.

---

## 1. Prerequisites

- Docker Engine ≥ 27 and Docker Compose ≥ 2.30. Older versions work for development but emit a warning in `make doctor`.
- ≥ 4 vCPU, ≥ 8 GB RAM, ≥ 20 GB free disk.
- Rust 1.99.1 for local builds (`rust-toolchain.toml`), Node 24/npm 11 for the frontend, or the pinned container fallback used by `scripts/run-web.sh`.
- Outbound HTTPS to DeepSeek, GitHub and Docker Hub.

## 2. Install

```bash
git clone <repository> ai-lab
cd ai-lab
make setup
cp .env.example .env
scripts/doctor.sh --generate-key
make doctor
make up
```

`make up` builds the five images and starts the `core` profile: `postgres`, `redis`, `migrate`, `api`, `worker`, `runner`, `docker-socket-proxy`, `web`. The web UI is at `http://127.0.0.1:8080`.

`.env` is gitignored. `--generate-key` writes a development `AILAB_MASTER_KEY`; production requires an operator-supplied key.

## 3. Profiles

| Profile | Command | Services |
|---|---|---|
| core (default) | `make up` | postgres, redis, migrate, api, worker, runner, docker-socket-proxy, web |
| dev | `make up-dev` | core + development ports (3000, 5432, 6379) and source mounts |
| observability | `make up-obs` | core + otel-collector, prometheus, grafana |
| tls | `make up-tls` | core + Caddy on 80/443 |
| egress | `make up-egress` | core + egress-proxy for sandbox dependency installs |

Only `web` publishes a host port in the core profile. `postgres` and `redis` are never published in production. Dev ports bind to `127.0.0.1`.

## 4. Health, logs and status

```bash
make ps            # container status and health
make logs          # follow core logs
curl -fsS http://127.0.0.1:8080/healthz
docker compose -f infra/compose/docker-compose.yml --profile core exec postgres pg_isready -U ailab
```

Readiness semantics: `api` checks Postgres and Redis; `worker` checks Postgres, Redis and a runner; `runner` checks its token and the Docker socket proxy; `web` checks nginx.

## 5. Upgrade

1. Read the changelog and the release notes.
2. Back up first: `make backup`.
3. Pull or build the new images.
4. Apply migrations: `make migrate`.
5. Recreate the services: `make up`.
6. Verify `make ps` and the golden path.

Migrations follow expand/contract: add the new shape, deploy, backfill, then remove the old shape in a later release. A failed migration blocks `api` and `worker` by design; read the migrate logs before retrying.

## 6. Backup and restore

```bash
make backup
# artifacts/backups/<UTC timestamp>.dump and <UTC timestamp>.manifest.json
make restore-drill BACKUP=artifacts/backups/<UTC timestamp>.dump
```

The manifest records the dump name, SHA-256, git revision and the latest migration version. The drill restores into `ailab_restore_drill` and compares row counts for the core tables.

- RPO ≤ 24 h with daily backups; ≤ 5 min with WAL archiving.
- RTO ≤ 1 h.
- Retention: 14 daily backups locally; off-host copy is an operator script.
- Run a restore drill monthly and before every release.

## 7. Scaling

- `api` and `web` are stateless; scale horizontally behind the edge proxy.
- `worker` scales with leases (`FOR UPDATE SKIP LOCKED`); set `AILAB_WORKER_CONCURRENCY` and add replicas.
- `runner` is node-local; one runner per Docker host.
- `postgres` is a single primary in v1; a read replica is optional and read-only.
- `redis` is a single instance in v1; it is cache/ephemeral, never authoritative.
- Default limits: api 1 CPU/512 MB, worker 2 CPU/1 GB, runner 1 CPU/512 MB, web 0.5 CPU/128 MB, postgres 2 CPU/2 GB, redis 1 CPU/768 MB.

## 8. Security defaults

- No `privileged` containers, no host networking, no host mounts except the Docker socket in `docker-socket-proxy`.
- The socket proxy allowlists only container/image/exec/log/info APIs.
- `api`, `worker`, `runner`, `web` and the socket proxy run read-only with `cap_drop: ALL` and `no-new-privileges`.
- Postgres is the only service with `cap_add` (`CHOWN`, `DAC_OVERRIDE`, `FOWNER`, `SETGID`, `SETUID`) because its entrypoint must own the data directory.
- Sandbox networks (`runner`, `sandbox-control`) are internal; sandbox egress only through `egress-proxy`.
- Secrets are never in Compose files or images; they come from `.env` or the operator secret store.

## 9. Image supply chain

Images are built by `infra/docker/*.Dockerfile`. `.github/workflows/images.yml` builds api/worker/web/runner, runs Trivy, produces SBOMs and signs with cosign on a release tag (WP-22 completes the signing and publication). Image budgets: api ≤120 MB, worker ≤140 MB, web ≤40 MB, runner ≤600 MB, migrate ≤80 MB.

## 10. Troubleshooting

| Symptom | Action |
|---|---|
| `migrate` exits non-zero | Read `make logs`; fix the migration; `make up` blocks api/worker by design |
| Postgres restart loop | Check the volume ownership and `PGDATA`; never downgrade the image |
| Port 8080 busy | `make doctor` reports the port; stop the conflicting process or set `AILAB_PORT` |
| Docker socket errors | Verify `docker-socket-proxy` health and the allowlist |
| Web 502 on `/api` | The api process or its health is down; check `make ps` and api logs |
| Disk full | `make clean` for build artifacts; prune worktrees; expand the volume |
| Budget exhausted | The run stops safely; raise the budget or pause the lab |

See `docs/failure-modes.md` and `docs/runbooks/` for recovery procedures.

## 11. References

- `.docs/sow/WP-03-INFRASTRUCTURE.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §18, §20
- `infra/compose/`, `infra/docker/`, `infra/nginx/`, `infra/caddy/`, `infra/egress/`
- `docs/service-catalog.md`, `docs/failure-modes.md`
