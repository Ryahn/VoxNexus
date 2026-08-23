# Communities

A community is the top-level social container on an instance (name, slug, description, timezone, join mode, icon/banner).

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
| `invite` | Join only via invite accept (or owner paths) |
| `application` | Reserved; not directly joinable yet |

## HTTP

| Method | Path | Who |
|---|---|---|
| `POST` | `/api/v1/communities` | Per creation policy |
| `GET` | `/api/v1/communities` | Session; lists communities you belong to |
| `GET` | `/api/v1/communities/{id}` | Member |
| `PATCH` | `/api/v1/communities/{id}` | Owner |
| `PUT` | `/api/v1/communities/{id}/icon` · `/banner` | Owner (image upload) |
| `GET` | `/api/v1/communities/{id}/icon` · `/banner` | Member |
| `POST` | `/api/v1/communities/{id}/join` | Session; open communities |
| `POST` | `/api/v1/communities/{id}/leave` | Member (owner leave blocked until ownership transfer exists) |
| `GET` | `/api/v1/communities/{id}/members` | Member; cursor page |
| `PATCH` | `/api/v1/communities/{id}/members/me` | Member; nickname |

Member roles today: `owner` \| `member`. Custom roles and a full permission engine are not available yet.

## Web UI

Community rail, create/join modals, settings, member list, and icon/banner uploads live in `apps/web`. Chat UI beyond that is still largely mock.
