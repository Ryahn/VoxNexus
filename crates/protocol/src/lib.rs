//! HTTP DTOs and gateway event types.

mod auth;
mod error;
mod gateway;
mod instance;
mod meta;
mod pagination;
mod profile;

pub use auth::{
    AccountResponse, AuthSessionResponse, ChangeEmailRequest, ChangePasswordRequest, LoginRequest,
    RegisterRequest,
};
pub use error::{error_codes, ErrorBody};
pub use gateway::{
    gateway_schema, gateway_schema_json, DevPingPayload, DevPongPayload, Envelope, EventScope,
    EventType, GatewaySchemaCatalog, HeartbeatAckPayload, HeartbeatPayload, HelloPayload,
    IdentifyPayload, InvalidSessionPayload, PresenceSyncPayload, PresenceUpdatePayload,
    ReadyPayload, ResumePayload, ResumedPayload, StatusUpdatePayload,
    DEFAULT_HEARTBEAT_INTERVAL_MS, GATEWAY_PROTOCOL_VERSION, GATEWAY_SUBPROTOCOL,
};
pub use instance::{
    CommunityCreateAcceptedResponse, InstanceSettingsResponse, UpdateInstanceSettingsRequest,
};
pub use meta::MetaResponse;
pub use pagination::{CursorPage, CursorQuery, DEFAULT_PAGE_LIMIT, MAX_PAGE_LIMIT};
pub use profile::{PresenceEntry, PresenceListResponse, ProfileResponse, UpdateProfileRequest};
