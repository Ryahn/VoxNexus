# Development Docker Compose stack

Feature Task F009. One command brings up Postgres, Redis, SeaweedFS (S3), Typesense, and the `voxnexus` app (Rust API + built SPA).

## Quick start

```powershell
cd deploy/docker
copy .env.example .env
docker compose -f docker-compose.yml up -d --build
```

Smoke:

```powershell
curl.exe http://127.0.0.1:8080/health
curl.exe http://127.0.0.1:8080/ready
curl.exe http://127.0.0.1:8080/api/v1/meta
# SPA (Axum serves WEB_DIST=/app/web)
curl.exe -I http://127.0.0.1:8080/
```

Stop:

```powershell
docker compose -f docker-compose.yml down
```
## Layout

| Path | Role |
|---|---|
| [`docker-compose.yml`](../deploy/docker/docker-compose.yml) | Services + volumes |
| [`Dockerfile`](../deploy/docker/Dockerfile) | Multi-stage: pnpm web + `cargo build --release` → debian-slim |
| [`.env.example`](../deploy/docker/.env.example) | Local secrets / toggles |
| [`seaweedfs-s3.json`](../deploy/docker/seaweedfs-s3.json) | S3 IAM matching Compose `S3_*` defaults |

## Network and ports

Services talk on the Compose network by DNS (`postgres`, `redis`, `seaweedfs`, `typesense`, `app`).

Published to the host (dev convenience):

| Port | Service |
|---|---|
| 8080 | App (API + SPA) |
| 5432 | Postgres |
| 6379 | Redis |
| 8333 | SeaweedFS S3 |
| 8108 | Typesense |

Prod-like overlays should drop host publishes for SeaweedFS (and optionally Redis/Postgres/Typesense) so only the app edge is exposed.

## App image

- Builds `@voxnexus/web` into `/app/web`
- Sets `WEB_DIST=/app/web` and `LISTEN_ADDR=0.0.0.0:8080`
- Axum serves the SPA; unknown `/api/*` paths stay JSON 404
- Migrations run on startup against Compose Postgres

## Local hybrid (Compose deps + cargo on host)

Keep Compose deps up, point `config.toml` at published ports (`5432`, `6379`, `8333`, `8108`) with the same S3/Typesense keys as `.env.example`, then `cargo run -p voxnexus` and `pnpm dev` as usual.
