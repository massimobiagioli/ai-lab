# Database restore

- **Detect:** readiness `NOT_READY`, integrity mismatch alert, corrupt relation, failed migration.
- **Check:** `scripts/integrity-check.sh`, the latest backup manifest, the migration version.
- **Mitigate:** `make backup` first if the DB is readable; `make restore-drill BACKUP=<dump>` to validate; restore into the primary only with the Tech Lead's approval; re-run migrations and the integrity job.
- **Escalate:** Tech Lead; declare an incident if the RPO is at risk.
