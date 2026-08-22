/* eslint-disable */
/**
 * This file was automatically generated from gateway.schema.json.
 * DO NOT MODIFY IT BY HAND. Run `pnpm codegen` instead.
 */

/**
 * Closed set of gateway `event_type` values for F007 (+ later features append here).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "EventType".
 */
export type EventType = 'HELLO' | 'HEARTBEAT' | 'HEARTBEAT_ACK' | 'DEV_PING' | 'DEV_PONG';

/**
 * Schema catalog for TypeScript codegen (`schemars` → `packages/protocol`).
 */
export interface GatewaySchemaCatalog {
  dev_ping_payload: DevPingPayload;
  dev_pong_payload: DevPongPayload;
  envelope: Envelope;
  event_scope: EventScope;
  event_type: EventType;
  heartbeat_ack_payload: HeartbeatAckPayload;
  heartbeat_payload: HeartbeatPayload;
  hello_payload: HelloPayload;
  [k: string]: unknown;
}
/**
 * Dev-only unauthenticated ping (requires `GATEWAY_ALLOW_UNAUTH`).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "DevPingPayload".
 */
export interface DevPingPayload {
  nonce: string;
  [k: string]: unknown;
}
/**
 * Reply to [`DevPingPayload`].
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "DevPongPayload".
 */
export interface DevPongPayload {
  nonce: string;
  [k: string]: unknown;
}
/**
 * Versioned JSON envelope on the gateway WebSocket.
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "Envelope".
 */
export interface Envelope {
  event_id: string;
  event_type: EventType;
  payload: unknown;
  scope?: EventScope | null;
  sequence: number;
  timestamp: string;
  [k: string]: unknown;
}
/**
 * Subscription / fanout scope (connection-level events omit this).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "EventScope".
 */
export interface EventScope {
  id: string;
  type: string;
  [k: string]: unknown;
}
/**
 * Server → client heartbeat acknowledgment.
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "HeartbeatAckPayload".
 */
export interface HeartbeatAckPayload {
  [k: string]: unknown;
}
/**
 * Client → server heartbeat (empty object).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "HeartbeatPayload".
 */
export interface HeartbeatPayload {
  [k: string]: unknown;
}
/**
 * `HELLO` payload after WebSocket accept.
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "HelloPayload".
 */
export interface HelloPayload {
  heartbeat_interval_ms: number;
  protocol_version: number;
  session_id: string;
  [k: string]: unknown;
}
