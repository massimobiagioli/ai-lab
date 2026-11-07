#!/usr/bin/env bash
# Create the least-privilege database roles on first cluster initialisation.
# Runs once via /docker-entrypoint-initdb.d before any migration.
set -euo pipefail

app_pw="${AILAB_APP_DB_PASSWORD:-ailab_app}"
migrate_pw="${AILAB_MIGRATE_DB_PASSWORD:-ailab_migrate}"
readonly_pw="${AILAB_READONLY_DB_PASSWORD:-ailab_readonly}"

psql -v ON_ERROR_STOP=1 \
  --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" \
  -v db="$POSTGRES_DB" \
  -v app_pw="$app_pw" -v migrate_pw="$migrate_pw" -v readonly_pw="$readonly_pw" <<'SQL'
SELECT format('CREATE ROLE ailab_migrate LOGIN PASSWORD %L', :'migrate_pw')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ailab_migrate')
\gexec
SELECT format('CREATE ROLE ailab_app LOGIN PASSWORD %L', :'app_pw')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ailab_app')
\gexec
SELECT format('CREATE ROLE ailab_readonly LOGIN PASSWORD %L', :'readonly_pw')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ailab_readonly')
\gexec

GRANT CONNECT ON DATABASE :"db" TO ailab_migrate, ailab_app, ailab_readonly;
GRANT USAGE ON SCHEMA public TO ailab_migrate, ailab_app, ailab_readonly;
GRANT CREATE, USAGE ON SCHEMA public TO ailab_migrate;

ALTER DEFAULT PRIVILEGES FOR ROLE ailab_migrate IN SCHEMA public
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO ailab_app;
ALTER DEFAULT PRIVILEGES FOR ROLE ailab_migrate IN SCHEMA public
  GRANT SELECT ON TABLES TO ailab_readonly;
ALTER DEFAULT PRIVILEGES FOR ROLE ailab_migrate IN SCHEMA public
  GRANT USAGE, SELECT ON SEQUENCES TO ailab_app, ailab_readonly;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO ailab_app, ailab_readonly;
SQL
