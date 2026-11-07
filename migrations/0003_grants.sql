-- Least-privilege grants for the application and read-only roles.
-- Runs as ailab_migrate (DDL role) after the schema and analytics views exist.

GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO ailab_app;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO ailab_readonly;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO ailab_app, ailab_readonly;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO ailab_app, ailab_readonly;

ALTER DEFAULT PRIVILEGES FOR ROLE ailab_migrate IN SCHEMA public
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO ailab_app;
ALTER DEFAULT PRIVILEGES FOR ROLE ailab_migrate IN SCHEMA public
  GRANT SELECT ON TABLES TO ailab_readonly;
ALTER DEFAULT PRIVILEGES FOR ROLE ailab_migrate IN SCHEMA public
  GRANT USAGE, SELECT ON SEQUENCES TO ailab_app, ailab_readonly;
ALTER DEFAULT PRIVILEGES FOR ROLE ailab_migrate IN SCHEMA public
  GRANT EXECUTE ON FUNCTIONS TO ailab_app, ailab_readonly;

GRANT USAGE ON SCHEMA analytics TO ailab_app, ailab_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA analytics TO ailab_app, ailab_readonly;
ALTER DEFAULT PRIVILEGES FOR ROLE ailab_migrate IN SCHEMA analytics
  GRANT SELECT ON TABLES TO ailab_app, ailab_readonly;
