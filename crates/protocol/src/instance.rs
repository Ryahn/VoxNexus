use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use voxnexus_domain::{CommunityCreationMode, RegistrationMode};

/// Instance settings visible to instance admins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct InstanceSettingsResponse {
    pub id: uuid::Uuid,
    pub name: String,
    pub public_url: String,
    pub registration_mode: RegistrationMode,
    pub community_creation_mode: CommunityCreationMode,
    /// When true, [`UpdateInstanceSettingsRequest::community_creation_mode`] is ignored.
    pub community_creation_mode_locked: bool,
    pub oidc_enabled: bool,
    pub oidc_issuer: Option<String>,
    pub oidc_client_id: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

/// Partial update of instance settings (instance admin only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateInstanceSettingsRequest {
    #[validate(length(min = 1, max = 128))]
    pub name: Option<String>,
    #[validate(url)]
    pub public_url: Option<String>,
    pub registration_mode: Option<RegistrationMode>,
    pub community_creation_mode: Option<CommunityCreationMode>,
    pub oidc_enabled: Option<bool>,
    pub oidc_issuer: Option<Option<String>>,
    pub oidc_client_id: Option<Option<String>>,
}
