---
name: Data Migrations
description: Use when changing PostgreSQL schema, indexes, queries, pgvector data, Redis keys, retention, backups or integrity checks.
---

# Data Migrations

## When to use

- A table, column, constraint, index, trigger or extension changes.
- A query, repository or projection changes.
- A vector index, embedding dimension or retrieval SQL changes.
- A Redis key, TTL, lock or rate-limit bucket changes.
- Retention, compaction or integrity rules change.

## Preconditions

- Read `.docs/technical-specification/APPENDIX-A-DATABASE.sql`, `.docs/DATA-MODEL.md` and `.docs/sow/WP-04-DATA-PLATFORM.md`.
- Confirm the change preserves the ownership matrix; no context writes another context's table.
- Have a fresh database and a representative dataset for migration testing.

## Workflow

1. Write the migration as a new file `migrations/{timestamp}_{name}.sql`. Never edit an applied migration.
2. Choose the pattern:
   - additive: add nullable column or table, dual write, backfill in batches, switch reads, contract later;
   - index: use `CREATE INDEX CONCURRENTLY` on large tables;
   - destructive: only after an expand/contract cycle and a release boundary.
3. Write a failing test for the new behavior or constraint.
4. Implement the migration and the repository change.
5. Run:
   - `make migrate` on an empty database;
   - `make migrate` on a copy with representative data;
   - `cargo sqlx prepare --workspace` and commit `.sqlx`;
   - constraint, concurrency and migration tests.
6. Verify the schema matches the documented DDL with a schema diff.
7. Verify performance on the reference dataset for changed queries.
8. Update `.docs/technical-specification/APPENDIX-A-DATABASE.sql` only if the canonical schema is intentionally changed; otherwise keep code aligned with it.
9. Update `.docs/DATA-MODEL.md` in the same change: entity meaning, attributes, relationships, logical access paths, retention and cross-context interactions. The data model is logical and contains no SQL syntax.
10. Update the data dictionary and the integrity checks.
11. Record the migration, commands and results in the WP progress line.

## Checklist

- [ ] Migration is forward-only and timestamped.
- [ ] No long `ACCESS EXCLUSIVE` lock without `lock_timeout`.
- [ ] Large indexes use `CONCURRENTLY`.
- [ ] Backfill is batched and resumable.
- [ ] Foreign keys, checks and unique constraints preserved.
- [ ] Query uses bound parameters and an allowlisted sort or filter.
- [ ] `.sqlx` metadata updated.
- [ ] Schema diff clean on a fresh database.
- [ ] `.docs/DATA-MODEL.md` updated in the same change.
- [ ] Concurrency tests pass.
- [ ] Performance budget met.
- [ ] Retention and integrity jobs updated.
- [ ] Backup and restore still valid.

## Anti-patterns

- Editing an applied migration.
- Dropping a column in the same release that stops writing it.
- Using `SELECT *` in production code.
- Building dynamic SQL with string interpolation.
- Adding an index without a query that needs it.
- Changing the embedding dimension without a new table and backfill.
- Using Redis as a source of truth.
- Changing an entity, relationship or access path without updating `.docs/DATA-MODEL.md`.

## References

- `../../../.docs/DATA-MODEL.md`
- `../../../.docs/technical-specification/APPENDIX-A-DATABASE.sql`
- `../../../.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §14
- `../../../.docs/sow/WP-04-DATA-PLATFORM.md`
- `../../../.docs/sow/WP-08-LAB-MEMORY.md`

## Supporting files

- None.
