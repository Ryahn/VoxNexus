//! Space HTTP DTOs (F022).

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
