# syntax=docker/dockerfile:1.7
FROM rust:1.99.1-trixie AS builder
RUN cargo install sqlx-cli --version 0.9.0 --no-default-features --features postgres,rustls --locked

FROM gcr.io/distroless/cc-debian13
LABEL org.opencontainers.image.title="ailab-migrate" \
      org.opencontainers.image.description="AI-Lab database migration job" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.source="https://github.com/example/ai-lab"
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY migrations /migrations
STOPSIGNAL SIGTERM
ENTRYPOINT ["sqlx", "migrate", "run", "--source", "/migrations"]
