# syntax=docker/dockerfile:1.7
FROM rust:1.99.1-alpine AS chef
RUN apk add --no-cache musl-dev gcc
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

FROM alpine:3.21
LABEL org.opencontainers.image.title="ailab-worker" \
      org.opencontainers.image.description="AI-Lab worker service" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.source="https://github.com/example/ai-lab"
RUN addgroup -g 10001 -S ailab \
 && adduser -u 10001 -S -G ailab -H -s /sbin/nologin ailab \
 && apk add --no-cache ca-certificates git tzdata
COPY --from=builder /src/target/release/ailab-worker /usr/local/bin/ailab-worker
USER 10001:10001
STOPSIGNAL SIGTERM
ENTRYPOINT ["/usr/local/bin/ailab-worker"]
