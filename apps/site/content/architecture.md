# Architecture

VoxNexus is a **modular monolith**: one Rust binary (`voxnexus`) serves HTTP, the WebSocket gateway, background jobs, and (optionally) the built SPA. The browser app is React + TypeScript in `apps/web`.

Channels, messaging, roles, and voice are not available yet.

## Hierarchy

```text
Instance → Community → Space → (Category → Channel → … later)
```

| Layer | Role today |
|---|---|
| **Instance** | One deployment. Registration mode, community-creation policy, OIDC settings. |
| **Community** | Discord-style server: settings, members, invites, icon/banner. |
| **Space** | Guilded-style group inside a community: name, topic/game, visibility. Not nested. Space membership is not available yet. |

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
         ├── crates/auth, domain, protocol
         ├── PostgreSQL (SQLx migrations)
         ├── Redis (Apalis jobs)
         ├── SeaweedFS S3 (avatars, banners, icons)
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
| `auth` | Passwords, sessions, OIDC RP, community/space persistence helpers |
| `realtime` | Gateway session loop, resume buffer, presence hub |
| `storage` | S3 object store trait + SeaweedFS client |
| `jobs` | Apalis workers on Redis |
| `search` | Typesense client + collection schemas |
| `permissions` | Stub (permission engine not available yet) |
| `media` | Image sniff/helpers for uploads |

Frontend packages: `@voxnexus/api-client`, `@voxnexus/protocol`, `@voxnexus/ui`.

## Authz today

Until a full permission engine exists, write access for community settings, invites, and Spaces is **community owner only**. Members can read (where allowed), join/leave, and manage their own nickname/profile/presence.

## Contracts

Rust is source of truth. After DTO or route changes: `pnpm codegen`. CI fails on drift (`pnpm check-codegen`). Live HTTP reference: [/docs/api](/docs/api).
