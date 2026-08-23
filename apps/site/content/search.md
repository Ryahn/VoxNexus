# Search (Typesense)

Derived search index behind a `SearchEngine` trait. Postgres remains the source of truth; document indexing for messages is not wired yet.

## Config

| Key | Meaning |
|---|---|
| `TYPESENSE_URL` | Typesense HTTP base |
| `TYPESENSE_API_KEY` | API key |

Startup pings `/health` and ensures collections; failure refuses to listen. `/ready` requires Typesense healthy.

## Collections (schema v1)

Ensured empty at startup in `crates/search`:

| Name | Purpose |
|---|---|
| `messages` | Future message search |
| `users` | Future people search |
| `channels` | Future channel search |

Query path (when built): Typesense returns IDs → Rust applies authz → response. Never trust the search engine for permissions.

## Tests

Unit tests use an in-memory engine. Live tests need `TYPESENSE_URL_TEST`.
