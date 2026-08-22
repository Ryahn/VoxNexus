//! Profile HTTP DTOs.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Public profile fields.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileResponse {
    pub account_id: Uuid,
    pub display_name: String,
    pub bio: String,
    pub has_avatar: bool,
    pub has_banner: bool,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
}

/// Patch display name and/or bio.
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 1, max = 64))]
    pub display_name: Option<String>,
    #[validate(length(max = 500))]
    pub bio: Option<String>,
}
