# Infrastructure

Owned by WP-03 (Docker, Compose, networks, volumes, proxies) and WP-20 (observability).

| Directory | Purpose |
|---|---|
| `docker/` | Service Dockerfiles (api, worker, runner, web, migrate) |
| `compose/` | `docker-compose.yml` and the dev, observability, TLS and egress overlays |
| `nginx/` | Web reverse proxy configuration |
| `caddy/` | Optional TLS termination for the `tls` profile |
| `egress/` | Sandbox egress allowlist and Squid configuration |
| `prometheus/` | Scrape configuration and alert rules |
| `grafana/` | Provisioned dashboards and datasources |
| `otel/` | OpenTelemetry Collector configuration |

The Makefile uses `infra/compose/docker-compose.yml` and the profile overlays. `make compose-validate` renders every profile and `make up` starts the core stack.
