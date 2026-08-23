# Configuration

VoxNexus loads an optional file, then overlays environment variables. **Environment always wins.** Missing or invalid required keys fail startup and name the key.

After load the process migrates Postgres, connects Redis, ensures the S3 bucket and Typesense collections, then serves HTTP/gateway (and optional SPA via `WEB_DIST`).

## File search order

1. `VOXNEXUS_CONFIG` — path to a file (fail if set and missing)
2. `./config.toml`
3. `./config.yaml` / `./config.yml`

Unix: config files must be mode **0600**. Keys match environment names — see [`config.example.toml`](/config.example.toml) and [`config.README.md`](/config.README.md).

## Required keys

| Key | Meaning | Example |
|---|---|---|
| `DATABASE_URL` | PostgreSQL | `postgres://voxnexus:voxnexus@127.0.0.1:5432/voxnexus?sslmode=disable` |
| `REDIS_URL` | Redis (jobs) | `redis://127.0.0.1:6379` |
| `S3_ENDPOINT` | SeaweedFS S3 API | `http://127.0.0.1:8333` |
| `S3_ACCESS_KEY` / `S3_SECRET_KEY` | S3 credentials | |
| `S3_BUCKET` | Bucket (created if missing) | `voxnexus` |
| `TYPESENSE_URL` / `TYPESENSE_API_KEY` | Typesense | `http://127.0.0.1:8108` |
| `PUBLIC_URL` | Public origin | `http://127.0.0.1:8080` |

## Common optional keys

| Key | Default | Meaning |
|---|---|---|
| `COOKIE_SECURE` | `false` | Secure / `__Host-` cookies behind TLS |
| `REGISTRATION_OPEN` | `true` | Early registration gate (prefer instance settings) |
| `COMMUNITY_CREATION_MODE` | `open` | `open` \| `admin_only` \| `single` |
| `COMMUNITY_CREATION_MODE_LOCKED` | `false` | Force-sync mode from config; block API overrides |
| `BOOTSTRAP_ADMIN_EMAIL` / `BOOTSTRAP_ADMIN_PASSWORD` | unset | First instance admin |
| `BOOTSTRAP_COMMUNITY_NAME` | unset | Seed community (useful with `single`) |
| `LOG_LEVEL` / `LOG_FORMAT` | `info` / `auto` | Tracing |
| `LISTEN_ADDR` | `127.0.0.1:8080` | Bind address |
| `METRICS_ENABLED` | `false` | `GET /metrics` |
| `GATEWAY_ALLOW_UNAUTH` | `false` | Dev-only unauthenticated gateway |
| `WEB_DIST` | unset | Built SPA directory (Compose sets this) |
| `OIDC_*` | unset | [OIDC](/docs/guides/oidc) |
| `LIVEKIT_*` / `SMTP_*` | unset | Later tasks |

Booleans accept `true`/`false`, `1`/`0`, `yes`/`no`, `on`/`off`.

## Two config places

1. **App** — `config.toml` for host `cargo run`
2. **Compose** — `deploy/docker/.env` for the `app` container (root `config.toml` is not mounted)

Keep overlapping secrets (S3, Typesense, `PUBLIC_URL`, cookie flags) aligned. Full key reference: `config.README.md`.
