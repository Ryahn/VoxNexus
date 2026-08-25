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
  | 'PRESENCE_SYNC'
  | 'MEMBER_JOIN'
  | 'MEMBER_LEAVE'
  | 'ROLE_CREATE'
  | 'ROLE_UPDATE'
  | 'ROLE_DELETE'
  | 'MEMBER_ROLE_UPDATE'
  | 'MESSAGE_CREATE'
  | 'MESSAGE_UPDATE'
  | 'MESSAGE_DELETE';
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
  community_role_payload: CommunityRolePayload;
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
  member_join_payload: MemberJoinPayload;
  member_leave_payload: MemberLeavePayload;
  member_role_update_payload: MemberRoleUpdatePayload;
  message_create_payload: MessageCreatePayload;
  message_delete_payload: MessageDeletePayload;
  message_update_payload: MessageUpdatePayload;
  presence_sync_payload: PresenceSyncPayload;
  presence_update_payload: PresenceUpdatePayload;
  ready_payload: ReadyPayload;
  resume_payload: ResumePayload;
  resumed_payload: ResumedPayload;
  role_delete_payload: RoleDeletePayload;
  status_update_payload: StatusUpdatePayload;
  [k: string]: unknown;
}
/**
 * Server → client when a role is created (F028).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "CommunityRolePayload".
 */
export interface CommunityRolePayload {
  color: string;
  community_id: string;
  gradient?: string | null;
  group_id?: string | null;
  hoist: boolean;
  icon_emoji?: string | null;
  icon_object_key?: string | null;
  id: string;
  is_everyone: boolean;
  mentionable: boolean;
  name: string;
  permissions: unknown;
  position: number;
  role_card: unknown;
  short_tag: string;
  weight: number;
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
 * Server → client when an account joins a community (F020).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "MemberJoinPayload".
 */
export interface MemberJoinPayload {
  account_id: string;
  community_id: string;
  display_name: string;
  nickname: string;
  role: string;
  [k: string]: unknown;
}
/**
 * Server → client when an account leaves a community (F020).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "MemberLeavePayload".
 */
export interface MemberLeavePayload {
  account_id: string;
  community_id: string;
  [k: string]: unknown;
}
/**
 * Server → client when a member's role set changes (F028).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "MemberRoleUpdatePayload".
 */
export interface MemberRoleUpdatePayload {
  account_id: string;
  community_id: string;
  role_ids: string[];
  [k: string]: unknown;
}
/**
 * Server → client when a text message is created (F035).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "MessageCreatePayload".
 */
export interface MessageCreatePayload {
  attachments?: AttachmentResponse[];
  author_display_name: string;
  author_id: string;
  channel_id: string;
  community_id: string;
  content: string;
  created_at: string;
  edited_at?: string | null;
  id: string;
  nonce?: string | null;
  referenced_message_id?: string | null;
  reply_to?: MessageReplyPreview | null;
  [k: string]: unknown;
}
/**
 * Attachment metadata on a message (or pending upload).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "AttachmentResponse".
 */
export interface AttachmentResponse {
  byte_size: number;
  content_type: string;
  filename: string;
  height?: number | null;
  id: string;
  thumbnail_url?: string | null;
  url: string;
  width?: number | null;
  [k: string]: unknown;
}
/**
 * Preview of the message being replied to.
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "MessageReplyPreview".
 */
export interface MessageReplyPreview {
  author_display_name: string;
  author_id: string;
  deleted: boolean;
  excerpt: string;
  message_id: string;
  [k: string]: unknown;
}
/**
 * Server → client when a text message is deleted (F036).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "MessageDeletePayload".
 */
export interface MessageDeletePayload {
  channel_id: string;
  community_id: string;
  id: string;
  [k: string]: unknown;
}
/**
 * Server → client when a text message is edited (F036).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "MessageUpdatePayload".
 */
export interface MessageUpdatePayload {
  attachments?: AttachmentResponse[];
  author_display_name: string;
  author_id: string;
  channel_id: string;
  community_id: string;
  content: string;
  created_at: string;
  edited_at?: string | null;
  id: string;
  nonce?: string | null;
  referenced_message_id?: string | null;
  reply_to?: MessageReplyPreview | null;
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
 * Server → client when a role is deleted (F028).
 *
 * This interface was referenced by `GatewaySchemaCatalog`'s JSON-Schema
 * via the `definition` "RoleDeletePayload".
 */
export interface RoleDeletePayload {
  community_id: string;
  role_id: string;
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
