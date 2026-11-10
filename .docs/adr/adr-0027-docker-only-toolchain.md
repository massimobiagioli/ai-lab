# ADR-0027 — Docker-only development toolchain

| Field | Value |
|---|---|
| Status | Accepted |
| Date | 2026-11-10 |
| Deciders | Tech Lead, QA, Dev |
| Supersedes | — |
| Superseded by | — |
| Related | ADR-0020, ADR-0024 |

## Context

The repository is deployed with Docker Compose and pins Rust, Node, Python and Playwright images. Some local commands still used host toolchains (`cargo fetch`, host `npm ci`, host `cargo test`, host `openssl`), and `scripts/doctor.sh --generate-key` mutated `.env` with host OpenSSL. This drifts from the stack contract and produces "works on my machine" differences between local runs and CI.

## Decision

AI-Lab is **Docker-only for application toolchains**.

- The host requires Docker Engine 27+, Docker Compose 2.30+, Make, Git and basic shell utilities only.
- Rust commands run through `scripts/run-rust.sh` in the pinned Rust image.
- Frontend commands run through `scripts/run-web.sh` in the pinned Node image.
- Node scripts run through `scripts/run-node.sh`.
- Python scripts run through `scripts/run-python.sh`.
- Key generation is the dedicated `make generate-key` target, implemented in Docker Node; `scripts/doctor.sh` no longer generates keys and no longer requires host Rust, Node, Python or OpenSSL.
- Playwright and BDD run in the pinned Playwright image; load runs in the k6 image; Compose, backup and restore commands use Docker Compose.
- Wrappers fail when Docker is unavailable. They do not fall back to host toolchains.
- The rule is documented in `CLAUDE.md`, `AGENTS.md`, `STACK.md`, the product operations/testing docs and the playbooks.

## Consequences

- Local, CI and release commands use the same pinned images.
- No host Rust/Node/Python/OpenSSL installation is required or supported.
- First use pulls the images and the cargo/npm caches; `make setup` warms them.
- A wrapper that silently falls back to a host command is a review defect.
- The doctor checks Docker, Compose, images, ports, disk, `.env` keys and connectivity only.

## Alternatives considered

- **Host toolchains with a container fallback**: rejected because it produces two different environments and hides missing Docker prerequisites.
- **A single multi-tool image for Rust, Node, Python and Playwright**: rejected because it is large, slower to pull and harder to pin.
- **Keeping key generation in `doctor.sh`**: rejected because the doctor is read-only and key generation needs its own explicit, auditable target.

## Compliance

- `Makefile` targets use the wrappers.
- `scripts/run-{rust,web,node,python}.sh` are the only application-toolchain entry points.
- `make generate-key` is the only key generator.
- `scripts/doctor.sh` checks the Docker-only prerequisites.
- `scripts/harness-lint.sh` and the documentation index remain green.
