//! OpenAPI document for the HTTP API. Rust handlers are the source of truth.

use utoipa::OpenApi;
use voxnexus_protocol::{
    AccountResponse, AuthSessionResponse, ChangeEmailRequest, ChangePasswordRequest,
    CommunityCreateAcceptedResponse, ErrorBody, InstanceSettingsResponse, LoginRequest,
    MetaResponse, ProfileResponse, RegisterRequest, UpdateInstanceSettingsRequest,
    UpdateProfileRequest,
};

use crate::auth;
use crate::communities;
use crate::http;
use crate::instance;
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
        instance::get_instance_settings,
        instance::update_instance_settings,
        communities::create_community,
        profile::get_my_profile,
        profile::update_my_profile,
        profile::upload_my_avatar,
        profile::upload_my_banner,
        profile::get_profile_by_id,
        profile::get_profile_avatar,
        profile::get_profile_banner
    ),
    components(schemas(
        MetaResponse,
        InstanceSettingsResponse,
        UpdateInstanceSettingsRequest,
        CommunityCreateAcceptedResponse,
        ErrorBody,
        RegisterRequest,
        LoginRequest,
        ChangePasswordRequest,
        ChangeEmailRequest,
        AccountResponse,
        AuthSessionResponse,
        ProfileResponse,
        UpdateProfileRequest
    )),
    tags(
        (name = "meta", description = "Unauthenticated instance identity"),
        (name = "instance", description = "Instance settings (instance admin)"),
        (name = "communities", description = "Community creation policy"),
        (name = "auth", description = "Registration, login, logout, and session"),
        (name = "profiles", description = "Account profiles, avatars, and banners")
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
