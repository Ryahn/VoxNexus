//! Gateway envelope and closed event types (WebSocket protocol).

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Negotiated WebSocket subprotocol.
pub const GATEWAY_SUBPROTOCOL: &str = "voxnexus.gateway.v1";

/// Current gateway protocol version advertised in `HELLO`.
pub const GATEWAY_PROTOCOL_VERSION: u32 = 1;

/// Default client heartbeat interval in milliseconds.
pub const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 15_000;

/// Closed set of gateway `event_type` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    Hello,
    Heartbeat,
    HeartbeatAck,
    Identify,
    Ready,
    Resume,
    Resumed,
    InvalidSession,
    DevPing,
    DevPong,
}

/// Subscription / fanout scope (connection-level events omit this).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EventScope {
    #[serde(rename = "type")]
    pub scope_type: String,
    pub id: Uuid,
}

/// Versioned JSON envelope on the gateway WebSocket.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Envelope {
    pub event_id: Uuid,
    pub sequence: u64,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<EventScope>,
    pub payload: Value,
}

impl Envelope {
    /// Build an outbound envelope with a fresh UUIDv7 and UTC timestamp.
    #[must_use]
    pub fn new(sequence: u64, event_type: EventType, payload: impl Serialize) -> Self {
        Self {
            event_id: Uuid::now_v7(),
            sequence,
            event_type,
            timestamp: Utc::now(),
            scope: None,
            payload: serde_json::to_value(payload).unwrap_or(Value::Null),
        }
    }
}

/// `HELLO` payload after WebSocket accept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HelloPayload {
    pub heartbeat_interval_ms: u64,
    pub protocol_version: u32,
    pub session_id: Uuid,
}

/// Client → server heartbeat (empty object).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HeartbeatPayload {}

/// Server → client heartbeat acknowledgment.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HeartbeatAckPayload {}

/// Client → server identify (HTTP session cookie already bound on the handshake).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IdentifyPayload {}

/// Server → client ready after successful identify.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReadyPayload {
    pub account_id: Uuid,
    pub session_id: Uuid,
    pub resume_token: String,
}

/// Client → server resume after reconnect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ResumePayload {
    pub session_id: Uuid,
    pub last_sequence: u64,
    pub resume_token: String,
}

/// Server → client after a successful resume (event buffer may still be empty in F013).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ResumedPayload {
    pub session_id: Uuid,
}

/// Server → client when resume cannot continue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InvalidSessionPayload {
    pub resumable: bool,
}

/// Dev-only ping (requires `GATEWAY_ALLOW_UNAUTH` after identify).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DevPingPayload {
    pub nonce: String,
}

/// Reply to [`DevPingPayload`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DevPongPayload {
    pub nonce: String,
}

/// Schema catalog for TypeScript codegen (`schemars` → `packages/protocol`).
#[derive(Debug, Clone, JsonSchema)]
pub struct GatewaySchemaCatalog {
    pub envelope: Envelope,
    pub event_type: EventType,
    pub event_scope: EventScope,
    pub hello_payload: HelloPayload,
    pub heartbeat_payload: HeartbeatPayload,
    pub heartbeat_ack_payload: HeartbeatAckPayload,
    pub identify_payload: IdentifyPayload,
    pub ready_payload: ReadyPayload,
    pub resume_payload: ResumePayload,
    pub resumed_payload: ResumedPayload,
    pub invalid_session_payload: InvalidSessionPayload,
    pub dev_ping_payload: DevPingPayload,
    pub dev_pong_payload: DevPongPayload,
}

/// Root JSON Schema for gateway types (deterministic export).
#[must_use]
pub fn gateway_schema() -> schemars::schema::RootSchema {
    schemars::schema_for!(GatewaySchemaCatalog)
}

/// Pretty JSON Schema written to `packages/protocol/gateway.schema.json`.
///
/// # Panics
///
/// Panics if the schema cannot be serialized, which indicates a bug in schemars.
#[must_use]
pub fn gateway_schema_json() -> String {
    let mut json = serde_json::to_string_pretty(&gateway_schema()).expect("schema serializes");
    if !json.ends_with('\n') {
        json.push('\n');
    }
    json
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_round_trips() {
        let hello = HelloPayload {
            heartbeat_interval_ms: DEFAULT_HEARTBEAT_INTERVAL_MS,
            protocol_version: GATEWAY_PROTOCOL_VERSION,
            session_id: Uuid::now_v7(),
        };
        let outbound = Envelope::new(1, EventType::Hello, &hello);
        let json = serde_json::to_string(&outbound).expect("serialize");
        let parsed: Envelope = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.event_type, EventType::Hello);
        assert_eq!(parsed.sequence, 1);
        let payload: HelloPayload = serde_json::from_value(parsed.payload).expect("payload");
        assert_eq!(payload.protocol_version, GATEWAY_PROTOCOL_VERSION);
    }

    #[test]
    fn event_type_serializes_screaming_snake() {
        let json = serde_json::to_string(&EventType::HeartbeatAck).expect("ser");
        assert_eq!(json, "\"HEARTBEAT_ACK\"");
        let identify = serde_json::to_string(&EventType::Identify).expect("ser");
        assert_eq!(identify, "\"IDENTIFY\"");
    }

    #[test]
    fn schema_export_is_deterministic() {
        assert_eq!(gateway_schema_json(), gateway_schema_json());
    }

    #[test]
    fn committed_schema_matches() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/protocol/gateway.schema.json");
        let committed = std::fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("missing {}: {error}; run `pnpm codegen`", path.display())
        });
        let committed: serde_json::Value =
            serde_json::from_str(&committed).expect("committed schema");
        let generated: serde_json::Value =
            serde_json::from_str(&gateway_schema_json()).expect("generated schema");
        assert_eq!(
            generated, committed,
            "packages/protocol/gateway.schema.json is stale; run `pnpm codegen`"
        );
    }
}
