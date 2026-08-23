# Database

VoxNexus uses PostgreSQL 16 as the system of record. The `voxnexus` process opens a SQLx pool on startup and applies versioned migrations from `/migrations` before it does anything else.

This is Feature Task F003. There is still no HTTP API.

## Startup

`sqlx migrate` runs **inside the binary** (`voxnexus_db::connect_and_migrate`). You do not need a separate `sqlx migrate run` for the app to start. Compose (F009) will still need a reachable Postgres; it will not replace in-process migrate.

Pool settings (F003): 10 connections, 5s acquire timeout (also bounds initial connect). Health is `SELECT 1`.

## Local Postgres

Example (not the production Compose stack — that is F009):

```powershell
docker run --name voxnexus-pg -d --rm -e POSTGRES_USER=voxnexus -e POSTGRES_PASSWORD=voxnexus -e POSTGRES_DB=voxnexus -p 5432:5432 postgres:16
```

Then set `DATABASE_URL` (see [`config.md`](config.md)). For local servers without TLS, add `sslmode=disable`:

```text
postgres://voxnexus:voxnexus@127.0.0.1:5432/voxnexus?sslmode=disable
```

## Migrations

Files live in [`migrations/`](../migrations) at the repository root. Naming follows SQLx reversible migrations: `{version}_{name}.up.sql` and `{version}_{name}.down.sql`.

F003 ships a baseline only. Product tables start in later Feature Tasks (accounts/sessions: F011–F012; see [`auth.md`](auth.md)). SQLx records applied versions in `_sqlx_migrations`.

## Tests

Live database tests **do not use testcontainers** in F003. They read **`DATABASE_URL_TEST`**.

- Unset or empty: tests print a skip message and pass (so `cargo test` still works without Postgres).
- Set: tests connect, `SELECT 1`, apply migrations, revert to version 0, and apply again.

Use a **throwaway** database. Revert will run against whatever `DATABASE_URL_TEST` points at.

```powershell
$env:DATABASE_URL_TEST = "postgres://voxnexus:voxnexus@127.0.0.1:5432/voxnexus_test?sslmode=disable"
createdb voxnexus_test   # or CREATE DATABASE voxnexus_test;
cargo test --workspace
```

F010 CI should set `DATABASE_URL_TEST` against a Postgres 16 service so these tests actually run on every PR.
