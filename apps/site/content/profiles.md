# Profiles and presence

## Profiles

Every account has a profile row (created on register / OIDC provision).

| Field | Notes |
|---|---|
| `display_name` | Shown in UI |
| `bio` | Short text |
| `custom_status` | Free-text status line |
| `presence_status` | Stored preference: `online` \| `idle` \| `dnd` \| `invisible` |
| Avatar / banner | Stored in SeaweedFS; served via authenticated GET |

### HTTP

| Method | Path |
|---|---|
| `GET` / `PATCH` | `/api/v1/me/profile` |
| `PUT` | `/api/v1/me/profile/avatar` · `/banner` |
| `GET` | `/api/v1/profiles/{account_id}` |
| `GET` | `/api/v1/profiles/{account_id}/avatar` · `/banner` |

All require a session. Image types: JPEG, PNG, GIF, WebP (size limits enforced in handlers/UI).

## Presence

Live presence is tracked in-memory by the gateway presence hub; profile stores the last chosen status / custom text.

| Method | Path | Notes |
|---|---|---|
| `GET` | `/api/v1/presence` | Snapshot for the signed-in user |

Gateway (after `IDENTIFY`):

- `PRESENCE_SYNC` — initial set for the connection
- `PRESENCE_UPDATE` — fanout when someone changes status
- Client may send `STATUS_UPDATE` to change own status

Public view maps `invisible` to offline-like behavior for other users. See [Gateway](/docs/guides/gateway).

## Web UI

Settings → Profile for display name, bio, images, and presence. The shell subscribes to the gateway for live updates.
