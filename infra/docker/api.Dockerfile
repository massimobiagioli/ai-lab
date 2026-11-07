# syntax=docker/dockerfile:1.7
FROM rust:1.99.1-trixie AS chef
RUN cargo install cargo-chef --locked
WORKDIR /src

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /src/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p ailab-api
COPY . .
RUN cargo build --release -p ailab-api

FROM gcr.io/distroless/cc-debian13
LABEL org.opencontainers.image.title="ailab-api" \
      org.opencontainers.image.description="AI-Lab API service" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.source="https://github.com/example/ai-lab"
COPY --from=builder /src/target/release/ailab-api /usr/local/bin/ailab-api
USER 10001:10001
EXPOSE 3000
STOPSIGNAL SIGTERM
ENTRYPOINT ["/usr/local/bin/ailab-api"]
