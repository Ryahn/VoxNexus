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

## Messages (text channels)

Members with `text.view` can list messages; `text.send` (alias `message.send`) is required to post. Hidden channels (no view) return **404** on list/send. View without send returns **403** on POST.

| Method | Path | Notes |
|---|---|---|
| `POST` | `/api/v1/channels/{channel_id}/messages` | Body `{ "content", "nonce"?, "referenced_message_id"?, "attachment_ids"? }`. Optional `Idempotency-Key` header. Content 0–4000 chars (empty allowed with attachments). Idempotent nonce → 200 with existing row. Reply target must be in the same channel (else 400). |
| `GET` | `/api/v1/channels/{channel_id}/messages` | Newest first. Query: `before`, `after`, `limit` (default 50, max 100). Each item may include `reply_to`, `attachments`, and `mentions`. |
| `POST` | `/api/v1/channels/{channel_id}/attachments` | Raw body upload. Requires `text.attach`. Header `X-Filename`. Images/PDF/text allowed (≤5 MiB). Executables rejected. Returns attachment metadata; enqueue thumbnail job for images. |
| `GET` | `/api/v1/attachments/{attachment_id}` | Requires `text.view` on the attachment’s channel (404 if hidden). Query `thumb=1` for thumbnail when present. |
| `PATCH` | `/api/v1/channels/{channel_id}/messages/{message_id}` | Author only. Body `{ "content" }`. Sets `edited_at`. Emits `MESSAGE_UPDATE`. |
| `DELETE` | `/api/v1/channels/{channel_id}/messages/{message_id}` | Author or `text.manage_messages`. Soft-delete. Emits `MESSAGE_DELETE`. Parent replies keep a deleted preview. |

Content may include structured mentions: `@{account_id}`, `@&{role_id}`, `@everyone`, `@here`. `@everyone`/`@here` require `community.mention_everyone` (rejected with 400 otherwise). Role mentions require `text.mention_roles`; non-mentionable roles need `community.manage_roles` or `community.mention_everyone`. Parsed mentions are stored for inbox (F043) and returned on message payloads.

`@everyone` defaults to view + send + attach + mention_roles. Live clients receive `MESSAGE_CREATE` over the gateway (see [Gateway](/docs/guides/gateway)); HTTP list remains the history source of truth.

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

Channel sidebar lists uncategorized channels first, then categories. Create a channel from the Space header (`+`) without a category, or from a category’s `+`. Categories remain optional grouping. Live text channels show a message transcript and composer; new messages arrive over the gateway. Reply from a message’s action bar; the composer shows a parent preview and clickable jump on replies. Attach files via the paperclip control, drag-drop, or paste — images render inline after send.
