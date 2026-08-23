//! Community HTTP DTOs (F019).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use voxnexus_domain::JoinMode;

/// Public community representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CommunityResponse {
    pub id: uuid::Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub timezone: String,
    pub join_mode: JoinMode,
    pub owner_account_id: uuid::Uuid,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub discoverable_on_instance: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

/// Create a community.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateCommunityRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 48))]
    pub slug: Option<String>,
    #[validate(length(max = 2000))]
    pub description: Option<String>,
    #[validate(length(min = 1, max = 64))]
    pub timezone: Option<String>,
    pub join_mode: Option<JoinMode>,
    pub discoverable_on_instance: Option<bool>,
}

/// Update community settings (owner).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateCommunityRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(max = 2000))]
    pub description: Option<String>,
    #[validate(length(min = 1, max = 64))]
    pub timezone: Option<String>,
    pub join_mode: Option<JoinMode>,
    pub discoverable_on_instance: Option<bool>,
}

/// List wrapper for communities the caller belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CommunityListResponse {
    pub communities: Vec<CommunityResponse>,
}
