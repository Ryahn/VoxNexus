# OIDC / SSO

VoxNexus acts as an OIDC relying party (authorization code + PKCE).

**Two ways to set issuer / client ID** (pick one):

1. **Config** — set `OIDC_ISSUER` and `OIDC_CLIENT_ID`. On every startup they are copied into the instance row and appear under **Settings → Instance** (SSO enabled automatically).
   - Host `cargo run`: use repo-root `config.toml`.
   - Compose `app` container: use `deploy/docker/.env` (root `config.toml` is **not** mounted).
2. **UI only** — leave those config keys empty and set issuer / client ID under **Settings → Instance**.

`OIDC_CLIENT_SECRET` always lives in config/env only (never stored in the DB). SSO will not work until the secret is set, even if the UI fields are filled.

## Redirect URI

Register this redirect URL with your IdP:

```text
{PUBLIC_URL}/api/v1/auth/oidc/callback
```

Example: `http://127.0.0.1:8080/api/v1/auth/oidc/callback`

## Environment

| Key | Purpose |
|-----|---------|
| `OIDC_ISSUER` | Provider issuer URL (discovery) |
| `OIDC_CLIENT_ID` | OAuth client ID |
| `OIDC_CLIENT_SECRET` | Client secret (never stored in DB) |
| `OIDC_ONLY` | When `true`, hide password login/register |
| `OIDC_LINK_BY_EMAIL` | Link OIDC logins to existing accounts by verified email (default `true`) |

When `OIDC_ISSUER` is unset, the instance row (UI) is the source of truth for enable / issuer / client ID.

## Authentik (local test stack)

**Happy path:** run Postgres/Redis/SeaweedFS/Typesense **and** Authentik in Compose; run VoxNexus on the host (`cargo run` + Vite). Both the browser and the server then reach Authentik at `http://127.0.0.1:9000`.

**Compose `app` container:** set `OIDC_ISSUER=http://host.docker.internal:9000/application/o/<slug>/` so the container can reach Authentik. `/api/v1/auth/oidc/start` responds with a **303** to that host — open it in Chrome/Edge. Cursor’s simple browser often shows a blank page because it does not follow that redirect. `127.0.0.1` as issuer fails inside the container (loopback is the container itself).

### 1. Start the stack

Copy `deploy/docker/.env.example` → `deploy/docker/.env` if needed, set a real `AUTHENTIK_SECRET_KEY` (50+ random chars), then from the repo root:

```bash
docker compose -f deploy/docker/docker-compose.yml -f deploy/docker/compose.authentik.yml --env-file deploy/docker/.env up -d
```

The overlay creates an `authentik` database on the shared Postgres (idempotent), then starts Authentik server (port **9000**) and worker.

### 2. Bootstrap admin

Open `http://127.0.0.1:9000`. Sign in with `AUTHENTIK_BOOTSTRAP_EMAIL` / `AUTHENTIK_BOOTSTRAP_PASSWORD` from `.env` (defaults in `.env.example`).

### 3. Create an OAuth2/OpenID Provider

In Authentik admin:

1. **Applications → Providers → Create** → **OAuth2/OpenID Provider**
2. **Redirect URIs:** `http://127.0.0.1:8080/api/v1/auth/oidc/callback` (or `{PUBLIC_URL}/api/v1/auth/oidc/callback`)
3. **Client type:** Confidential
4. **Scopes:** include `openid` and `email` (add `profile` if you want display name claims)
5. Save and note the issuer URL (typically `http://127.0.0.1:9000/application/o/<slug>/`)

### 4. Create an Application

**Applications → Applications → Create**, bind it to that provider, pick a slug (e.g. `voxnexus`). Copy **Client ID** and **Client Secret**.

### 5. Point VoxNexus at Authentik

In `config.toml` (host-run) or Compose `.env` (overlay passes these into `app`):

```toml
OIDC_ISSUER = "http://127.0.0.1:9000/application/o/voxnexus/"
OIDC_CLIENT_ID = "<client-id>"
OIDC_CLIENT_SECRET = "<client-secret>"
# OIDC_ONLY = "false"
# OIDC_LINK_BY_EMAIL = "true"
```

Enable OIDC in **Settings → Instance** (`oidc_enabled`, issuer, client ID). Issuer/client ID can also seed from env on first boot; the secret is always config-only.

Confirm discovery works:

```text
http://127.0.0.1:9000/application/o/voxnexus/.well-known/openid-configuration
```

Include **groups** in the ID token if you plan to map IdP groups to community roles later.

## HTTP

| Route | Description |
|-------|-------------|
| `GET /api/v1/auth/oidc/start` | Redirect to IdP |
| `GET /api/v1/auth/oidc/callback` | Code exchange, session cookie, redirect home |

`GET /api/v1/meta` exposes `oidc_enabled` and `password_login_enabled` for the login UI.

## Account linking

1. Existing `(issuer, subject)` → sign in.
2. Verified email + `OIDC_LINK_BY_EMAIL` → link identity to local account.
3. Registration open → JIT account (no instance admin; no extra permissions).

Identity rows live in `auth_identities` with unique `(issuer, subject)`.
