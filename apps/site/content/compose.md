# Development Docker Compose stack

One command brings up Postgres, Redis, SeaweedFS (S3), Typesense, and the `voxnexus` app (API + built SPA).

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
curl.exe -I http://127.0.0.1:8080/
```

Stop: `docker compose -f docker-compose.yml down`

## Layout

| Path | Role |
|---|---|
| [`docker-compose.yml`](/deploy/docker/docker-compose.yml) | Services + volumes |
| [`Dockerfile`](/deploy/docker/Dockerfile) | pnpm web + `cargo build --release` |
| [`.env.example`](/deploy/docker/.env.example) | Secrets / toggles |
| [`seaweedfs-s3.json`](/deploy/docker/seaweedfs-s3.json) | S3 IAM matching `S3_*` |
| [`compose.authentik.yml`](/deploy/docker/compose.authentik.yml) | Optional Authentik overlay — [OIDC](/docs/guides/oidc) |

## Ports (host)

| Port | Service |
|---|---|
| 8080 | App (API + SPA) |
| 5432 | Postgres |
| 6379 | Redis |
| 8333 | SeaweedFS S3 |
| 8108 | Typesense |

Inside the network, services use DNS names (`postgres`, `redis`, `seaweedfs`, `typesense`, `app`).

## App image

- Builds `@voxnexus/web` into the image web dist
- Sets `WEB_DIST` and listens on `0.0.0.0:8080`
- Runs migrations on startup
- Serves the SPA; unknown `/api/*` stays JSON 404

## Hybrid (Compose deps + cargo/pnpm on host)

Keep deps up, point `config.toml` at published ports with the same S3/Typesense keys as `.env.example`, then `cargo run -p voxnexus` and `pnpm dev`.
