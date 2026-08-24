# Developing

Short map for contributing to the current product surface.

## Prerequisites

Rust (stable via `rust-toolchain.toml`), pnpm 10, Docker (or local Postgres 16 + Redis + SeaweedFS S3 + Typesense).

## Typical loop

1. Start deps: [Docker Compose](/docs/setup/compose) (full stack or deps-only).
2. Copy `config.example.toml` → `config.toml`; point URLs at published ports.
3. API: `cargo run -p voxnexus`
4. Web: `pnpm dev` (Vite proxies `/api` and the gateway to `:8080`)
5. Docs site: `pnpm dev:site`

## Quality gate

```powershell
pnpm install
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm check-codegen
pnpm build
pnpm lint
```

Live integration tests need `DATABASE_URL_TEST` / `REDIS_URL_TEST` (optional `S3_*_TEST` / `TYPESENSE_*_TEST`). Unset → those tests skip.

## Where to change what

| Change | Touch |
|---|---|
| HTTP route / DTO | `crates/server`, `crates/protocol`, then `pnpm codegen` |
| Gateway event | `crates/protocol` + `crates/realtime`, then `pnpm codegen` |
| Schema | `migrations/*.up.sql` + `*.down.sql`; types in `crates/domain` |
| Session / password / OIDC | `crates/auth`, handlers in `crates/server` |
| Permission codes / eval / overrides | `crates/permissions`, gates in `crates/server` |
| Web UI | `apps/web` via `@voxnexus/api-client` / `@voxnexus/protocol` |
| Operator docs | `apps/site/content/*.md` + `nav.ts` / `content.ts` / `App.tsx` |
