# Gateway

Feature Task F007. Realtime chat does **not** use this socket until F035. Until F013, production-style configs refuse the gateway.

## Endpoint

`GET /api/v1/gateway` — WebSocket upgrade. Subprotocol: `voxnexus.gateway.v1`.

## Config

| Key | Default | Meaning |
|---|---|---|
| `GATEWAY_ALLOW_UNAUTH` | `false` | When `false`, the upgrade returns **503** `gateway_unavailable` (auth arrives in F013). Set `true` only for local protocol work. Enables the unauthenticated `DEV_PING` / `DEV_PONG` events. |

## Lifecycle (F007)

1. Client connects with subprotocol `voxnexus.gateway.v1`.
2. Server sends `HELLO` `{ heartbeat_interval_ms, protocol_version, session_id }`.
3. Client must send `HEARTBEAT` at least every `heartbeat_interval_ms`; server replies `HEARTBEAT_ACK`.
4. Missed heartbeats for `2 × interval` close the socket.
5. Optional (unauth only): client `DEV_PING` `{ nonce }` → server `DEV_PONG` `{ nonce }`.

Envelope shape (all events):

```json
{
  "event_id": "018f...",
  "sequence": 1,
  "event_type": "HELLO",
  "timestamp": "2026-08-21T00:00:00Z",
  "payload": {}
}
```

`IDENTIFY` / `READY` / chat events land in later Feature Tasks.

## Types and client

Rust types live in `crates/protocol` (schemars). Committed schema: `packages/protocol/gateway.schema.json`. Generated TS + `createGatewayClient` stub: `@voxnexus/protocol`.

```powershell
pnpm codegen
```

Web debug stub: set `VITE_GATEWAY_DEBUG=true` and run the SPA with the API listening and `GATEWAY_ALLOW_UNAUTH=true`.
