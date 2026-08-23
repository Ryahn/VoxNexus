# Authentication

Feature Tasks F011–F012. Local email/password accounts, Postgres sessions, and browser cookies.

## Schema

Migration `20260822140000_accounts`:

| Table | Purpose |
|---|---|
| `accounts` | UUIDv7 id, nullable unique email, nullable `password_hash`, `is_bot`, `is_instance_admin`, timestamps |
| `auth_identities` | OIDC (and later) links: unique `(issuer, subject)` |
| `sessions` | Hashed session secret, sliding expiry (30 days), optional user-agent / IP |

Domain types live in `voxnexus-domain`. Password hashing (Argon2id) and session helpers live in `voxnexus-auth`.

Until F017, every account uses a fixed `DEFAULT_INSTANCE_ID`. Registration is gated by `REGISTRATION_OPEN` (default **true**).

## HTTP

| Method | Path | Notes |
|---|---|---|
| `POST` | `/api/v1/auth/register` | Creates account + session cookie |
| `POST` | `/api/v1/auth/login` | Timing-safe verify; session cookie |
| `POST` | `/api/v1/auth/logout` | Deletes session; clears cookie |
| `GET` | `/api/v1/auth/me` | Current account or `401` |

Cookie name: `vn_session` when `COOKIE_SECURE=false`, `__Host-vn_session` when secure (Secure, Path=/, HttpOnly, SameSite=Lax).

Mutating `/api` requests require `Origin` or `Referer` matching `PUBLIC_URL`. When `COOKIE_SECURE=false`, localhost / `127.0.0.1` (any port) is also accepted so the Vite proxy works.

## Web UI

`/login` and `/register` pages in `apps/web` call the generated `@voxnexus/api-client` with `credentials: 'include'`.
