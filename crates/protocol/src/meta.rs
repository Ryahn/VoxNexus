use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use voxnexus_domain::{CommunityCreationMode, RegistrationMode};

/// Build, version, and public instance policy flags for the SPA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MetaResponse {
    pub name: String,
    pub version: String,
    pub registration_mode: RegistrationMode,
    pub community_creation_mode: CommunityCreationMode,
    /// Whether SSO sign-in is configured and enabled for this instance.
    pub oidc_enabled: bool,
    /// Whether email/password login and registration are offered.
    pub password_login_enabled: bool,
}
