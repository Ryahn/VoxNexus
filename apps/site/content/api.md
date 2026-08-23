# HTTP API conventions

Feature Task F005. Product routes live under `/api/v1`. Probes stay at `/health`, `/ready`, and optional `/metrics` (see [`observability.md`](observability.md)).

## Sample route

`GET /api/v1/meta` returns `{ "name": "voxnexus", "version": "<crate version>" }`. No authentication.

The OpenAPI document and generated TypeScript client live in `packages/api-client`. See [`codegen.md`](codegen.md).

## Errors

Every API and unknown-route error uses this JSON body (never a stack trace):

```json
{
  "code": "not_found",
  "message": "The requested resource was not found.",
  "details": { "fields": { "name": ["length"] } },
  "request_id": "018f..."
}
```

`details` is omitted when empty. `request_id` matches `x-request-id` (UUIDv7).

| HTTP | `code` | When |
|---|---|---|
| 400 | `invalid_json` | Body is not JSON or the `Content-Type` is wrong |
| 400 | `validation_error` | JSON parsed but failed field rules (`details.fields`) |
| 401 | `unauthenticated` | Missing/invalid session, or wrong password |
| 403 | `permission_denied` | CSRF failure, registration closed, or later authz |
| 404 | `not_found` | Unknown path, or a resource the caller must not see |
| 409 | `conflict` | Duplicate email (or identity) |
| 429 | `rate_limited` | (F116) |
| 500 | `internal` | Unexpected server failure |

## Pagination

List endpoints use cursor query params in `crates/protocol`: `before`, `after`, and `limit` (default **50**, max **100**). Responses include `items` and `has_more`.

## Limits and middleware

- Request body cap: **1 MiB**.
- Responses may be gzip-compressed.
- CORS allows the origin derived from `PUBLIC_URL` (credentials on).
- CSRF Origin/Referer checks run on mutating methods (see [`auth.md`](auth.md)).

Handlers that accept JSON should use `ValidatedJson<T>` (or `AppJson<T>` when there are no field rules) so clients always see the error schema.

Auth routes: [`auth.md`](auth.md).
