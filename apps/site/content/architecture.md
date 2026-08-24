# Architecture

VoxNexus is a **modular monolith**: one Rust binary (`voxnexus`) serves HTTP, the WebSocket gateway, background jobs, and (optionally) the built SPA. The browser app is React + TypeScript in `apps/web`.

Message fanout and live voice are not available yet. Categories, channels, roles, and permissions are.

## Hierarchy

```text
Instance → Community → Space → Category → Channel
```

| Layer | Role |
|---|---|
| **Instance** | One deployment. Registration mode, community-creation policy, OIDC settings. |
| **Community** | Discord-style server: settings, members, invites, cosmetics, roles. |
| **Space** | Guilded-style group inside a community (not nested). Open or restricted membership. |
| **Category** | Ordered channel group; optional `space_id`. |
| **Channel** | `text` / `voice` / `forum` container with topic, position, archive, and permission overrides. |

## Process layout

```text
Browser (apps/web)
    │  cookie session + CSRF
    ▼
Axum (crates/server)
    ├── /api/v1/*          REST (utoipa → OpenAPI)
    ├── /api/v1/gateway    WebSocket
    ├── /health /ready [/metrics]
    └── WEB_DIST SPA       optional static files
         │
         ├── crates/auth, domain, protocol, permissions
         ├── PostgreSQL (SQLx migrations)
         ├── Redis (Apalis jobs)
         ├── SeaweedFS S3 (avatars, banners, icons, role icons)
         └── Typesense (collections ready; indexing later)
```

## Crate map

| Crate | Responsibility |
|---|---|
| `server` | Composition root, routes, middleware, OpenAPI |
| `config` | File + env loading; fail-fast validation |
| `db` | Pool + migrate |
| `domain` | IDs, enums, pure types (no I/O) |
| `protocol` | HTTP DTOs + gateway envelopes (schemars) |
| `auth` | Passwords, sessions, OIDC RP, community/space/channel persistence helpers |
| `permissions` | Permission codes, grant merge, override layers, explain traces |
| `realtime` | Gateway session loop, resume buffer, presence hub |
| `storage` | S3 object store trait + SeaweedFS client |
| `jobs` | Apalis workers on Redis |
| `search` | Typesense client + collection schemas |
| `media` | Image sniff/helpers for uploads |

Frontend packages: `@voxnexus/api-client`, `@voxnexus/protocol`, `@voxnexus/ui`.

## Authz

The permission engine in `crates/permissions` resolves allow/deny for channel view, channel manage, role manage, and related codes. Community **owner** always bypasses. Restricted Spaces deny non-members before role grants. Category and channel overrides refine grants. See [Permissions](/docs/guides/permissions).

Some write paths remain **owner-only** today (community settings/invites/cosmetics, Space CRUD and admin member add/remove). Channel and role management use `community.manage_channels` / `community.manage_roles`.

## Contracts

Rust is source of truth. After DTO or route changes: `pnpm codegen`. CI fails on drift (`pnpm check-codegen`). Live HTTP reference: [/docs/api](/docs/api).
