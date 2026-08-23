/* eslint-disable */
/**
 * This file was automatically generated from gateway.schema.json.
 * DO NOT MODIFY IT BY HAND. Run `pnpm codegen` instead.
 */

/**
 * Closed set of gateway `event_type` values.
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "EventType".
 */
export type EventType =
  | 'HELLO'
  | 'HEARTBEAT'
  | 'HEARTBEAT_ACK'
  | 'IDENTIFY'
  | 'READY'
  | 'RESUME'
  | 'RESUMED'
  | 'INVALID_SESSION'
  | 'DEV_PING'
  | 'DEV_PONG'
  | 'STATUS_UPDATE'
  | 'PRESENCE_UPDATE'
  | 'PRESENCE_SYNC';
/**
 * Presence exposed to clients (offline when disconnected or hidden from viewers).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "PublicPresenceStatus".
 */
export type PublicPresenceStatus = 'online' | 'idle' | 'dnd' | 'invisible' | 'offline';
/**
 * Stored preference and gateway-reported status while connected.
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "PresenceStatus".
 */
export type PresenceStatus = 'online' | 'idle' | 'dnd' | 'invisible';

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
  identify_payload: IdentifyPayload;
  invalid_session_payload: InvalidSessionPayload;
  presence_sync_payload: PresenceSyncPayload;
  presence_update_payload: PresenceUpdatePayload;
  ready_payload: ReadyPayload;
  resume_payload: ResumePayload;
  resumed_payload: ResumedPayload;
  status_update_payload: StatusUpdatePayload;
  [k: string]: unknown;
}
/**
 * Dev-only ping (requires `GATEWAY_ALLOW_UNAUTH` after identify).
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
/**
 * Client → server identify (HTTP session cookie already bound on the handshake).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "IdentifyPayload".
 */
export interface IdentifyPayload {
  [k: string]: unknown;
}
/**
 * Server → client when resume cannot continue.
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "InvalidSessionPayload".
 */
export interface InvalidSessionPayload {
  resumable: boolean;
  [k: string]: unknown;
}
/**
 * Server → client bulk presence after identify.
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "PresenceSyncPayload".
 */
export interface PresenceSyncPayload {
  presences: PresenceUpdatePayload[];
  [k: string]: unknown;
}
/**
 * Server → client presence change for one account.
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "PresenceUpdatePayload".
 */
export interface PresenceUpdatePayload {
  account_id: string;
  custom_status: string;
  status: PublicPresenceStatus;
  [k: string]: unknown;
}
/**
 * Server → client ready after successful identify.
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "ReadyPayload".
 */
export interface ReadyPayload {
  account_id: string;
  resume_token: string;
  session_id: string;
  [k: string]: unknown;
}
/**
 * Client → server resume after reconnect.
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "ResumePayload".
 */
export interface ResumePayload {
  last_sequence: number;
  resume_token: string;
  session_id: string;
  [k: string]: unknown;
}
/**
 * Server → client after a successful resume (event buffer may still be empty in F013).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "ResumedPayload".
 */
export interface ResumedPayload {
  session_id: string;
  [k: string]: unknown;
}
/**
 * Client → server presence / custom status change (F018).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "StatusUpdatePayload".
 */
export interface StatusUpdatePayload {
  custom_status?: string | null;
  status?: PresenceStatus | null;
  [k: string]: unknown;
}
