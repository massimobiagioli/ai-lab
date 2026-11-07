# Changelog

All notable changes to AI-Lab are documented here. The format follows Keep a Changelog and the project uses Semantic Versioning.

## [Unreleased]

### Added

- WP-00: governance, quality gates, status model, metrics views, templates, change control, risk register, `sow-lint`, CI gate mapping.
- WP-01: C4 architecture, domain model, service catalog, contracts, data ownership, failure modes, architecture fitness checks, ADR-0023 runner API.
- WP-02: Rust workspace, Angular 22.2 workspace, pinned toolchains, Make interface, developer scripts, hooks, supply-chain configuration, contribution docs.
- WP-03: Docker/Compose platform, five images, health probes, networks, TLS/observability/egress profiles, backup/restore, operations docs.
- WP-04: PostgreSQL migrations, analytics views, seed demo, constraint/concurrency suites, integrity checks, database docs.
- WP-05: threat model, abuse cases, AES-256-GCM, exposure guard, redaction, security headers, ASVS/audit/incident docs.
- WP-06: domain kernel, UUIDv7 ids, value objects, errors, ports, outbox, event catalog, testkit.
- WP-07: inventory aggregates and invariants. WP-08: lab and scoped hybrid memory. WP-09: Kanban matrix and guards.
- WP-10: SOW lifecycle and acyclic plan materialization. WP-11: DeepSeek port, prompts, routing and budgets.
- WP-12: SCM port, GitHub adapter and webhooks. WP-13: scheduler eligibility, leases, retries and recovery.
- WP-14: sandbox spec, tool policy and egress guards. WP-15: QA verdicts and autonomy merge policy.
- WP-16: REST/WS contracts, rate limits, backfill, deterministic OpenAPI/AsyncAPI.
- WP-17: pixel tokens and status badge. WP-18: A* pathfinding and truthful office state. WP-19: dashboard flow summary.
- WP-20: telemetry metric contract, Prometheus alerts and observability docs. WP-21: testing strategy and quality gates.

### Changed

- TypeScript is pinned to 6.0.x for Angular 22.2 compatibility (ADR-0024).

### Security

- Secret scanning, dependency policy and container scanning are wired into CI.
