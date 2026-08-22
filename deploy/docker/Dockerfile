# syntax=docker/dockerfile:1

# Web SPA (Vite) -------------------------------------------------------------
FROM node:22-bookworm AS web
WORKDIR /src
RUN corepack enable && corepack prepare pnpm@10.14.0 --activate
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY apps/web/package.json apps/web/
COPY packages/api-client/package.json packages/api-client/
COPY packages/protocol/package.json packages/protocol/
COPY packages/ui/package.json packages/ui/
RUN pnpm install --frozen-lockfile
COPY packages packages
COPY apps/web apps/web
RUN pnpm --filter @voxnexus/web build

# Rust binary ----------------------------------------------------------------
FROM rust:bookworm AS rust
WORKDIR /src
COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY crates crates
COPY migrations migrations
COPY tools tools
RUN cargo build --release -p voxnexus

# Runtime --------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=rust /src/target/release/voxnexus /usr/local/bin/voxnexus
COPY --from=web /src/apps/web/dist /app/web
ENV WEB_DIST=/app/web \
    LISTEN_ADDR=0.0.0.0:8080 \
    RUST_LOG=info
EXPOSE 8080
USER nobody
HEALTHCHECK --interval=10s --timeout=3s --start-period=20s --retries=10 \
    CMD curl -fsS http://127.0.0.1:8080/health || exit 1
CMD ["voxnexus"]
