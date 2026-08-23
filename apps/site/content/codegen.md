# Codegen

Rust is the source of truth for HTTP and gateway contracts. Committed TypeScript artifacts must match.

## Generate

```powershell
pnpm codegen
```

That runs:

1. `export-openapi` — writes `packages/api-client/openapi.json` from utoipa
2. `export-events` — writes `packages/protocol/gateway.schema.json` from schemars
3. OpenAPI → `@voxnexus/api-client`
4. JSON Schema → `@voxnexus/protocol` generated types

Do not hand-edit generated files. Change Rust handlers/DTOs, then regenerate.

## Drift check

```powershell
pnpm check-codegen
```

Regenerates and fails if `git diff` is non-empty under the generated packages.

`apps/web` imports `@voxnexus/api-client` and `@voxnexus/protocol`. Vite proxies `/api` (including WebSocket) to `http://127.0.0.1:8080`.
