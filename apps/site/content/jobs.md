# Background jobs (Redis + Apalis)

Feature Task F008J. Background workers run beside the HTTP server in the same `voxnexus` binary. Redis is the queue, not a system of record — job payloads are small IDs (into Postgres later), never blobs.

## Config

| Key | Meaning |
|---|---|
| `REDIS_URL` | Redis connection URL |

Startup opens a reconnecting connection, `PING`s Redis, then spawns the Apalis monitor. Ctrl+C / SIGTERM stops HTTP and workers together.

## Sample job: `HealthPing`

```rust
HealthPing { id: String } // UUIDv7
```

Namespace: `voxnexus::health_ping`. Exhausted retries land in the Redis dead-letter set for that namespace (`dead_letter_key()`).

Retry: tower `RetryPolicy` with 3 instant retries on the worker.

Enqueue helpers live in `voxnexus_jobs` (`enqueue_health_ping`). Later tasks (thumbnails, unfurl, search index) add typed jobs the same way.

## `/ready`

`redis` is **required**: `PING` must succeed. Typesense is required (F008S).

## Tests

Live tests run only when `REDIS_URL_TEST` is set:

```powershell
docker run -d --name voxnexus-redis -p 6379:6379 redis:7-alpine
$env:REDIS_URL_TEST="redis://127.0.0.1:6379"
cargo test -p voxnexus-jobs
```

- Enqueue + worker processes a ping
- Failed job retries until success
