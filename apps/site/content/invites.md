# Invites

Invite codes let people join a community without an open join mode.

## Model

| Field | Meaning |
|---|---|
| `code` | Unguessable public code |
| `max_uses` | Optional cap (`1…1000`); omit for unlimited |
| `uses` | Successful accepts so far |
| `expires_at` | Optional absolute expiry (from `expire_after` on create) |
| `paused` | When true, accepts fail until unpaused |
| `revoked_at` | Soft-revoked; preview/accept treat as missing |

Create body may set `expire_after: { unit: "hours" \| "days" \| "months", value }` instead of a raw timestamp.

## HTTP

| Method | Path | Who |
|---|---|---|
| `POST` | `/api/v1/communities/{id}/invites` | Owner |
| `GET` | `/api/v1/communities/{id}/invites` | Owner |
| `PATCH` | `/api/v1/communities/{id}/invites/{invite_id}` | Owner (pause / unpause) |
| `DELETE` | `/api/v1/communities/{id}/invites/{invite_id}` | Owner (revoke) |
| `GET` | `/api/v1/invites/{code}` | Session; preview (no join) |
| `POST` | `/api/v1/invites/{code}/accept` | Session; joins as `member` |

Accept rejects paused, expired, exhausted, already-a-member, and application-mode communities.

## Web UI

Invite manager on community settings; join modal accepts either a community UUID (open join) or an invite code.
