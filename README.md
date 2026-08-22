# VoxNexus

Self-hostable community chat: Discord-class chat and voice, Guilded-class Spaces, and a first-class app/workflow platform. Source-available for one private personal instance — not a hosted SaaS and not OSI open source.

**AI-assisted build.** This project was developed with substantial help from AI coding tools: [Cursor](https://cursor.com/), Claude, and Grok. Humans own the product direction, review, and responsibility for what ships.

## License

[VoxNexus Source-Available Personal Use License v1](LICENSE). Study the code and run one private personal instance. Commercial use, public instances, reuse in other projects, and redistribution are not allowed except by contributing under the [CLA](CLA.md).

Plain-language FAQ: [LICENSE.md](LICENSE.md). Short form: [NOTICE](NOTICE).

## What works today

Rust binary `voxnexus` plus a Vite/React SPA:

| Surface | Behavior |
|---|---|
| `GET /health` | Liveness |
| `GET /ready` | Postgres + Redis + SeaweedFS + Typesense |
| `GET /api/v1/meta` | Instance name + version |
| `POST /api/v1/auth/register` · `login` · `logout` · `GET …/me` | Local email/password sessions (cookie) |
| `GET /api/v1/gateway` | WebSocket gateway; session cookie required; IDENTIFY → READY (resume token) |
| `GET /metrics` | Prometheus scrape when `METRICS_ENABLED=true` |
| SPA | Register / login / logout + meta; optional gateway debug UI. Served by Axum when `WEB_DIST` is set (Compose), or via Vite in dev |
| Object storage | S3 client to SeaweedFS; bucket created on startup if missing |
| Jobs | Apalis workers on Redis; sample `HealthPing` job |
| Search | Typesense client; `messages` / `users` / `channels` collections ensured |

Docker Compose stack: [`deploy/docker`](deploy/docker). Config keys and options: [`config.README.md`](config.README.md).

## Stack

- **Server:** Rust (stable via `rust-toolchain.toml`), Axum, Tokio, SQLx + PostgreSQL 16
- **Deps:** Redis (jobs), SeaweedFS S3 (media), Typesense (search)
- **Web:** React 19, Vite, TypeScript, pnpm workspace
- **Contracts:** OpenAPI → `@voxnexus/api-client`; gateway JSON Schema → `@voxnexus/protocol`

## Layout

```text
apps/web             Vite + React SPA
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
packages/ui          shared presentational components
tools/codegen        export OpenAPI + gateway JSON Schema
migrations/          SQLx migration files
deploy/docker/       Compose stack + app Dockerfile
```

## Prerequisites

- [Rust](https://rustup.rs/) (stable) with rustfmt and clippy
- [pnpm](https://pnpm.io/) 10
- [Docker](https://docs.docker.com/get-docker/) (Compose) **or** PostgreSQL 16 + Redis + SeaweedFS/S3 + Typesense reachable as in `config.toml`

## Configure

Copy [`config.example.toml`](config.example.toml) → `config.toml` (Unix: `chmod 600`), or set the same keys as environment variables. Env overrides the file. Missing/invalid required keys fail startup with the key name.

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
