# syntax=docker/dockerfile:1.7
FROM rust:1.99.1-trixie@sha256:8a4e90c7946b74626488b1446f712c0fe0ed1a596cbd3d871684bc9c5433ae07 AS builder
RUN cargo install sqlx-cli --version 0.9.0 --no-default-features --features postgres,rustls --locked

FROM gcr.io/distroless/cc-debian13@sha256:58e12be899275afaa05f1b03051ab19037ed7f252528e57a3a9ad7e220a62578
LABEL org.opencontainers.image.title="ailab-migrate" \
      org.opencontainers.image.description="AI-Lab database migration job" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.source="https://github.com/example/ai-lab"
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY migrations /migrations
STOPSIGNAL SIGTERM
ENTRYPOINT ["sqlx", "migrate", "run", "--source", "/migrations"]
