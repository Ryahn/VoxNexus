# Authentication

Local email/password accounts, Postgres sessions, and cookie-based API auth. OIDC: [OIDC](/docs/guides/oidc).

## Model

| Table | Purpose |
|---|---|
| `accounts` | UUIDv7 id, email, password hash (nullable for OIDC-only), flags |
| `auth_identities` | External IdP links `(issuer, subject)` |
| `sessions` | Hashed session secret, sliding expiry (~30 days) |

Passwords: Argon2id in `crates/auth`. Accounts use the single default instance id until multi-instance work exists.

## HTTP

| Method | Path | Notes |
|---|---|---|
| `POST` | `/api/v1/auth/register` | Account + session cookie when registration allows |
| `POST` | `/api/v1/auth/login` | Timing-safe verify; session cookie |
| `POST` | `/api/v1/auth/logout` | Deletes session; clears cookie |
| `GET` | `/api/v1/auth/me` | Current account or `401` |
| `POST` | `/api/v1/auth/me/password` | Requires current password |
| `PATCH` | `/api/v1/auth/me/email` | Immediate until SMTP verification exists |

Cookie: `vn_session` when `COOKIE_SECURE=false`, `__Host-vn_session` when secure (HttpOnly, Path=/, SameSite=Lax; Secure when enabled).

Mutating `/api` calls need `Origin` or `Referer` matching `PUBLIC_URL`. With `COOKIE_SECURE=false`, localhost / `127.0.0.1` (any port) is also accepted for the Vite proxy.

Registration follows instance `registration_mode` / config `REGISTRATION_OPEN` — [Instance](/docs/guides/instance).

## Web UI

`/login` and `/register` in `apps/web` use `@voxnexus/api-client` with `credentials: 'include'`. The gateway reuses the same cookie after upgrade.
