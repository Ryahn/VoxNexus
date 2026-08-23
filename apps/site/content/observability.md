# Observability

Feature Task F004. The `voxnexus` process listens for HTTP probes after it has loaded config and migrated Postgres.

## Endpoints

| Path | Auth | Meaning |
|---|---|---|
| `GET /health` | none | Liveness. Process is up. Does not check dependencies. **200** `{ "status": "ok" }`. |
| `GET /ready` | none | Readiness. Postgres `SELECT 1`, Redis `PING`, SeaweedFS `HeadBucket`, and Typesense `/health` are required. **200** if all pass, **503** otherwise. |
| `GET /metrics` | none | Prometheus text. **Off by default** (`METRICS_ENABLED=false`). When enabled, unauthenticated (bind `LISTEN_ADDR` to localhost in production unless you scrape through a trusted network). |

Every response includes `x-request-id` (UUIDv7). Incoming `x-request-id` is kept and echoed. Unknown paths on the full app return the JSON error envelope in [`api.md`](api.md).

## Logs

`tracing` + `tracing-subscriber`. `LOG_LEVEL` sets verbosity. `LOG_FORMAT`:

- `auto` (default): pretty when compiling with debug assertions **and** stderr is a TTY; JSON otherwise (release / journald / files)
- `pretty` or `json` to force

JSON events include `request_id` from the HTTP span when a request is in flight.

## Config

See [`config.md`](config.md) for `LISTEN_ADDR` (default `127.0.0.1:8080`), `METRICS_ENABLED`, and `LOG_FORMAT`.
