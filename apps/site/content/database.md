# Database

PostgreSQL 16 is the system of record. The `voxnexus` binary opens a SQLx pool and runs migrations from `/migrations` before serving traffic.

## Startup

`connect_and_migrate` runs inside the process — no separate migrate step for normal boots. Pool: 10 connections, 5s acquire timeout. Health: `SELECT 1` (used by `/ready`).

## Local Postgres

```powershell
docker run --name voxnexus-pg -d --rm `
  -e POSTGRES_USER=voxnexus -e POSTGRES_PASSWORD=voxnexus -e POSTGRES_DB=voxnexus `
  -p 5432:5432 postgres:16
```

```text
DATABASE_URL=postgres://voxnexus:voxnexus@127.0.0.1:5432/voxnexus?sslmode=disable
```

Prefer Compose for the full dependency set — [Docker Compose](/docs/setup/compose).

## Migrations

Reversible SQLx files: `{version}_{name}.up.sql` / `.down.sql` at repo-root `migrations/`. Applied versions live in `_sqlx_migrations`.

Current product tables include accounts/sessions, profiles/presence columns, instance settings, communities/members, invites, and spaces.

## Tests

Live DB tests use **`DATABASE_URL_TEST`** (not loaded by the server). Unset → skip. Set → connect, migrate, and exercise against a **throwaway** database (some tests revert).

```powershell
$env:DATABASE_URL_TEST = "postgres://voxnexus:voxnexus@127.0.0.1:5432/voxnexus_test?sslmode=disable"
cargo test --workspace
```

CI sets this against a Postgres 16 service — [CI](/docs/guides/ci).
