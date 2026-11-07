---
name: Rust TDD
description: Use for any Rust backend change in domain, application, adapters, worker, API or CLI, following Red-Green-Refactor with deterministic tests.
---

# Rust TDD

## When to use

- The task changes a Rust crate under `crates/`.
- A bug needs a failing regression test.
- A new domain invariant, use case, repository or adapter is required.

Do not use this skill for frontend-only or documentation-only tasks.

## Preconditions

- Rust toolchain `1.99.1` is active (`rustup show`).
- The workspace builds: `cargo check --workspace`.
- The target crate and its package name are known. Package names follow `ailab-<crate>` for first-party crates.
- Database tests have a migrated template database available through `DATABASE_URL` or testcontainers.

## Workflow

1. Read the task's acceptance criteria and the relevant spec section.
2. Write a failing test in the crate's `tests/` or a `#[cfg(test)]` module. Use `proptest` for invariants and `insta` for event/prompt/DTO snapshots.
3. Run the focused test and confirm the expected failure:
   - `cargo nextest run -p ailab-domain <test_name>`
   - or `cargo test -p ailab-domain <test_name>`
4. Implement the minimum production code.
5. Run the focused test, then the crate suite:
   - `cargo nextest run -p ailab-domain`
6. Refactor with tests green.
7. Run the full relevant suite and gates:
   - `cargo fmt --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo nextest run --workspace`
   - `cargo llvm-cov --workspace --summary-only` when coverage is affected
8. For repository tests, use `sqlx::test` or testcontainers with the PostgreSQL 18 + pgvector image. Never mock the database for constraint or concurrency tests.
9. For compile-time SQL, run `cargo sqlx prepare --workspace` and commit `.sqlx` changes when queries change.
10. Record exact commands and results in the WP progress line.

## Checklist

- [ ] Failing test observed before production code.
- [ ] Test expresses behavior, not implementation detail.
- [ ] Production change is the minimum required.
- [ ] Refactor completed with tests green.
- [ ] Unit, integration and repository tests pass.
- [ ] `cargo fmt --check` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] No `unwrap` or `expect` in production paths.
- [ ] No `unsafe` in first-party crates.
- [ ] Database constraints and concurrency covered where relevant.
- [ ] Coverage thresholds met for changed code.
- [ ] `.sqlx` metadata updated when queries change.
- [ ] Evidence recorded.

## Anti-patterns

- Testing private helpers instead of observable behavior.
- Using `sleep` to wait for async work; use deterministic channels or polling with a bound.
- Mocking the database for constraint tests.
- Writing `unwrap` in request paths.
- Updating `.sqlx` without changing the query, or changing a query without updating `.sqlx`.
- Weakening or deleting a test to make CI pass.

## References

- `../../../.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §13, §19
- `../../../.docs/sow/WP-06-DOMAIN-KERNEL.md`
- `../../../.docs/sow/WP-04-DATA-PLATFORM.md`
- `../../../.docs/sow/WP-21-TESTING.md`
- `../../../.docs/PRINCIPLES.md` §1

## Supporting files

- None.
