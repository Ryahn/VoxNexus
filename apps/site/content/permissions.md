# Permissions

Authz lives in `crates/permissions`. Handlers call `require_permission` / `require_manage_channels` / `require_channel_view` (and friends) in `crates/server`.

## Codes

| Code | Aliases | Meaning |
|---|---|---|
| `community.administrator` | | Bypass most checks (not owner-only paths) |
| `community.manage_channels` | | Categories, channels, overrides |
| `community.manage_roles` | | Roles, groups, assignments |
| `text.view` | `channel.view` | See a channel (lists and GET) |
| `text.send` | `message.send` | Send in a channel (enforced when messaging ships; stripped under timeout) |

Unknown codes fail validation on Explain Access.

## Resolution order

For a permission check (with optional channel context):

1. **Membership** — must be a community member (instance admins may skip this gate).
2. **Owner** — community owner → allow.
3. **Restricted Space** — if the channel's Space is restricted and the actor is not a space member → deny.
4. **Administrator** — `community.administrator` → allow (except owner-only permissions).
5. **Timeout** — timed-out members cannot `text.send`.
6. **Role grants** — merge assigned roles (including `@everyone`) by weight.
7. **Overrides** — category layer, then channel layer. Within a layer: collapse role overrides for the actor's roles, then apply the member override (member allow/deny wins over role at that layer). Channel can restore what a category denied.

Hidden channels return **404** (not 403) when the actor lacks `text.view`.

## Overrides HTTP

Requires `community.manage_channels`.

| Scope | List | Upsert role | Upsert member | Delete |
|---|---|---|---|---|
| Channel | `GET /api/v1/channels/{id}/permission-overrides` | `PUT …/permission-overrides/roles/{role_id}` | `PUT …/permission-overrides/members/{account_id}` | `DELETE /api/v1/communities/{id}/permission-overrides/{override_id}` |
| Category | `GET /api/v1/categories/{id}/permission-overrides` | `PUT …/permission-overrides/roles/{role_id}` | `PUT …/permission-overrides/members/{account_id}` | same delete path |

Upsert body: `{ "permissions": … }` allow/deny sets (see OpenAPI schema).

## Explain Access

`POST /api/v1/permissions/explain`

```json
{
  "community_id": "…",
  "account_id": "…",
  "permission": "text.view",
  "channel_id": "…"
}
```

Returns `decision` (`allow` \| `deny`) and ordered `steps` (membership, owner, space, administrator, roles, overrides, …). Callers may explain themselves; explaining another account requires `community.manage_channels`.

## View As

`POST /api/v1/permissions/view-as/channels` (requires `community.manage_channels`)

Simulate the channel list for:

| `mode` | Body | Behavior |
|---|---|---|
| `visitor` | — | Non-member; typically sees no channels |
| `member` | `account_id` | That member's roles + member overrides |
| `roles` | `role_ids` | `@everyone` plus those roles (no member overrides). Assumes Space membership when `space_id` is set |

Optional `space_id` scopes the list. Response: `{ mode, label, channels }` using the **same** `resolve` + override path as live checks.

**Simulation only:** View As does not change the session. Mutating APIs always use the real actor. The web client shows a toolbar (community menu → **View As…**) and filters the sidebar; create/reorder stay disabled while previewing.

## Still owner-only

Community settings, invites, cosmetics, Space CRUD, and Space admin member add/remove do **not** yet use role permissions — community owner only.
