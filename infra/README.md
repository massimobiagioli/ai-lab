# Infrastructure

Owned by WP-03 (Docker, Compose, networks, volumes, proxies) and WP-20 (observability).

| Directory | Purpose |
|---|---|
| `docker/` | Service Dockerfiles (api, worker, runner, web) |
| `compose/` | `compose.yaml` and profile overlays |
| `nginx/` | Web reverse proxy configuration |
| `prometheus/` | Scrape configuration and alert rules |
| `grafana/` | Provisioned dashboards and datasources |
| `otel/` | OpenTelemetry Collector configuration |

The Makefile expects `infra/compose/compose.yaml`. Until WP-03 lands, Compose-dependent targets print a clear error and exit non-zero.
