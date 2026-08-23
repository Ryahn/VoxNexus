# Observability

| Path | Auth | Meaning |
|---|---|---|
| `GET /health` | none | Liveness. Process is up; no dependency checks. |
| `GET /ready` | none | Postgres + Redis + SeaweedFS `HeadBucket` + Typesense `/health`. |
| `GET /metrics` | none | Prometheus text when `METRICS_ENABLED=true` (restrict exposure in production). |

Every response includes `x-request-id` (UUIDv7). Incoming values are echoed when present.

## Logs

`tracing` via `LOG_LEVEL` and `LOG_FORMAT` (`auto` | `pretty` | `json`). Auto is pretty in debug+TTY, JSON otherwise. HTTP spans carry `request_id`.

See [Configuration](/docs/setup/config).
