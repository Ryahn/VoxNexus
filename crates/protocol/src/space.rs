//! Space HTTP DTOs (F022 / F023).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;
use voxnexus_domain::SpaceVisibility;

/// Public Space representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SpaceResponse {
    pub id: Uuid,
    pub community_id: Uuid,
    pub name: String,
    pub description: String,
    pub topic: String,
    pub game: String,
    pub visibility: SpaceVisibility,
    pub icon_url: Option<String>,
    pub position: i32,
    /// Whether the caller is a member of this space.
    pub is_member: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

/// Create a Space in a community.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateSpaceRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(max = 2000))]
    pub description: Option<String>,
    #[validate(length(max = 200))]
    pub topic: Option<String>,
    #[validate(length(max = 100))]
    pub game: Option<String>,
    pub visibility: Option<SpaceVisibility>,
}

/// Update Space settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateSpaceRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(max = 2000))]
    pub description: Option<String>,
    #[validate(length(max = 200))]
    pub topic: Option<String>,
    #[validate(length(max = 100))]
    pub game: Option<String>,
    pub visibility: Option<SpaceVisibility>,
    pub position: Option<i32>,
}

/// Spaces in a community, ordered by position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SpaceListResponse {
    pub spaces: Vec<SpaceResponse>,
}

/// One space member with profile display fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SpaceMemberResponse {
    pub space_id: Uuid,
    pub account_id: Uuid,
    pub display_name: String,
    pub has_avatar: bool,
    pub avatar_url: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub joined_at: DateTime<Utc>,
}

/// Space members list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SpaceMemberListResponse {
    pub members: Vec<SpaceMemberResponse>,
}

/// Add a community member to a space (restricted / manual).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct AddSpaceMemberRequest {
    pub account_id: Uuid,
}
