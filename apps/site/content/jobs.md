# Background jobs

Apalis workers run in the same `voxnexus` binary. Redis is the queue, not a system of record — payloads are small IDs, not blobs.

## Config

`REDIS_URL` is required. Startup PINGs Redis, then starts workers. Shutdown stops HTTP and workers together. `/ready` requires Redis.

## Sample job

`HealthPing { id }` under namespace `voxnexus::health_ping` exercises enqueue, retry, and dead-letter. Later indexers/thumbnails follow the same pattern.

## Tests

Live tests need `REDIS_URL_TEST`.
