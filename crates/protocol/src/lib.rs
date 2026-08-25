//! HTTP DTOs and gateway event types.

mod audit;
mod auth;
mod category;
mod channel;
mod community;
mod error;
mod explain;
mod gateway;
mod instance;
mod mention;
mod message;
mod meta;
mod pagination;
mod permission_override;
mod profile;
mod role;
mod space;
mod view_as;

pub use audit::{AuditEventListResponse, AuditEventResponse, ListAuditEventsQuery};
pub use auth::{
    AccountResponse, AuthSessionResponse, ChangeEmailRequest, ChangePasswordRequest, LoginRequest,
    RegisterRequest,
};
pub use category::{
    CategoryListResponse, CategoryResponse, CreateCategoryRequest, ListCategoriesQuery,
    ReorderCategoriesRequest, UpdateCategoryRequest,
};
pub use channel::{
    ChannelListResponse, ChannelResponse, CreateChannelRequest, ListChannelsQuery,
    ReorderChannelsRequest, UpdateChannelRequest,
};
pub use community::{
    CommunityListResponse, CommunityMemberListResponse, CommunityMemberResponse, CommunityResponse,
    CreateCommunityRequest, CreateInviteRequest, DeleteCommunityRequest, InviteExpireAfter,
    InviteExpireUnit, InviteListResponse, InvitePreviewResponse, InviteResponse,
    TransferCommunityRequest, UpdateCommunityRequest, UpdateInviteRequest, UpdateNicknameRequest,
};
pub use error::{error_codes, ErrorBody};
pub use explain::{ExplainPermissionRequest, ExplainPermissionResponse, PermissionExplainStep};
pub use gateway::{
    gateway_schema, gateway_schema_json, CommunityRolePayload, DevPingPayload, DevPongPayload,
    Envelope, EventScope, EventType, GatewaySchemaCatalog, HeartbeatAckPayload, HeartbeatPayload,
    HelloPayload, IdentifyPayload, InvalidSessionPayload, MemberJoinPayload, MemberLeavePayload,
    MemberRoleUpdatePayload, MessageCreatePayload, MessageDeletePayload, MessageUpdatePayload,
    PresenceSyncPayload, PresenceUpdatePayload, ReadyPayload, ResumePayload, ResumedPayload,
    RoleDeletePayload, StatusUpdatePayload, TypingStartPayload, TypingStartRequest,
    DEFAULT_HEARTBEAT_INTERVAL_MS, GATEWAY_PROTOCOL_VERSION, GATEWAY_SUBPROTOCOL,
};
pub use instance::{InstanceSettingsResponse, UpdateInstanceSettingsRequest};
pub use mention::{parse_mentions, MentionSet, MessageMentions};
pub use message::{
    AttachmentResponse, CreateMessageRequest, ListMessagesQuery, MessageListResponse,
    MessageReplyPreview, MessageResponse, UpdateMessageRequest,
};
pub use meta::MetaResponse;
pub use pagination::{CursorPage, CursorQuery, DEFAULT_PAGE_LIMIT, MAX_PAGE_LIMIT};
pub use permission_override::{
    PermissionOverrideListResponse, PermissionOverrideResponse, UpsertPermissionOverrideRequest,
};
pub use profile::{PresenceEntry, PresenceListResponse, ProfileResponse, UpdateProfileRequest};
pub use role::{
    AssignRoleRequest, BulkAssignRoleGroupRequest, CreateRoleGroupRequest, CreateRoleRequest,
    ReorderRolesRequest, RoleGroupListResponse, RoleGroupResponse, RoleListResponse, RoleResponse,
    UpdateRoleGroupRequest, UpdateRoleRequest,
};
pub use space::{
    AddSpaceMemberRequest, CreateSpaceRequest, SpaceListResponse, SpaceMemberListResponse,
    SpaceMemberResponse, SpaceResponse, UpdateSpaceRequest,
};
pub use view_as::{ViewAsChannelsRequest, ViewAsChannelsResponse, ViewAsMode};
