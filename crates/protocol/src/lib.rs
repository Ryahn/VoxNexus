//! HTTP DTOs and gateway event types.

mod auth;
mod category;
mod community;
mod error;
mod gateway;
mod instance;
mod meta;
mod pagination;
mod profile;
mod space;

pub use auth::{
    AccountResponse, AuthSessionResponse, ChangeEmailRequest, ChangePasswordRequest, LoginRequest,
    RegisterRequest,
};
pub use category::{
    CategoryListResponse, CategoryResponse, CreateCategoryRequest, ListCategoriesQuery,
    ReorderCategoriesRequest, UpdateCategoryRequest,
};
pub use community::{
    CommunityListResponse, CommunityMemberListResponse, CommunityMemberResponse, CommunityResponse,
    CreateCommunityRequest, CreateInviteRequest, DeleteCommunityRequest, InviteExpireAfter,
    InviteExpireUnit, InviteListResponse, InvitePreviewResponse, InviteResponse,
    TransferCommunityRequest, UpdateCommunityRequest, UpdateInviteRequest, UpdateNicknameRequest,
};
pub use error::{error_codes, ErrorBody};
pub use gateway::{
    gateway_schema, gateway_schema_json, DevPingPayload, DevPongPayload, Envelope, EventScope,
    EventType, GatewaySchemaCatalog, HeartbeatAckPayload, HeartbeatPayload, HelloPayload,
    IdentifyPayload, InvalidSessionPayload, MemberJoinPayload, MemberLeavePayload,
    PresenceSyncPayload, PresenceUpdatePayload, ReadyPayload, ResumePayload, ResumedPayload,
    StatusUpdatePayload, DEFAULT_HEARTBEAT_INTERVAL_MS, GATEWAY_PROTOCOL_VERSION,
    GATEWAY_SUBPROTOCOL,
};
pub use instance::{InstanceSettingsResponse, UpdateInstanceSettingsRequest};
pub use meta::MetaResponse;
pub use pagination::{CursorPage, CursorQuery, DEFAULT_PAGE_LIMIT, MAX_PAGE_LIMIT};
pub use profile::{PresenceEntry, PresenceListResponse, ProfileResponse, UpdateProfileRequest};
pub use space::{
    AddSpaceMemberRequest, CreateSpaceRequest, SpaceListResponse, SpaceMemberListResponse,
    SpaceMemberResponse, SpaceResponse, UpdateSpaceRequest,
};
