# Roles

Communities have an `@everyone` role (auto-created) plus custom roles. Role grants feed the [permission engine](/docs/guides/permissions). Membership still uses `owner` \| `member` on `community_members`; roles are an additional grant layer.

## Role fields

| Field | Notes |
|---|---|
| `name` | Display name |
| `position` | Display order only (drag-reorder) |
| `weight` | Unique 1–1000; **lower = higher priority** when merging grants |
| `permissions` | JSON grant set (family/bitmaps or codes — see OpenAPI) |
| `is_everyone` | Built-in; not deletable |
| `color` / `hoist` / `mentionable` | Presentation |
| `group_id` | Optional [role group](#role-groups) |
| `short_tag` / `icon_emoji` / `icon_object_key` / `gradient` / `role_card` | Cosmetics |

New members receive `@everyone`. By default that role includes `text.view` so channels are visible unless overridden.

## HTTP — roles

| Method | Path | Who |
|---|---|---|
| `POST` | `/api/v1/communities/{id}/roles` | `community.manage_roles` |
| `GET` | `/api/v1/communities/{id}/roles` | Member |
| `GET` | `/api/v1/roles/{role_id}` | Member of that community |
| `PATCH` | `/api/v1/roles/{role_id}` | `community.manage_roles` |
| `DELETE` | `/api/v1/roles/{role_id}` | `community.manage_roles` (not `@everyone`) |
| `POST` | `/api/v1/communities/{id}/roles/reorder` | `community.manage_roles`; body `{ "role_ids": […] }` |
| `POST` | `/api/v1/roles/{role_id}/clone` | `community.manage_roles` |
| `PUT` / `GET` / `DELETE` | `/api/v1/roles/{role_id}/icon` | Manage / member / manage |

## Assignments

| Method | Path | Who |
|---|---|---|
| `GET` | `/api/v1/communities/{id}/members/{account_id}/roles` | Member |
| `PUT` | `/api/v1/communities/{id}/members/{account_id}/roles` | `community.manage_roles`; replace set |
| `PUT` | `/api/v1/communities/{id}/members/{account_id}/roles/{role_id}` | Assign one |
| `DELETE` | `/api/v1/communities/{id}/members/{account_id}/roles/{role_id}` | Remove one |

## Role groups

Named folders for organizing roles in the UI; bulk-assign applies a group's roles to members.

| Method | Path |
|---|---|
| `POST` / `GET` | `/api/v1/communities/{id}/role-groups` |
| `PATCH` / `DELETE` | `/api/v1/role-groups/{group_id}` |
| `POST` | `/api/v1/communities/{id}/role-groups/bulk-assign` |

## Web UI

Role manager (create, reorder, cosmetics, permissions), member role assignment, and role groups.
