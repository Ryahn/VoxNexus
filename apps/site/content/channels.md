# Categories & channels

Categories group channels. Channels are typed containers (`text`, `voice`, `forum`). Message and voice sessions are not available yet; structure, ordering, archive, and authz are.

## Categories

Optional `space_id` scopes a category to a Space. Omit for community-wide categories.

| Method | Path | Who |
|---|---|---|
| `POST` | `/api/v1/communities/{id}/categories` | `community.manage_channels` |
| `GET` | `/api/v1/communities/{id}/categories` | Member |
| `PATCH` | `/api/v1/categories/{category_id}` | `community.manage_channels` |
| `DELETE` | `/api/v1/categories/{category_id}` | `community.manage_channels` |
| `POST` | `/api/v1/communities/{id}/categories/reorder` | `community.manage_channels`; body `{ "category_ids": […] }` |

## Channels

| Field | Notes |
|---|---|
| `type` | `text` \| `voice` \| `forum` |
| `name` | Required, ≤100 |
| `topic` | Optional, ≤500 |
| `space_id` / `category_id` | Optional scope; omit `category_id` for a loose channel in the Space (or community root) |
| `position` | Ordering within the list scope |
| `archived_at` | Set when archived; hidden from default lists |
| `config` | JSON bag for type-specific options |

Categories are optional grouping. Channels do **not** require a category.

### HTTP

| Method | Path | Who |
|---|---|---|
| `POST` | `/api/v1/communities/{id}/channels` | `community.manage_channels` |
| `GET` | `/api/v1/communities/{id}/channels` | Member; filtered to channels the caller may `text.view`. Query: `space_id`, `category_id`, `include_archived` |
| `GET` | `/api/v1/channels/{channel_id}` | `text.view` (404 if hidden) |
| `PATCH` | `/api/v1/channels/{channel_id}` | `community.manage_channels` |
| `DELETE` | `/api/v1/channels/{channel_id}` | `community.manage_channels` |
| `POST` | `/api/v1/communities/{id}/channels/reorder` | `community.manage_channels`; body `{ "channel_ids": […] }` |
| `POST` | `/api/v1/channels/{channel_id}/archive` | `community.manage_channels` |
| `POST` | `/api/v1/channels/{channel_id}/restore` | `community.manage_channels` |
| `POST` | `/api/v1/channels/{channel_id}/clone` | `community.manage_channels` |

Community **owner** always satisfies manage/view checks. See [Permissions](/docs/guides/permissions) for overrides and Explain Access.

## Web UI

Channel sidebar lists uncategorized channels first, then categories. Create a channel from the Space header (`+`) without a category, or from a category’s `+`. Categories remain optional grouping.
