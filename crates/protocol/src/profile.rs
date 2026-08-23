//! Profile HTTP DTOs.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;
use voxnexus_domain::{PresenceStatus, PublicPresenceStatus};

/// Public profile fields.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileResponse {
    pub account_id: Uuid,
    pub display_name: String,
    pub bio: String,
    pub presence_status: PublicPresenceStatus,
    pub custom_status: String,
    pub has_avatar: bool,
    pub has_banner: bool,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
}

/// Patch display name, bio, presence, and/or custom status.
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct UpdateProfileRequest {
    /// Empty string is allowed (new accounts start blank until set).
    #[validate(length(max = 64))]
    pub display_name: Option<String>,
    #[validate(length(max = 500))]
    pub bio: Option<String>,
    pub presence_status: Option<PresenceStatus>,
    #[validate(length(max = 128))]
    pub custom_status: Option<String>,
}

/// Instance-wide presence snapshot (F018).
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PresenceEntry {
    pub account_id: Uuid,
    pub status: PublicPresenceStatus,
    pub custom_status: String,
}

/// Online users visible on this instance.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PresenceListResponse {
    pub presences: Vec<PresenceEntry>,
}
