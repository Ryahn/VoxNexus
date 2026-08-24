//! OpenAPI document for the HTTP API. Rust handlers are the source of truth.

use utoipa::OpenApi;
use voxnexus_domain::{ChannelType, CommunityMemberRole, JoinMode, SpaceVisibility};
use voxnexus_protocol::{
    AccountResponse, AddSpaceMemberRequest, AssignRoleRequest, AuditEventListResponse,
    AuditEventResponse, AuthSessionResponse, BulkAssignRoleGroupRequest, CategoryListResponse,
    CategoryResponse, ChangeEmailRequest, ChangePasswordRequest, ChannelListResponse,
    ChannelResponse, CommunityListResponse, CommunityMemberListResponse, CommunityMemberResponse,
    CommunityResponse, CreateCategoryRequest, CreateChannelRequest, CreateCommunityRequest,
    CreateInviteRequest, CreateMessageRequest, CreateRoleGroupRequest, CreateRoleRequest,
    CreateSpaceRequest, DeleteCommunityRequest, ErrorBody, ExplainPermissionRequest,
    ExplainPermissionResponse, InstanceSettingsResponse, InviteExpireAfter, InviteExpireUnit,
    InviteListResponse, InvitePreviewResponse, InviteResponse, ListAuditEventsQuery,
    ListCategoriesQuery, ListChannelsQuery, ListMessagesQuery, LoginRequest, MessageListResponse,
    MessageReplyPreview, MessageResponse, MetaResponse, PermissionExplainStep,
    PermissionOverrideListResponse, PermissionOverrideResponse, PresenceEntry,
    PresenceListResponse, ProfileResponse, RegisterRequest, ReorderCategoriesRequest,
    ReorderChannelsRequest, ReorderRolesRequest, RoleGroupListResponse, RoleGroupResponse,
    RoleListResponse, RoleResponse, SpaceListResponse, SpaceMemberListResponse,
    SpaceMemberResponse, SpaceResponse, TransferCommunityRequest, UpdateCategoryRequest,
    UpdateChannelRequest, UpdateCommunityRequest, UpdateInstanceSettingsRequest,
    UpdateInviteRequest, UpdateMessageRequest, UpdateNicknameRequest, UpdateProfileRequest,
    UpdateRoleGroupRequest, UpdateRoleRequest, UpdateSpaceRequest, UpsertPermissionOverrideRequest,
    ViewAsChannelsRequest, ViewAsChannelsResponse, ViewAsMode,
};

use crate::audit;
use crate::auth;
use crate::categories;
use crate::channels;
use crate::communities;
use crate::explain;
use crate::http;
use crate::instance;
use crate::invites;
use crate::messages;
use crate::oidc;
use crate::permission_overrides;
use crate::presence;
use crate::profile;
use crate::roles;
use crate::spaces;
use crate::view_as;

/// Generated OpenAPI 3 document for `/api/v1`.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "VoxNexus HTTP API",
        description = "Versioned HTTP API for a VoxNexus instance. Probe routes `/health`, `/ready`, and `/metrics` are not part of this document."
    ),
    paths(
        http::meta,
        auth::register,
        auth::login,
        auth::logout,
        auth::me,
        auth::change_my_password,
        auth::change_my_email,
        oidc::oidc_start,
        oidc::oidc_callback,
        instance::get_instance_settings,
        instance::update_instance_settings,
        communities::create_community,
        communities::list_communities,
        communities::get_community_by_id,
        communities::update_community_settings,
        communities::transfer_community,
        communities::delete_community,
        communities::upload_community_icon,
        communities::upload_community_banner,
        communities::upload_community_tag_badge,
        communities::upload_community_invite_splash,
        communities::get_community_icon,
        communities::get_community_banner,
        communities::get_community_tag_badge,
        communities::get_community_invite_splash,
        communities::join_community,
        communities::leave_community,
        communities::list_community_members,
        communities::update_my_nickname,
        audit::list_community_audit_events,
        invites::create_community_invite,
        invites::list_community_invites,
        invites::revoke_community_invite,
        invites::update_community_invite,
        invites::get_invite_preview,
        invites::accept_invite,
        spaces::create_space,
        spaces::list_spaces,
        spaces::get_space_by_id,
        spaces::update_space,
        spaces::delete_space,
        spaces::join_space,
        spaces::leave_space,
        spaces::list_space_members,
        spaces::add_space_member,
        spaces::remove_space_member,
        categories::create_category,
        categories::list_categories,
        categories::reorder_categories,
        categories::get_category_by_id,
        categories::update_category,
        categories::delete_category,
        permission_overrides::list_category_permission_overrides,
        permission_overrides::upsert_category_role_permission_override,
        permission_overrides::upsert_category_member_permission_override,
        channels::create_channel,
        channels::list_channels,
        channels::reorder_channels,
        channels::get_channel_by_id,
        channels::update_channel,
        channels::delete_channel,
        channels::archive_channel,
        channels::restore_channel,
        channels::clone_channel,
        messages::create_message,
        messages::list_messages,
        messages::update_message,
        messages::delete_message,
        permission_overrides::list_channel_permission_overrides,
        permission_overrides::upsert_channel_role_permission_override,
        permission_overrides::upsert_channel_member_permission_override,
        permission_overrides::delete_permission_override,
        explain::explain_permission,
        view_as::view_as_channels,
        roles::create_role,
        roles::list_roles,
        roles::reorder_roles,
        roles::get_role_by_id,
        roles::update_role,
        roles::delete_role,
        roles::clone_role,
        roles::list_member_roles,
        roles::assign_member_role,
        roles::remove_member_role,
        roles::create_role_group,
        roles::list_role_groups,
        roles::update_role_group,
        roles::delete_role_group,
        roles::bulk_assign_role_group,
        roles::upload_role_icon,
        roles::get_role_icon,
        roles::delete_role_icon,
        profile::get_my_profile,
        profile::update_my_profile,
        profile::upload_my_avatar,
        profile::upload_my_banner,
        profile::get_profile_by_id,
        profile::get_profile_avatar,
        profile::get_profile_banner,
        presence::list_presence
    ),
    components(schemas(
        MetaResponse,
        InstanceSettingsResponse,
        UpdateInstanceSettingsRequest,
        CreateCommunityRequest,
        UpdateCommunityRequest,
        TransferCommunityRequest,
        DeleteCommunityRequest,
        UpdateNicknameRequest,
        CreateInviteRequest,
        UpdateInviteRequest,
        InviteExpireAfter,
        InviteExpireUnit,
        InviteResponse,
        InviteListResponse,
        InvitePreviewResponse,
        CommunityResponse,
        CommunityListResponse,
        CommunityMemberResponse,
        CommunityMemberListResponse,
        CommunityMemberRole,
        JoinMode,
        SpaceVisibility,
        CreateSpaceRequest,
        UpdateSpaceRequest,
        AddSpaceMemberRequest,
        SpaceResponse,
        SpaceListResponse,
        SpaceMemberResponse,
        SpaceMemberListResponse,
        CategoryResponse,
        CategoryListResponse,
        CreateCategoryRequest,
        UpdateCategoryRequest,
        ReorderCategoriesRequest,
        ListCategoriesQuery,
        ChannelResponse,
        ChannelListResponse,
        CreateChannelRequest,
        UpdateChannelRequest,
        ReorderChannelsRequest,
        ListChannelsQuery,
        ChannelType,
        MessageResponse,
        MessageReplyPreview,
        MessageListResponse,
        CreateMessageRequest,
        UpdateMessageRequest,
        ListMessagesQuery,
        PermissionOverrideResponse,
        PermissionOverrideListResponse,
        UpsertPermissionOverrideRequest,
        ExplainPermissionRequest,
        ExplainPermissionResponse,
        PermissionExplainStep,
        ViewAsChannelsRequest,
        ViewAsChannelsResponse,
        ViewAsMode,
        AuditEventResponse,
        AuditEventListResponse,
        ListAuditEventsQuery,
        RoleResponse,
        RoleListResponse,
        CreateRoleRequest,
        UpdateRoleRequest,
        ReorderRolesRequest,
        AssignRoleRequest,
        RoleGroupResponse,
        RoleGroupListResponse,
        CreateRoleGroupRequest,
        UpdateRoleGroupRequest,
        BulkAssignRoleGroupRequest,
        ErrorBody,
        RegisterRequest,
        LoginRequest,
        ChangePasswordRequest,
        ChangeEmailRequest,
        AccountResponse,
        AuthSessionResponse,
        ProfileResponse,
        UpdateProfileRequest,
        PresenceEntry,
        PresenceListResponse
    )),
    tags(
        (name = "meta", description = "Unauthenticated instance identity"),
        (name = "instance", description = "Instance settings (instance admin)"),
        (name = "communities", description = "Communities and settings"),
        (name = "invites", description = "Community invite links"),
        (name = "spaces", description = "Spaces (groups) within a community"),
        (name = "categories", description = "Channel sidebar categories"),
        (name = "channels", description = "Channels within a community"),
        (name = "messages", description = "Text channel messages"),
        (name = "roles", description = "Community roles and assignments"),
        (name = "permissions", description = "Permission explain, View As, and overrides"),
        (name = "audit", description = "Community audit log"),
        (name = "auth", description = "Registration, login, logout, and session"),
        (name = "profiles", description = "Account profiles, avatars, and banners"),
        (name = "presence", description = "Instance-wide online presence")
    )
)]
pub struct ApiDoc;

/// OpenAPI document matching the running handlers.
#[must_use]
pub fn spec() -> utoipa::openapi::OpenApi {
    let mut spec = ApiDoc::openapi();
    env!("CARGO_PKG_VERSION").clone_into(&mut spec.info.version);
    spec
}

/// Pretty JSON for `packages/api-client/openapi.json`.
///
/// # Panics
///
/// Panics if the in-memory OpenAPI document cannot be serialized, which indicates a bug in utoipa.
#[must_use]
pub fn spec_json() -> String {
    let mut json = serde_json::to_string_pretty(&spec()).expect("openapi serializes");
    if !json.ends_with('\n') {
        json.push('\n');
    }
    json
}

#[cfg(test)]
mod tests {
    use super::{spec, spec_json};
    use std::path::PathBuf;

    #[test]
    fn spec_is_deterministic() {
        let first = serde_json::to_value(spec()).expect("spec");
        let second = serde_json::to_value(spec()).expect("spec");
        assert_eq!(first, second);
    }

    #[test]
    fn committed_openapi_matches_handlers() {
        let committed_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/api-client/openapi.json");
        let committed = std::fs::read_to_string(&committed_path).unwrap_or_else(|error| {
            panic!(
                "missing committed OpenAPI at {}: {error}; run `pnpm codegen`",
                committed_path.display()
            )
        });
        let committed: serde_json::Value =
            serde_json::from_str(&committed).expect("committed openapi.json");
        let generated: serde_json::Value =
            serde_json::from_str(&spec_json()).expect("generated openapi");
        assert_eq!(
            generated, committed,
            "packages/api-client/openapi.json is stale; run `pnpm codegen`"
        );
    }
}
