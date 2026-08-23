# Continuous integration

Workflow: `.github/workflows/ci.yml`.

## `check` (every push / PR)

| Step | Command |
|---|---|
| Format | `cargo fmt --check` |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Tests | `cargo test --workspace` (Postgres + Redis services; optional S3/Typesense via `*_TEST`) |
| Web | `pnpm build` |
| Contracts | `pnpm check-codegen` |
| Biome | `pnpm lint` |

## `compose-smoke` (main / manual)

`deploy/docker` up → `/health` → `/ready` → `/api/v1/meta` → SPA `/` → down. Skipped on PRs by default.

Local parity commands: [Developing](/docs/guides/develop).
