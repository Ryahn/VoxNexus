# Configuration reference

Companion to [`config.example.toml`](config.example.toml). Copy that file to `config.toml` (Unix: `chmod 600`) or set the same names as environment variables. **Environment always wins** over the file.

Load order:

1. `VOXNEXUS_CONFIG` — path to a file (startup fails if set and missing)
2. `./config.toml`
3. `./config.yaml` / `./config.yml`
4. Environment only, if none of the above exist

Empty strings are treated as missing. Invalid values fail startup and name the key.

Boolean keys accept: `true` / `false`, `1` / `0`, `yes` / `no`, `on` / `off` (case-insensitive). TOML may also use bare `true`/`false`.

---

## Required

### `DATABASE_URL`

PostgreSQL connection URL. Migrations run on startup; `/ready` pings this database.

```toml
DATABASE_URL = "postgres://voxnexus:voxnexus@127.0.0.1:5432/voxnexus?sslmode=disable"
```

Compose default port on the host is `5432`. Must be a valid URL.

### `REDIS_URL`

Redis URL for Apalis job workers.

```toml
REDIS_URL = "redis://127.0.0.1:6379"
```

### `S3_ENDPOINT`

HTTP(S) base URL of the S3-compatible API (SeaweedFS in the Compose stack).

```toml
S3_ENDPOINT = "http://127.0.0.1:8333"
```

### `S3_ACCESS_KEY` / `S3_SECRET_KEY`

Credentials for the S3 API. Treated as secrets (not logged in plain form).

```toml
S3_ACCESS_KEY = "any"
S3_SECRET_KEY = "any"
```

Compose defaults match [`deploy/docker/seaweedfs-s3.json`](deploy/docker/seaweedfs-s3.json).

### `S3_BUCKET`

Bucket name. Created on startup if missing.

```toml
S3_BUCKET = "voxnexus"
```

Non-empty string required (not a URL).

### `TYPESENSE_URL` / `TYPESENSE_API_KEY`

Typesense HTTP API and API key. Collections (`messages`, `users`, `channels`) are ensured on startup. The key is a secret.

```toml
TYPESENSE_URL = "http://127.0.0.1:8108"
TYPESENSE_API_KEY = "xyz"
```

### `PUBLIC_URL`

Public origin of this instance. Used for CORS (allowed origin) and as the canonical base URL operators advertise.

```toml
PUBLIC_URL = "http://127.0.0.1:8080"
```

Must be a valid URL (scheme + host; include port when not on 80/443).

---

## Optional (with defaults)

### `COOKIE_SECURE`

Default: `false`

When `true`, session cookies use the `__Host-vn_session` name and the `Secure` flag (HTTPS only). When `false`, the cookie is `vn_session` without `Secure` (local HTTP).

```toml
COOKIE_SECURE = "false"
```

Set `true` behind TLS in production.

### `REGISTRATION_OPEN`

Default: `true`

When `true`, `POST /api/v1/auth/register` is allowed. When `false`, registration is rejected.

```toml
# REGISTRATION_OPEN = "true"
```

### `LOG_LEVEL`

Default: `info`

Process log verbosity (case-insensitive).

| Value | Meaning |
|---|---|
| `error` | Errors only |
| `warn` / `warning` | Warnings and above |
| `info` | Normal operational logs (default) |
| `debug` | Verbose debugging |
| `trace` | Most verbose |

```toml
LOG_LEVEL = "info"
```

### `LOG_FORMAT`

Default: `auto`

How logs are written to stderr (case-insensitive).

| Value | Behavior |
|---|---|
| `auto` | Pretty human text in **debug builds** when stderr is a TTY; **JSON** otherwise (release builds, or non-TTY such as Docker/systemd) |
| `pretty` | Always human-readable text |
| `json` | Always one JSON object per line (good for log aggregators) |

```toml
# LOG_FORMAT = "auto"
```

Examples:

```toml
LOG_FORMAT = "json"
LOG_FORMAT = "pretty"
LOG_FORMAT = "auto"
```

### `LISTEN_ADDR`

Default: `127.0.0.1:8080`

HTTP bind address as `host:port`. Use `0.0.0.0:8080` inside containers so the port is reachable from outside the container (Compose sets this).

```toml
# LISTEN_ADDR = "127.0.0.1:8080"
```

### `METRICS_ENABLED`

Default: `false`

When `true`, serves Prometheus metrics at `GET /metrics`.

```toml
# METRICS_ENABLED = "false"
```

### `GATEWAY_ALLOW_UNAUTH`

Default: `false`

When `true`, allows the `DEV_PING` / `DEV_PONG` events **after** a cookie-authenticated identify. Gateway upgrade itself always requires a valid session cookie.

```toml
# GATEWAY_ALLOW_UNAUTH = "false"
```

Pair with the web app: sign in, then `$env:VITE_GATEWAY_DEBUG="true"` and `pnpm dev`.

### `WEB_DIST`

Default: unset

Path to a directory of built SPA assets. When set to an existing directory, Axum serves the SPA (including `/`). Compose sets `WEB_DIST=/app/web`. For local API-only + Vite, leave unset and run `pnpm dev` separately.

```toml
# WEB_DIST = "apps/web/dist"
```

---

## Reserved (optional, unused yet)

These may be omitted. If set, URL-shaped values must still parse as URLs.

| Key | Intended use |
|---|---|
| `LIVEKIT_URL`, `LIVEKIT_API_KEY`, `LIVEKIT_API_SECRET` | Voice / LiveKit media plane |
| `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET` | External OIDC login |
| `SMTP_URL`, `SMTP_FROM` | Outbound email |

```toml
# LIVEKIT_URL = "http://127.0.0.1:7880"
# LIVEKIT_API_KEY = ""
# LIVEKIT_API_SECRET = ""
# OIDC_ISSUER = ""
# OIDC_CLIENT_ID = ""
# OIDC_CLIENT_SECRET = ""
# SMTP_URL = ""
# SMTP_FROM = ""
```

---

## Test-only env (not server config)

Used by integration tests; **not** read by `Config::load()`:

| Variable | Purpose |
|---|---|
| `DATABASE_URL_TEST` | Live Postgres tests |
| `REDIS_URL_TEST` | Live Redis / jobs tests |
| `S3_ENDPOINT_TEST` (+ related `S3_*_TEST`) | Live object-storage tests |
| `TYPESENSE_URL_TEST` / `TYPESENSE_API_KEY_TEST` | Live search tests |

---

## Secrets

Do not commit secrets. Prefer env vars in production. `config.toml` is gitignored; keep it private (Unix mode `0600`).
