# VoxNexus

A self-hostable community chat system: Discord-class chat and voice, Guilded-class organization (Spaces), and a first-class app/workflow platform.

This is **not** a hosted SaaS and **not** an OSI open-source project. Default install (later phases) is a single server via Docker Compose.

## License (not OSI)

VoxNexus is **source-available** under the [VoxNexus Source-Available Personal Use License v1](LICENSE). You may study the code and run one private personal instance. You may not use it commercially, operate a public instance, reuse the code in other projects, or redistribute it, except by contributing under the [CLA](CLA.md).

See [LICENSE.md](LICENSE.md) for a plain-language FAQ and [NOTICE](NOTICE) for the short form.

## Status

Engineering foundation (Feature Task F006). The `voxnexus` binary serves `/health`, `/ready`, optional `/metrics`, and `GET /api/v1/meta`. The web app loads instance name and version through a generated TypeScript client. There is no Docker stack yet.

Development follows [`docs/MASTER_PLAN.md`](docs/MASTER_PLAN.md). See [`docs/config.md`](docs/config.md), [`docs/database.md`](docs/database.md), [`docs/observability.md`](docs/observability.md), [`docs/api.md`](docs/api.md), and [`docs/codegen.md`](docs/codegen.md).

## Repository layout

```text
apps/web          Vite + React SPA
crates/server     voxnexus binary (composition root)
crates/config     env + file configuration
crates/db         PostgreSQL pool and migrations
crates/*          other domain crates (stubs until later Feature Tasks)
packages/api-client  generated OpenAPI TypeScript client
packages/ui       shared presentational components
tools/codegen     export OpenAPI from Rust handlers
migrations/       SQLx migration files
docs/             master plan and operator docs
```

## Build

Requires [Rust](https://rustup.rs/) (stable, via `rust-toolchain.toml`) and [pnpm](https://pnpm.io/).

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm check-codegen
pnpm install
pnpm build
```

Database integration tests (including `/ready`) run only when `DATABASE_URL_TEST` is set (see [`docs/database.md`](docs/database.md)). `/health` and `x-request-id` tests do not need Postgres.

## Run the server

Copy [`config.example.toml`](config.example.toml) to `config.toml` (Unix: `chmod 600`) or set the same keys as environment variables. PostgreSQL 16 must be reachable; migrations run on startup. The process then listens (default `127.0.0.1:8080`).

```powershell
cargo run -p voxnexus
```

```powershell
curl.exe http://127.0.0.1:8080/health
curl.exe http://127.0.0.1:8080/ready
curl.exe http://127.0.0.1:8080/api/v1/meta
```

Without required keys (for example `DATABASE_URL`), the process prints the missing key name and exits non-zero. If Postgres is down at startup, it exits non-zero after the connect timeout.

## Web

With the API listening on `127.0.0.1:8080`, start the SPA. Vite proxies `/api` to the binary so `getMeta` hits `GET /api/v1/meta`.

```powershell
pnpm --filter @voxnexus/web dev
```

After HTTP type changes, regenerate the client (`pnpm codegen`). Drift: `pnpm check-codegen`.
