# Codegen

Feature Tasks F006–F007. Rust is the source of truth for HTTP and gateway contracts. Committed TypeScript artifacts must match.

## Generate

```powershell
pnpm codegen
```

That runs:

1. `export-openapi` — writes [`packages/api-client/openapi.json`](../packages/api-client/openapi.json) from utoipa.
2. `export-events` — writes [`packages/protocol/gateway.schema.json`](../packages/protocol/gateway.schema.json) from schemars.
3. `@hey-api/openapi-ts` — fetch client under `packages/api-client/src/`.
4. `json-schema-to-typescript` — gateway types under `packages/protocol/src/generated/`.

Do not hand-edit generated files. Change Rust handlers/DTOs, then regenerate.

## Drift check

```powershell
pnpm check-codegen
```

Regenerates and fails if `git diff` is non-empty under `packages/api-client` or the committed gateway schema/generated types.

The web app imports `@voxnexus/api-client` (`getMeta`) and `@voxnexus/protocol` (`createGatewayClient`). Vite proxies `/api` (including WebSocket) to `http://127.0.0.1:8080`.
