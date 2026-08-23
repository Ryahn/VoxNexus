# Object storage

Feature Task F008. Attachment bytes live in an S3-compatible store (SeaweedFS in Compose). The app never uses the local disk as a production object store.

## Config

Uses the existing S3 keys from [`config.md`](config.md):

| Key | Meaning |
|---|---|
| `S3_ENDPOINT` | S3 API base URL (SeaweedFS `weed server -s3`, LocalStack, etc.) |
| `S3_ACCESS_KEY` / `S3_SECRET_KEY` | Credentials |
| `S3_BUCKET` | Bucket name (created on startup if missing) |

Path-style addressing is forced so SeaweedFS / MinIO / LocalStack work without virtual-host DNS. The AWS SDK is configured with checksum calculation/validation **WhenRequired** so SeaweedFS is not broken by AWS's newer default integrity headers.

## Startup

After Postgres migrate, the process builds an `S3ObjectStore`, runs **ensure bucket** (`HeadBucket`, then `CreateBucket` if missing), and refuses to listen if that fails.

## `/ready`

`seaweedfs` is **required**: `HeadBucket` must succeed. Redis and Typesense are required (F008J / F008S).

## API surface (`crates/storage`)

- `ObjectStore` trait: `put`, `get`, `delete`, `presign_get`, `head_bucket`, `ensure_bucket`
- `ObjectKey::parse` rejects `..`, absolute paths, backslashes, empty segments
- `MemoryObjectStore` for unit tests only
- `S3ObjectStore` via `aws-sdk-s3` (rustls)

## Tests

- Unit: key validation + memory put/get/delete (always run; no S3 required)
- Live S3: only when `S3_ENDPOINT_TEST` is set **and** that URL answers. Credentials: `S3_ACCESS_KEY_TEST` / `S3_SECRET_KEY_TEST` / `S3_BUCKET_TEST`, else app `S3_*` env vars, else `any` / `any` / `voxnexus-test`. Match keys to the store (same as `config.toml` for the SeaweedFS container below).

`dispatch failure` means the TCP connect failed (nothing on that host:port). Unset the env var to skip, or start SeaweedFS:

```powershell
# after Docker Desktop is running — quote dotted flags (PowerShell splits -s3.config otherwise)
$s3Json = Join-Path $env:TEMP "voxnexus-seaweed-s3.json"
@'
{"identities":[{"name":"voxnexus","credentials":[{"accessKey":"12345asdfg","secretKey":"67890vbnm,kmhg"}],"actions":["Admin","Read","Write","List","Tagging"]}]}
'@ | Set-Content -Path $s3Json -Encoding utf8

docker run -d --name voxnexus-seaweedfs -p 8333:8333 `
  -v "${s3Json}:/etc/seaweedfs/s3.json:ro" `
  chrislusf/seaweedfs server "-dir=/data" "-s3" "-s3.config=/etc/seaweedfs/s3.json" "-ip.bind=0.0.0.0"

$env:S3_ENDPOINT_TEST="http://127.0.0.1:8333"
$env:S3_ACCESS_KEY_TEST="12345asdfg"
$env:S3_SECRET_KEY_TEST="67890vbnm,kmhg"
$env:S3_BUCKET_TEST="voxnexus"
cargo test -p voxnexus-storage --test s3_roundtrip

Remove-Item Env:S3_ENDPOINT_TEST
```

Compose (F009) will own the long-lived SeaweedFS service; until then this container matches `config.toml`.
