# Configuration

VoxNexus loads process configuration from an optional file, then overlays environment variables. **Environment always wins.** The server refuses to start if a required value is missing or invalid, and the error names the key.

This is Feature Task F002–F009. The process connects to PostgreSQL, migrates, connects to Redis (Apalis workers), ensures the S3 bucket, ensures Typesense collections, and listens for `/health`, `/ready`, `/api/v1`, the gateway WebSocket (when allowed), and optionally the built SPA via `WEB_DIST`. CORS uses the origin from `PUBLIC_URL`.

## File search order

1. `VOXNEXUS_CONFIG` — absolute or relative path to a file. If set and the file is missing, startup fails.
2. `./config.toml`
3. `./config.yaml`
4. `./config.yml`

If none of those exist, environment variables alone are enough.

On Unix, a config file must be mode **0600** (not group- or world-readable) because it may contain secrets. Windows does not enforce a POSIX mode; keep the file private anyway.

File keys use the same names as environment variables. See [`config.example.toml`](../config.example.toml).

## Required keys

| Key | Meaning | Example |
|---|---|---|
| `DATABASE_URL` | PostgreSQL URL | `postgres://voxnexus:voxnexus@127.0.0.1:5432/voxnexus?sslmode=disable` |
| `REDIS_URL` | Redis URL (jobs) | `redis://127.0.0.1:6379` |
| `S3_ENDPOINT` | SeaweedFS S3 API | `http://127.0.0.1:8333` |
| `S3_ACCESS_KEY` | S3 access key | |
| `S3_SECRET_KEY` | S3 secret key | |
| `S3_BUCKET` | Bucket name | `voxnexus` |
| `TYPESENSE_URL` | Typesense HTTP API | `http://127.0.0.1:8108` |
| `TYPESENSE_API_KEY` | Typesense API key | |
| `PUBLIC_URL` | Public origin of this instance | `http://127.0.0.1:8080` |

URL values must parse as URLs. Empty strings are treated as missing.

## Optional keys (defaults)

| Key | Default | Meaning |
|---|---|---|
| `COOKIE_SECURE` | `false` | Set `true` behind TLS (`true`/`false`, `1`/`0`, `yes`/`no`, `on`/`off`) |
| `REGISTRATION_OPEN` | `true` | Allow `POST /api/v1/auth/register` until instance settings (F017) |
| `LOG_LEVEL` | `info` | `error`, `warn`, `info`, `debug`, or `trace` |
| `LOG_FORMAT` | `auto` | `auto`, `pretty`, or `json`. Auto is pretty in debug+TTY, JSON otherwise |
| `LISTEN_ADDR` | `127.0.0.1:8080` | HTTP bind address (`host:port`) |
| `METRICS_ENABLED` | `false` | Serve Prometheus `GET /metrics` |
| `GATEWAY_ALLOW_UNAUTH` | `false` | Allow unauthenticated WebSocket gateway (dev only; see [`gateway.md`](gateway.md)) |
| `WEB_DIST` | _(unset)_ | Directory of built SPA assets. When set to an existing dir, Axum serves the SPA (Compose sets `/app/web`) |

HTTP probes are documented in [`observability.md`](observability.md). API errors and `GET /api/v1/meta` are in [`api.md`](api.md). Auth cookies/sessions: [`auth.md`](auth.md). Gateway: [`gateway.md`](gateway.md). Object storage: [`storage.md`](storage.md). Jobs: [`jobs.md`](jobs.md). Search: [`search.md`](search.md). Compose: [`compose.md`](compose.md).

## Optional keys (unused until later Feature Tasks)

These may be omitted. Invalid URLs still fail parse if they are set.

| Key | Later task |
|---|---|
| `LIVEKIT_URL`, `LIVEKIT_API_KEY`, `LIVEKIT_API_SECRET` | F061 |
| `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `OIDC_ONLY`, `OIDC_LINK_BY_EMAIL` | F018O — see `apps/site/content/oidc.md` |
| `SMTP_URL`, `SMTP_FROM` | F117 |

## Secrets

Do not put secrets in the repository. Prefer environment variables in production. If you use a file, keep it 0600 and out of git (`config.toml` is gitignored).

Live database tests use `DATABASE_URL_TEST` (not loaded by the server). See [`database.md`](database.md).
