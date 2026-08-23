//! OpenAPI document for the HTTP API. Rust handlers are the source of truth.

use utoipa::OpenApi;
use voxnexus_domain::{CommunityMemberRole, JoinMode};
use voxnexus_protocol::{
    AccountResponse, AuthSessionResponse, ChangeEmailRequest, ChangePasswordRequest,
    CommunityListResponse, CommunityMemberListResponse, CommunityMemberResponse, CommunityResponse,
    CreateCommunityRequest, CreateInviteRequest, ErrorBody, InstanceSettingsResponse,
    InviteExpireAfter, InviteExpireUnit, InviteListResponse, InvitePreviewResponse, InviteResponse,
    LoginRequest, MetaResponse, PresenceEntry, PresenceListResponse, ProfileResponse,
    RegisterRequest, UpdateCommunityRequest, UpdateInstanceSettingsRequest, UpdateInviteRequest,
    UpdateNicknameRequest, UpdateProfileRequest,
};

use crate::auth;
use crate::communities;
use crate::http;
use crate::instance;
use crate::invites;
use crate::oidc;
use crate::presence;
use crate::profile;

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
        communities::upload_community_icon,
        communities::upload_community_banner,
        communities::get_community_icon,
        communities::get_community_banner,
        communities::join_community,
        communities::leave_community,
        communities::list_community_members,
        communities::update_my_nickname,
        invites::create_community_invite,
        invites::list_community_invites,
        invites::revoke_community_invite,
        invites::update_community_invite,
        invites::get_invite_preview,
        invites::accept_invite,
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
