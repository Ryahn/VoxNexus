# VoxNexus
[![Build](https://github.com/Ryahn/VoxNexus/actions/workflows/ci.yml/badge.svg)](https://github.com/Ryahn/VoxNexus/actions/workflows/ci.yml)

Self-hostable community chat: Discord-class chat and voice, Guilded-class Spaces, and a first-class app/workflow platform. **Source-available** for inspection, contribution, and one private personal instance — not OSI open source, and not free for commercial use, public hosting, or reuse in other projects.

**AI-assisted build.** This project was developed with substantial help from AI coding tools: [Cursor](https://cursor.com/), Claude, and Grok. Humans own the product direction, review, and responsibility for what ships.

## License

[VoxNexus Source-Available Personal Use License v1](LICENSE). Study the code and run one private personal instance. Commercial use, public instances, reuse in other projects, and redistribution are not allowed without separate written permission (contributions go through the [CLA](CLA.md)).

Plain-language FAQ: [LICENSE.md](LICENSE.md). Short form: [NOTICE](NOTICE).

## What works today

Rust binary `voxnexus` plus a Vite/React SPA:

| Surface | Behavior |
|---|---|
| `GET /health` | Liveness |
| `GET /ready` | Postgres + Redis + SeaweedFS + Typesense |
| `GET /api/v1/meta` | Instance name + version |
| `POST /api/v1/auth/register` · `login` · `logout` · `GET …/me` | Local email/password sessions (cookie) |
| `POST …/auth/me/password` · `PATCH …/auth/me/email` | Change password (re-auth) and email (immediate until SMTP) |
| `GET/PATCH /api/v1/me/profile` · avatar/banner upload | Own profile; images stored in SeaweedFS |
| `GET /api/v1/profiles/{id}` · `/avatar` · `/banner` | Read profiles/images (session required) |
| `GET /api/v1/gateway` | WebSocket gateway; session cookie required; IDENTIFY → READY (resume token) |
| `GET /metrics` | Prometheus scrape when `METRICS_ENABLED=true` |
| SPA | VOX UI shell (communities/chat chrome on mock data) + session auth + live profile settings. Served by Axum when `WEB_DIST` is set (Compose), or via Vite in dev |
| Object storage | S3 client to SeaweedFS; bucket created on startup if missing |
| Jobs | Apalis workers on Redis; sample `HealthPing` job |
| Search | Typesense client; `messages` / `users` / `channels` collections ensured |

Docker Compose stack: [`deploy/docker`](deploy/docker). Config keys and options: [`config.README.md`](config.README.md).

## Stack

- **Server:** Rust (stable via `rust-toolchain.toml`), Axum, Tokio, SQLx + PostgreSQL 16
- **Deps:** Redis (jobs), SeaweedFS S3 (media), Typesense (search)
- **Web:** React 19, Vite, TypeScript, Tailwind, Zustand, pnpm workspace — VOX UI shell in `apps/web`
- **Contracts:** OpenAPI → `@voxnexus/api-client`; gateway JSON Schema → `@voxnexus/protocol`

## Layout

```text
apps/web             Vite + React SPA (VOX UI shell + auth)
crates/server        voxnexus binary (HTTP + gateway + workers)
crates/config        file + env configuration
crates/db            PostgreSQL pool and migrations
crates/auth          sessions + password auth
crates/protocol      shared HTTP/gateway types (Rust)
crates/realtime      WebSocket session loop
crates/storage       S3 / SeaweedFS object store
crates/jobs          Apalis workers + Redis queue
crates/search        Typesense client
crates/*             other domain crates (permissions, media, …)
packages/api-client  generated OpenAPI TypeScript client
packages/protocol    generated gateway types + WS client
packages/ui          shared presentational primitives (extract from shell over time)
tools/codegen        export OpenAPI + gateway JSON Schema
migrations/          SQLx migration files
deploy/docker/       Compose stack + app Dockerfile
```

## Prerequisites

- [Rust](https://rustup.rs/) (stable) with rustfmt and clippy
- [pnpm](https://pnpm.io/) 10
- [Docker](https://docs.docker.com/get-docker/) (Compose) **or** PostgreSQL 16 + Redis + SeaweedFS/S3 + Typesense reachable as in `config.toml`

## Configure

There are **two** config places. Both are hand-edited; keep overlapping values in sync yourself.

1. **App config** — copy [`config.example.toml`](config.example.toml) → `config.toml` (Unix: `chmod 600`) for `cargo run` without Compose. Uses `127.0.0.1` host URLs for Postgres, Redis, SeaweedFS, and Typesense (the ports published by Compose, or your own local services).

2. **Compose env** — copy [`deploy/docker/.env.example`](deploy/docker/.env.example) → `deploy/docker/.env` for `docker compose`. Sets Postgres container credentials plus app env vars injected by [`docker-compose.yml`](deploy/docker/docker-compose.yml). Connection URLs inside the stack use Docker service names (`postgres`, `redis`, `seaweedfs`, `typesense`), not `127.0.0.1`.

Shared keys (`PUBLIC_URL`, `S3_BUCKET`, `S3_ACCESS_KEY`, `S3_SECRET_KEY`, `TYPESENSE_API_KEY`, `COOKIE_SECURE`, `LOG_LEVEL`, `GATEWAY_ALLOW_UNAUTH`, `METRICS_ENABLED`) should match when you run the app against the same stack. [`deploy/docker/seaweedfs-s3.json`](deploy/docker/seaweedfs-s3.json) must use the same S3 credentials as both files.

Env vars override `config.toml`. Missing/invalid required keys fail startup with the key name.

Full key-by-key reference (defaults and allowed values such as `LOG_FORMAT = auto|pretty|json`): **[`config.README.md`](config.README.md)**.

## Docker Compose

```powershell
cd deploy/docker
copy .env.example .env
docker compose -f docker-compose.yml up -d --build
curl.exe http://127.0.0.1:8080/health
curl.exe http://127.0.0.1:8080/ready
curl.exe http://127.0.0.1:8080/api/v1/meta
```

Brings up Postgres, Redis, SeaweedFS, Typesense, and the app (API + built SPA on `:8080`).

## Build and check

```powershell
pnpm install
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm check-codegen
pnpm build
pnpm lint
```

Live integration tests need env such as `DATABASE_URL_TEST` / `REDIS_URL_TEST` (and optional `S3_*_TEST` / `TYPESENSE_*_TEST`). CI runs the same gate on every PR (`.github/workflows/ci.yml`).

## Run (without Compose)

Point `config.toml` at local Postgres/Redis/S3/Typesense, then:

```powershell
cargo run -p voxnexus
```

```powershell
curl.exe http://127.0.0.1:8080/health
curl.exe http://127.0.0.1:8080/ready
curl.exe http://127.0.0.1:8080/api/v1/meta
```

SPA with Vite proxy to `:8080`:

```powershell
pnpm dev
```

Gateway debug UI (also set `GATEWAY_ALLOW_UNAUTH=true` on the server):

```powershell
$env:VITE_GATEWAY_DEBUG="true"
pnpm dev
```

After HTTP or gateway type changes: `pnpm codegen`. Drift check: `pnpm check-codegen`.
