# Migration failure

- **Detect:** the migration job fails, the schema version mismatch, readiness `NOT_READY`.
- **Check:** `_sqlx_migrations`, the failing SQL, the backup and the application version.
- **Mitigate:** stop the rollout; restore the pre-migration backup only if the migration is not backward-compatible; fix forward with a new migration and a CR.
- **Escalate:** Tech Lead; never edit an applied migration.
