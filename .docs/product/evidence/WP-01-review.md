# WP-01 Independent Architecture Review

| Field | Value |
|---|---|
| Reviewer | review-agent (independent subagent) |
| Author | Tech Lead (agent) |
| Date | 2026-11-07 |
| Scope | C4 model, bounded contexts, service catalog, contracts, data ownership, failure modes, fitness test, ADRs |
| Verdict | `APPROVED` |

The reviewer is a different actor from the author. The review was iterative: the first pass returned `CHANGES_REQUESTED` for three blocking defects; the author fixed them and the final pass approved.

## Final acceptance results

| AC | Result | Evidence |
|---|---|---|
| AC1 one owner, aggregates, tables and events per context | PASS | 32/32 tables mapped once in `.tools/architecture/contexts.tsv`; `.docs/product/domain-model.md`, `.docs/product/data-ownership.md`, WP-01 §3.5/§3.6 and `.docs/DATA-MODEL.md` agree |
| AC2 fitness test rejects cross-context table access | PASS | `.tools/check-architecture-fitness.sh`; self-test 13/13 including schema-qualified, quoted, escaped, multi-line, lowercase and unattributed SQL; wired in `docs-build` and `rust-integration` |
| AC3 service catalog covers Compose containers | PASS | All 8 base services plus the `tls`, `observability` and `egress` profile services appear in `.docs/product/service-catalog.md`; `.tools/check-architecture-fitness.sh` now scans `infra/compose/docker-compose*.yml` and fails on an uncatalogued service (self-test 15/15 includes the `infra/compose` cases). |
| AC4 runner API contract implemented/consumed without drift | DEFERRED | Contract delivered and recorded as ADR-0023; WP-13/WP-14 implement and consume it with a compatibility test |
| AC5 every ADR exists, follows the template and is linked | PASS | 24 ADRs registered; ADR-0001..0024 all include the `## Status` section required by `.docs/product/templates/adr.md`; all resolve from `.docs/product/architecture.md` §8 |
| AC6 failure matrix exercised by chaos tests | DEFERRED | Matrix delivered in `.docs/product/failure-modes.md`; WP-21 owns the chaos suite |
| AC7 diagrams build from source and are fresh | PASS | 4 `.mmd` sources, 4 fresher `.svg` exports, all embedded |

## Blocking defects found and resolved

1. `workspaces` ownership contradicted the Lab context. Resolved: Platform/Tenancy owns `workspaces` everywhere.
2. `github_webhook_deliveries` owner was ambiguous. Resolved: the SCM integration adapter context owns it, with aggregate `WebhookDelivery`.
3. The fitness test had false negatives for schema-qualified, quoted and multi-line SQL and did not gate CI. Resolved: robust extraction, `paths.tsv`, unattributed-SQL failure, CI wiring and 13 self-test cases.

## Review round 2 (WP review, 2026-11-07)

The independent reviewer re-ran the review on `review/wp-01-architecture` and returned `CHANGES_REQUESTED` for four blocking defects; all were fixed and re-reviewed:

1. Compose coverage was a false negative (`check-architecture-fitness.sh` only probed the repository root; the real file is `infra/compose/docker-compose.yml`). Resolved: the check now scans all `infra/compose/docker-compose*.yml` files and fails on an uncatalogued service; two self-test cases added (15/15).
2. ADR-0001..ADR-0022 omitted the `## Status` section required by the ADR template. Resolved: every ADR now has the section.
3. AC3 status was contradictory across the WP, review plan and evidence. Resolved: AC3 is now `PASS` everywhere with the compose evidence.
4. The review plan pre-asserted the QA approval. Resolved: the QA verdict is recorded after the reviewer run; this record carries the final result.

The `caddy` (TLS profile) service was added to the catalog to cover every Compose file.

## Reviewer statement

The WP-01 artifacts satisfy the architecture contract. AC3 is verified by the compose coverage check; AC4 and AC6 are downstream verifications owned by WP-13/WP-14 and WP-21. Approved for WP closure.
