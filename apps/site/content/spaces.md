# Spaces

A Space is a flat group inside a community (Guilded-style). Spaces are not nested. Categories and channels may optionally attach to a Space via `space_id`.

## Fields

| Field | Notes |
|---|---|
| `name` | Required, ≤100 chars |
| `description` | Optional, ≤2000 |
| `topic` | Optional short topic, ≤200 |
| `game` | Optional game/label metadata, ≤100 |
| `visibility` | `open` (default) or `restricted` |
| `position` | Ordering within the community |
| `icon_url` | Reserved; icon upload not available yet |
| `is_member` | Whether the caller is in `space_members` |

## Visibility and membership

| Visibility | Who can see / join |
|---|---|
| `open` | Community members can list and `POST …/join`. |
| `restricted` | Listed only for space members and the community owner. Join is blocked; an owner adds members with `POST …/members`. |

Anyone in the community can leave a Space they belong to. Restricted Spaces also gate permission evaluation: non-members are denied before role grants apply (see [Permissions](/docs/guides/permissions)).

Space **create / update / delete** and **admin add/remove** are community **owner** only today.

## HTTP

| Method | Path | Who |
|---|---|---|
| `POST` | `/api/v1/communities/{community_id}/spaces` | Owner |
| `GET` | `/api/v1/communities/{community_id}/spaces` | Member (filtered by visibility) |
| `GET` | `/api/v1/spaces/{space_id}` | Member of parent community (restricted: member or owner) |
| `PATCH` | `/api/v1/spaces/{space_id}` | Owner |
| `DELETE` | `/api/v1/spaces/{space_id}` | Owner |
| `POST` | `/api/v1/spaces/{space_id}/join` | Community member; open Spaces only |
| `POST` | `/api/v1/spaces/{space_id}/leave` | Space member |
| `GET` | `/api/v1/spaces/{space_id}/members` | Space member or community owner |
| `POST` | `/api/v1/spaces/{space_id}/members` | Owner; body `{ "account_id" }` |
| `DELETE` | `/api/v1/spaces/{space_id}/members/{account_id}` | Owner |

## Web UI

Space switcher in the community sidebar, create-Space modal, and space member management. Channel lists respect Space scope and visibility.
