# Gateway

Authenticated WebSocket for presence, community events, and text-channel message fanout.

## Endpoint

`GET /api/v1/gateway` — WebSocket upgrade. Subprotocol: `voxnexus.gateway.v1`.

Requires a valid session cookie. Set `GATEWAY_ALLOW_UNAUTH=true` only for local protocol work; that also enables `DEV_PING` / `DEV_PONG` after identify.

## Lifecycle

1. Server → `HELLO` `{ heartbeat_interval_ms, protocol_version, session_id }`
2. Client → `IDENTIFY` `{}` (account comes from the cookie on the handshake)
3. Server → `READY` `{ account_id, session_id, resume_token }`, then `PRESENCE_SYNC`
4. Client → `HEARTBEAT`; server → `HEARTBEAT_ACK`
5. Missing heartbeats for `2 × interval` closes the socket
6. Reconnect: `RESUME` `{ session_id, last_sequence, resume_token }` → replay missed fanout events from the in-memory ring (if still contiguous), then `RESUMED`, or `INVALID_SESSION`

Envelope:

```json
{
  "event_id": "018f...",
  "sequence": 1,
  "event_type": "HELLO",
  "timestamp": "2026-08-21T00:00:00Z",
  "payload": {}
}
```

`event_type` is `SCREAMING_SNAKE_CASE`. Community/channel fanout events include `scope: { "type": "community"|"channel", "id": "…" }`.

## Presence

- Client `STATUS_UPDATE` — `{ status?, custom_status? }`
- Server `PRESENCE_UPDATE` / `PRESENCE_SYNC` — see [Profiles](/docs/guides/profiles)

`MEMBER_JOIN` / `MEMBER_LEAVE` and role events fan out to online community members.

## Messages

On successful `POST …/messages`, the server emits `MESSAGE_CREATE` (channel scope) to **online** members who can `text.view` that channel. Hidden members do not receive the event. `MESSAGE_UPDATE` / `MESSAGE_DELETE` fan out on edit/delete. Create/update payloads include `referenced_message_id` and `reply_to` when the message is a reply, plus `attachments` and `mentions` when present.

Clients should HTTP-list history on open, then apply gateway creates. Resume replays buffered fanout envelopes while the ring still holds them (capacity 1000 per session).

## Types

Rust: `crates/protocol`. Schema: `packages/protocol/gateway.schema.json`. Client helper: `createGatewayClient` in `@voxnexus/protocol`. Regenerate with `pnpm codegen`.

SPA debug UI: `$env:VITE_GATEWAY_DEBUG="true"; pnpm dev` (API must be up; use `GATEWAY_ALLOW_UNAUTH=true` only for the unauth path).
