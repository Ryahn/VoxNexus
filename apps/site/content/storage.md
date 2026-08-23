# Object storage

Attachment bytes live in an S3-compatible store (SeaweedFS in Compose). Local disk is not a production object store.

## Config

| Key | Meaning |
|---|---|
| `S3_ENDPOINT` | S3 API base URL |
| `S3_ACCESS_KEY` / `S3_SECRET_KEY` | Credentials |
| `S3_BUCKET` | Bucket name (created on startup if missing) |

Path-style addressing is forced so SeaweedFS / MinIO / LocalStack work without virtual-host DNS.

## Startup and readiness

After migrate, the process builds the S3 client, ensures the bucket, and refuses to listen on failure. `/ready` requires a successful `HeadBucket`.

## Used by
Profile avatars/banners and community icons/banners. Keys and metadata live in Postgres; bytes in the bucket. Downloads go through authenticated app routes (not public SeaweedFS URLs).

## Crate

`crates/storage`: `ObjectStore` trait (`put` / `get` / `delete` / `presign_get` / …), `S3ObjectStore`, and an in-memory store for tests.

Live S3 tests run only when `S3_ENDPOINT_TEST` is set — see [Developing](/docs/guides/develop).
