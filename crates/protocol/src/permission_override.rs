//! Channel/category permission override DTOs (F030).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Stored permission override.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PermissionOverrideResponse {
    pub id: Uuid,
    pub community_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub role_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub permissions: Value,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

/// Overrides for a channel (includes category-scoped rows when applicable).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PermissionOverrideListResponse {
    pub overrides: Vec<PermissionOverrideResponse>,
}

/// Upsert a role or member override on a channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpsertPermissionOverrideRequest {
    pub permissions: Value,
}
