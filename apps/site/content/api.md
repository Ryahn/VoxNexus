# HTTP API conventions

Product routes live under `/api/v1`. Probes: `/health`, `/ready`, optional `/metrics` — [Observability](/docs/guides/observability).

Interactive OpenAPI: [/docs/api](/docs/api). Generated client: `@voxnexus/api-client` — [Codegen](/docs/guides/codegen).

## Sample public route

`GET /api/v1/meta` → `{ "name": "voxnexus", "version": "…", … }` (login-mode hints for the SPA). No auth.

## Errors

```json
{
  "code": "not_found",
  "message": "The requested resource was not found.",
  "details": { "fields": { "name": ["length"] } },
  "request_id": "018f..."
}
```

`details` omitted when empty. `request_id` matches `x-request-id` (UUIDv7).

| HTTP | `code` | When |
|---|---|---|
| 400 | `invalid_json` | Body not JSON / wrong Content-Type |
| 400 | `validation_error` | Field rules failed (`details.fields`) |
| 401 | `unauthenticated` | Missing/invalid session or bad password |
| 403 | `permission_denied` | CSRF, policy, or authz |
| 404 | `not_found` | Unknown path or hidden resource |
| 409 | `conflict` | Duplicate email / slug / etc. |
| 429 | `rate_limited` | Reserved for future rate limiting |
| 500 | `internal` | Unexpected failure |

## Pagination

Cursor params in `crates/protocol`: `before`, `after`, `limit` (default **50**, max **100**). List payloads use `items` + `has_more` (or domain wrappers like `communities` / `spaces` / `channels` — see OpenAPI).

## Limits and middleware

- JSON body cap: **6 MiB** (uploads use their own limits)
- Optional gzip responses
- CORS origin from `PUBLIC_URL` (credentials on)
- CSRF Origin/Referer checks on mutating `/api` methods — [Authentication](/docs/guides/auth)

Prefer validated JSON extractors so clients always see this error shape.

## Surface map

| Area | Doc |
|---|---|
| Auth / OIDC | [Authentication](/docs/guides/auth), [OIDC](/docs/guides/oidc) |
| Profiles / presence | [Profiles](/docs/guides/profiles) |
| Instance | [Instance](/docs/guides/instance) |
| Communities / members | [Communities](/docs/guides/communities) |
| Invites | [Invites](/docs/guides/invites) |
| Spaces | [Spaces](/docs/guides/spaces) |
| Categories / channels | [Channels](/docs/guides/channels) |
| Roles | [Roles](/docs/guides/roles) |
| Permissions / overrides | [Permissions](/docs/guides/permissions) |
| Gateway | [Gateway](/docs/guides/gateway) |
