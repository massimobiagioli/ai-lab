# syntax=docker/dockerfile:1.7
FROM node:24.22.0-trixie-slim AS builder
WORKDIR /app
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web ./
RUN npm run build

FROM nginxinc/nginx-unprivileged:1.29-alpine-slim
LABEL org.opencontainers.image.title="ailab-web" \
      org.opencontainers.image.description="AI-Lab Angular web application" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.source="https://github.com/example/ai-lab"
COPY infra/nginx/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/dist/web/browser /usr/share/nginx/html
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --retries=6 --start-period=10s \
  CMD wget -qO- http://127.0.0.1:8080/healthz || exit 1
STOPSIGNAL SIGTERM
