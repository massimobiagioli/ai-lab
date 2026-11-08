# Change Request Register

This directory is the canonical record of AI-Lab change requests. Each CR is a separate file named `CR-NNNN.md`, allocated sequentially and never reused. The format is defined in `../change-control.md` and `../templates/change-request.md`.

| CR | Title | Decision | Approver | Date | Linked ADR |
|---|---|---|---|---|---|
| [CR-0001](./CR-0001.md) | Adopt `.docs/product/` for product documentation and keep `.docs/` for the harness | approved | Tech Lead | 2026-11-07 | — |
| [CR-0002](./CR-0002.md) | Close WP-00 with environmental and consumer responsibilities assigned | approved | Human Operator + Tech Lead | 2026-11-07 | — |
| [CR-0003](./CR-0003.md) | Enable the review merge workflow under branch protection | approved | Tech Lead | 2026-11-07 | — |
| [CR-0004](./CR-0004.md) | Add AILAB_RUNNER_TOKEN to the environment variable contract | approved | Tech Lead | 2026-11-07 | ADR-0023 |
| [CR-0005](./CR-0005.md) | Set the TypeScript pin to 6.0.x in specification §5.1 | approved | Tech Lead | 2026-11-07 | ADR-0024 |
| [CR-0006](./CR-0006.md) | Add AILAB_RUNNER_URL and AILAB_RUNNER_BIND to Appendix D.4 | approved | Tech Lead | 2026-11-07 | ADR-0023 |
| [CR-0007](./CR-0007.md) | Add the least-privilege database role passwords to Appendix D.4 | approved | Tech Lead | 2026-11-07 | ADR-0003 |
| [CR-0008](./CR-0008.md) | Reopen WP-13 to deliver the missing orchestration pipeline | approved | Tech Lead | 2026-11-07 | — |
| [CR-0009](./CR-0009.md) | Re-baseline WP-13/WP-14 and schedule the platform build phase | approved | Tech Lead | 2026-11-07 | — |
| [CR-0010](./CR-0010.md) | Reopen WP-16 for the edge server build | approved | Tech Lead | 2026-11-07 | ADR-0014 |

Rules:

1. A file in this directory appears exactly once in the table.
2. Every row points to an existing file.
3. A rejected or deferred CR stays in the register.
4. A CR that changes an accepted ADR links the superseding ADR.
5. A CR never removes history; it supersedes.
