# Continuous integration

Feature Task F010. GitHub Actions workflow [`.github/workflows/ci.yml`](../.github/workflows/ci.yml).

## Jobs

### `check` (every push / PR)

Matches the local quality gate in the README:

| Step | Command |
|---|---|
| Format | `cargo fmt --all -- --check` |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Tests | `cargo test --workspace` (Postgres + Redis service containers; MinIO + Typesense via `docker run`; `DATABASE_URL_TEST` / `REDIS_URL_TEST` / `S3_*_TEST` / `TYPESENSE_*_TEST`) |
| Web | `pnpm build` |
| Contracts | `pnpm check-codegen` |
| Biome | `pnpm lint` |

Live S3 / Typesense tests remain skipped unless their `*_TEST` env vars are set (Compose smoke covers those deps).

### `compose-smoke` (main / master push, or workflow_dispatch)

```text
cd deploy/docker
cp .env.example .env
docker compose -f docker-compose.yml up -d --build
curl /health → /ready → /api/v1/meta → SPA /
docker compose down -v
```

Skipped on pull requests so image builds do not block every PR. Re-run manually from the Actions tab (`workflow_dispatch`).

## Local parity

```powershell
pnpm install
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
$env:DATABASE_URL_TEST="postgres://voxnexus:voxnexus@127.0.0.1:5432/voxnexus?sslmode=disable"
$env:REDIS_URL_TEST="redis://127.0.0.1:6379"
cargo test --workspace
pnpm check-codegen
pnpm build
pnpm lint
```
