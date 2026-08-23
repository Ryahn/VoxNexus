# Search (Typesense)

Feature Task F008S. Derived search index behind a `SearchEngine` trait. PostgreSQL remains the system of record; Typesense is rebuilt from it (F057).

## Config

| Key | Meaning |
|---|---|
| `TYPESENSE_URL` | Typesense HTTP base (e.g. `http://127.0.0.1:8108`) |
| `TYPESENSE_API_KEY` | Admin / search API key |

Startup builds a `TypesenseClient`, `PING`s `/health`, and ensures collections. The process refuses to listen if that fails.

## Collections (schema v1)

Versioned in `crates/search` (`SCHEMA_VERSION`). Ensured empty at startup; indexers arrive in F057.

| Name | Fields (plus Typesense `id`) |
|---|---|
| `messages` | `community_id`, `channel_id`, `author_id`, `body`, `created_at`, `schema_version` |
| `users` | `username`, `display_name?`, `schema_version` |
| `channels` | `community_id`, `name`, `schema_version` |

## `/ready`

`typesense` is **required**: `/health` must report ok.

## API surface (`crates/search`)

- `SearchEngine`: `ping`, `ensure_collections`, `upsert_document`, `delete_document`, `search`
- `TypesenseClient` via `reqwest` (rustls) + `X-TYPESENSE-API-KEY`
- `MemorySearchEngine` for unit tests only

## Tests

- Unit: memory upsert / search / delete
- Live: only when `TYPESENSE_URL_TEST` is set. Optional `TYPESENSE_API_KEY_TEST` (else `TYPESENSE_API_KEY`, else `xyz`):

```powershell
docker run -d --name voxnexus-typesense -p 8108:8108 `
  -e TYPESENSE_DATA_DIR=/data `
  -e TYPESENSE_API_KEY=asdfasdfasdf23445345 `
  -v typesense-data:/data `
  typesense/typesense:27.1 `
  --data-dir /data --api-key=asdfasdfasdf23445345 --enable-cors

$env:TYPESENSE_URL_TEST="http://127.0.0.1:8108"
$env:TYPESENSE_API_KEY_TEST="asdfasdfasdf23445345"
cargo test -p voxnexus-search --test roundtrip
```
