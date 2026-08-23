//! Community role HTTP DTOs (F028).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Public role representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RoleResponse {
    pub id: Uuid,
    pub community_id: Uuid,
    pub name: String,
    pub position: i32,
    pub color: String,
    pub hoist: bool,
    pub mentionable: bool,
    pub permissions: Value,
    pub is_everyone: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

/// Create a custom role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 32))]
    pub color: Option<String>,
    pub hoist: Option<bool>,
    pub mentionable: Option<bool>,
    pub manage_roles: Option<bool>,
}

/// Update a role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub position: Option<i32>,
    #[validate(length(min = 1, max = 32))]
    pub color: Option<String>,
    pub hoist: Option<bool>,
    pub mentionable: Option<bool>,
    pub manage_roles: Option<bool>,
}

/// Roles in a community.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RoleListResponse {
    pub roles: Vec<RoleResponse>,
}

/// Reorder roles by id list (`@everyone` stays at position 0).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct ReorderRolesRequest {
    #[validate(length(min = 1, max = 200))]
    pub role_ids: Vec<Uuid>,
}

/// Assign a role to a member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
}
