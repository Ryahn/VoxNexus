//! HTTP DTOs and gateway event types.

mod auth;
mod error;
mod gateway;
mod meta;
mod pagination;

pub use auth::{AccountResponse, AuthSessionResponse, LoginRequest, RegisterRequest};
pub use error::{error_codes, ErrorBody};
pub use gateway::{
    gateway_schema, gateway_schema_json, DevPingPayload, DevPongPayload, Envelope, EventScope,
    EventType, GatewaySchemaCatalog, HeartbeatAckPayload, HeartbeatPayload, HelloPayload,
    DEFAULT_HEARTBEAT_INTERVAL_MS, GATEWAY_PROTOCOL_VERSION, GATEWAY_SUBPROTOCOL,
};
pub use meta::MetaResponse;
pub use pagination::{CursorPage, CursorQuery, DEFAULT_PAGE_LIMIT, MAX_PAGE_LIMIT};
