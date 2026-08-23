# Spaces

A Space is a flat group inside a community (Guilded-style). Spaces are not nested. Channels and Space membership are not available yet.

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

`restricted` is stored for later access control. Space membership is not enforced yet; for now, community members can list/read Spaces, and only the **owner** can create/update/delete.

## HTTP

| Method | Path | Who |
|---|---|---|
| `POST` | `/api/v1/communities/{community_id}/spaces` | Owner |
| `GET` | `/api/v1/communities/{community_id}/spaces` | Member |
| `GET` | `/api/v1/spaces/{space_id}` | Member of parent community |
| `PATCH` | `/api/v1/spaces/{space_id}` | Owner |
| `DELETE` | `/api/v1/spaces/{space_id}` | Owner |

## Web UI

Space list/switcher in the community sidebar and a create-Space modal. Empty Space views are placeholders until categories and channels exist.
