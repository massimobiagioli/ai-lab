# Disk full

- **Detect:** Postgres/runner disk alerts, failed writes, `NOT_READY`.
- **Check:** `df -h`, Postgres WAL/backups, the runner workspaces and logs.
- **Mitigate:** remove expired backups and artifacts, vacuum Postgres, prune Docker images, move backups off-host.
- **Escalate:** Tech Lead; declare an incident if the DB cannot write.
