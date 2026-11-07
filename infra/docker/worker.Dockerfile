# syntax=docker/dockerfile:1.7
FROM rust:1.99.1-trixie@sha256:8a4e90c7946b74626488b1446f712c0fe0ed1a596cbd3d871684bc9c5433ae07 AS chef
RUN cargo install cargo-chef --locked
WORKDIR /src

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /src/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p ailab-worker
COPY . .
RUN cargo build --release -p ailab-worker

FROM debian:trixie-slim@sha256:a72b4433de1cf7d6bae0c4cee74d0547f58601a643b3375f070c51600d63b6b2
LABEL org.opencontainers.image.title="ailab-worker" \
      org.opencontainers.image.description="AI-Lab worker service" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.source="https://github.com/example/ai-lab"
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates git tzdata \
 && rm -rf /var/lib/apt/lists/* /usr/lib/apt /usr/bin/apt* /usr/bin/dpkg* /usr/sbin/dpkg* \
 && groupadd -g 10001 ailab \
 && useradd -u 10001 -g 10001 -m -s /usr/sbin/nologin ailab
COPY --from=builder /src/target/release/ailab-worker /usr/local/bin/ailab-worker
USER 10001:10001
STOPSIGNAL SIGTERM
ENTRYPOINT ["/usr/local/bin/ailab-worker"]
