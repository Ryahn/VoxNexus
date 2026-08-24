# Communities

A community is the top-level social container on an instance (name, slug, description, timezone, join mode, icon/banner, cosmetics).

## Creation policy

Instance setting `community_creation_mode` (see [Instance](/docs/guides/instance)):

| Mode | Who can create |
|---|---|
| `open` | Any signed-in account |
| `admin_only` | Instance admins |
| `single` | At most one community (bootstrap / first create); further creates are rejected |

Creator becomes **owner** and a member. Optional bootstrap via `BOOTSTRAP_ADMIN_EMAIL` / `BOOTSTRAP_ADMIN_PASSWORD` / `BOOTSTRAP_COMMUNITY_NAME` in config.

## Join modes

| `join_mode` | Behavior |
|---|---|
| `open` | Members can `POST …/join` |
| `invite` | Join only via invite accept |
| `application` | Reserved; create/update rejects setting this mode for now |

## Ownership and delete

| Method | Path | Who |
|---|---|---|
| `POST` | `/api/v1/communities/{id}/transfer` | Owner; body `{ "account_id" }` (must already be a member) |
| `POST` | `/api/v1/communities/{id}/delete` | Owner; irreversible cascade |

Owners cannot leave until they transfer ownership.

## Cosmetics

In addition to icon and banner:

| Method | Path |
|---|---|
| `PUT` / `GET` | `/api/v1/communities/{id}/tag-badge` |
| `PUT` / `GET` | `/api/v1/communities/{id}/invite-splash` |

Owner uploads; members can fetch. URLs appear on the community response when set.

## HTTP

| Method | Path | Who |
|---|---|---|
| `POST` | `/api/v1/communities` | Per creation policy |
| `GET` | `/api/v1/communities` | Session; lists communities you belong to |
| `GET` | `/api/v1/communities/{id}` | Member |
| `PATCH` | `/api/v1/communities/{id}` | Owner |
| `PUT` | `/api/v1/communities/{id}/icon` · `/banner` | Owner |
| `GET` | `/api/v1/communities/{id}/icon` · `/banner` | Member |
| `POST` | `/api/v1/communities/{id}/join` | Session; open communities |
| `POST` | `/api/v1/communities/{id}/leave` | Member (not owner) |
| `GET` | `/api/v1/communities/{id}/members` | Member; cursor page |
| `PATCH` | `/api/v1/communities/{id}/members/me` | Member; nickname |

Membership role column: `owner` \| `member`. Custom [Roles](/docs/guides/roles) and [Permissions](/docs/guides/permissions) grant capabilities beyond that.

## Web UI

Community rail, create/join modals, settings (including transfer/delete), member list, and media uploads live in `apps/web`.
