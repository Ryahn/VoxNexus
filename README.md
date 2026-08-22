# VoxNexus

Self-hostable community chat: Discord-class chat and voice, Guilded-class Spaces, and a first-class app/workflow platform. Source-available for one private personal instance — not a hosted SaaS and not OSI open source.

## License

[VoxNexus Source-Available Personal Use License v1](LICENSE). Study the code and run one private personal instance. Commercial use, public instances, reuse in other projects, and redistribution are not allowed except by contributing under the [CLA](CLA.md).

Plain-language FAQ: [LICENSE.md](LICENSE.md). Short form: [NOTICE](NOTICE).

## What works today

Rust binary `voxnexus` plus a Vite/React SPA:

| Surface | Behavior |
|---|---|
| `GET /health` | Liveness |
| `GET /ready` | Postgres ping + SeaweedFS `HeadBucket` (Redis/Typesense still skipped) |
| `GET /api/v1/meta` | Instance name + version |
| `GET /api/v1/gateway` | WebSocket gateway (Hello / heartbeat / DevPing); gated by `GATEWAY_ALLOW_UNAUTH` |
| `GET /metrics` | Prometheus scrape when `METRICS_ENABLED=true` |
| Web app | Loads meta via the generated API client; optional gateway debug UI |
| Object storage | S3 client to SeaweedFS; bucket created on startup if missing |

Redis and Typesense keys are required at startup but not connected yet. There is no Docker Compose stack yet (F009).

Operator docs: [config](docs/config.md), [database](docs/database.md), [storage](docs/storage.md), [API](docs/api.md), [gateway](docs/gateway.md), [observability](docs/observability.md), [codegen](docs/codegen.md).

## Stack

- **Server:** Rust (stable via `rust-toolchain.toml`), Axum, Tokio, SQLx + PostgreSQL 16
- **Web:** React 19, Vite, TypeScript, pnpm workspace
- **Contracts:** OpenAPI → `@voxnexus/api-client`; gateway JSON Schema → `@voxnexus/protocol`

## Layout

```text
apps/web             Vite + React SPA
crates/server        voxnexus binary (HTTP + gateway composition root)
crates/config        file + env configuration
crates/db            PostgreSQL pool and migrations
crates/protocol      shared HTTP/gateway types (Rust)
crates/realtime      WebSocket session loop
crates/storage       S3 / SeaweedFS object store
crates/*             domain crates (auth, permissions, media, …)
packages/api-client  generated OpenAPI TypeScript client
packages/protocol    generated gateway types + WS client
packages/ui          shared presentational components
tools/codegen        export OpenAPI + gateway JSON Schema
migrations/          SQLx migration files
docs/                operator and planning docs
```

## Prerequisites

- [Rust](https://rustup.rs/) (stable) with rustfmt and clippy
- [pnpm](https://pnpm.io/) 10
- PostgreSQL 16 reachable at `DATABASE_URL`
- S3-compatible endpoint at `S3_ENDPOINT` (SeaweedFS or LocalStack)

## Configure

Copy [`config.example.toml`](config.example.toml) to `config.toml` (Unix: `chmod 600`) or set the same keys as environment variables. Env overrides the file. Missing or invalid required keys fail startup with the key name.

Defaults: listen `127.0.0.1:8080`, CORS origin from `PUBLIC_URL`. Useful toggles: `GATEWAY_ALLOW_UNAUTH`, `METRICS_ENABLED`, `LOG_LEVEL`, `LOG_FORMAT`.

## Build and check

```powershell
pnpm install
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm check-codegen
pnpm build
```

Database integration tests need `DATABASE_URL_TEST` ([docs/database.md](docs/database.md)). Live S3 tests need `S3_ENDPOINT_TEST` ([docs/storage.md](docs/storage.md)).

## Run

Terminal 1 — API (migrations run on startup):

```powershell
cargo run -p voxnexus
```

```powershell
curl.exe http://127.0.0.1:8080/health
curl.exe http://127.0.0.1:8080/ready
curl.exe http://127.0.0.1:8080/api/v1/meta
```

Terminal 2 — SPA (Vite proxies `/api` and WebSockets to `:8080`):

```powershell
pnpm dev
```

Gateway debug UI (also set `GATEWAY_ALLOW_UNAUTH=true` on the server):

```powershell
$env:VITE_GATEWAY_DEBUG="true"
pnpm dev
```

After HTTP or gateway type changes: `pnpm codegen`. Drift check: `pnpm check-codegen`.
