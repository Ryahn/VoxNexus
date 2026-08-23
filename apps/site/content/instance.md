# Instance settings

One VoxNexus process = one instance. Settings live in Postgres and are readable by any signed-in user; updates require an **instance admin**.

## Fields

| Field | Meaning |
|---|---|
| `name` | Instance display name |
| `public_url` | Canonical origin (CORS / links); should match `PUBLIC_URL` |
| `registration_mode` | `open` \| `invite` \| `closed` |
| `community_creation_mode` | `open` \| `admin_only` \| `single` |
| `community_creation_mode_locked` | When true, API ignores mode patches; config may force-sync on startup |
| OIDC fields | `oidc_enabled`, `oidc_issuer`, `oidc_client_id` (secret stays in config only) |

## Bootstrap

On first start, optional `BOOTSTRAP_ADMIN_EMAIL` / `BOOTSTRAP_ADMIN_PASSWORD` create an instance admin. With `community_creation_mode = single`, `BOOTSTRAP_COMMUNITY_NAME` can create the sole community.

`REGISTRATION_OPEN` in config still influences early behavior; prefer instance `registration_mode` once the row exists.

## HTTP

| Method | Path | Who |
|---|---|---|
| `GET` | `/api/v1/instance/settings` | Session |
| `PATCH` | `/api/v1/instance/settings` | Instance admin |

`GET /api/v1/meta` stays public (name, version, and login-mode hints for the SPA).

## Web UI

Settings → Instance for admins. OIDC operator detail: [OIDC](/docs/guides/oidc).
