# syntax=docker/dockerfile:1.7
# Runner image and sandbox base. It carries the pinned build toolchains the
# sandbox needs (WP-14 image contract); application secrets are never baked in.
ARG RUST_IMAGE=rust:1.99.1-trixie@sha256:8a4e90c7946b74626488b1446f712c0fe0ed1a596cbd3d871684bc9c5433ae07
ARG NODE_IMAGE=node:24.22.0-trixie-slim@sha256:a6155384446670e9719bdc4fd0ebe4d4b29fb32a8d6aa523da3547a6cfbcf22c
ARG PYTHON_IMAGE=python:3.12-slim-bookworm@sha256:935c3c5b12357a7e10caa9c1e6ca5692344d97cf229832ef584be2cec9fc03df
ARG RUNTIME_IMAGE=debian:trixie-slim@sha256:a72b4433de1cf7d6bae0c4cee74d0547f58601a643b3375f070c51600d63b6b2

FROM ${RUST_IMAGE} AS chef
RUN cargo install cargo-chef --locked
WORKDIR /src

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /src/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p ailab-orchestrator --bin ailab-runner
COPY . .
RUN cargo build --release -p ailab-orchestrator --bin ailab-runner \
 && cargo install just --locked --root /tools

FROM ${NODE_IMAGE} AS node

FROM ${PYTHON_IMAGE} AS python

FROM ${RUNTIME_IMAGE}
LABEL org.opencontainers.image.title="ailab-runner" \
      org.opencontainers.image.description="AI-Lab runner and sandbox base" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.source="https://github.com/example/ai-lab"
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      ca-certificates git build-essential pkg-config make ripgrep jq patch diffutils coreutils \
 && rm -rf /var/lib/apt/lists/* \
 && groupadd -g 10001 ailab \
 && useradd -u 10001 -g 10001 -m -s /usr/sbin/nologin ailab

COPY --from=builder /usr/local/rustup /usr/local/rustup
COPY --from=builder /usr/local/cargo /usr/local/cargo
COPY --from=builder /tools/bin/just /usr/local/bin/just
COPY --from=node /usr/local/bin/node /usr/local/bin/node
COPY --from=node /usr/local/lib/node_modules /usr/local/lib/node_modules
RUN ln -sf /usr/local/lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
 && ln -sf /usr/local/lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx
COPY --from=python /usr/local /usr/local
COPY --from=builder /src/target/release/ailab-runner /usr/local/bin/ailab-runner

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:/usr/local/bin:/usr/local/sbin:/usr/sbin:/usr/bin:/sbin:/bin

RUN node --version && npm --version && python3 --version && rustc --version && just --version

USER 10001:10001
WORKDIR /workspace
EXPOSE 8081
STOPSIGNAL SIGTERM
ENTRYPOINT ["/usr/local/bin/ailab-runner"]
