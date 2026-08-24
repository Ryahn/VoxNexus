//! Community role HTTP DTOs (F028 / F030-A).

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
    /// Display order only (drag-reorder).
    pub position: i32,
    /// Unique 1–1000; lower = higher priority.
    pub weight: i32,
    pub group_id: Option<Uuid>,
    pub color: String,
    pub hoist: bool,
    pub mentionable: bool,
    pub permissions: Value,
    pub is_everyone: bool,
    pub short_tag: String,
    pub icon_emoji: Option<String>,
    pub icon_object_key: Option<String>,
    pub gradient: Option<String>,
    pub role_card: Value,
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
    pub weight: Option<i32>,
    pub group_id: Option<Uuid>,
    #[validate(length(max = 16))]
    pub short_tag: Option<String>,
    #[validate(length(max = 32))]
    pub icon_emoji: Option<String>,
    #[validate(length(max = 128))]
    pub gradient: Option<String>,
    pub role_card: Option<Value>,
    pub permissions: Option<Value>,
}

/// Update a role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub position: Option<i32>,
    pub weight: Option<i32>,
    pub group_id: Option<Uuid>,
    /// When true, remove the role from its group.
    pub clear_group: Option<bool>,
    #[validate(length(min = 1, max = 32))]
    pub color: Option<String>,
    pub hoist: Option<bool>,
    pub mentionable: Option<bool>,
    pub manage_roles: Option<bool>,
    pub permissions: Option<Value>,
    #[validate(length(max = 16))]
    pub short_tag: Option<String>,
    #[validate(length(max = 32))]
    pub icon_emoji: Option<String>,
    /// Set true to clear icon emoji.
    pub clear_icon_emoji: Option<bool>,
    #[validate(length(max = 128))]
    pub gradient: Option<String>,
    pub clear_gradient: Option<bool>,
    pub role_card: Option<Value>,
}

/// Roles in a community.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RoleListResponse {
    pub roles: Vec<RoleResponse>,
}

/// Reorder roles by id list (display `position` only; does not change weight).
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

/// Role group (organizational).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RoleGroupResponse {
    pub id: Uuid,
    pub community_id: Uuid,
    pub name: String,
    pub display_order: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

/// List of role groups.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RoleGroupListResponse {
    pub groups: Vec<RoleGroupResponse>,
}

/// Create a role group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateRoleGroupRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
}

/// Update a role group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateRoleGroupRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub display_order: Option<i32>,
}

/// Bulk assign roles to a group (or ungroup when `group_id` is null).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct BulkAssignRoleGroupRequest {
    #[validate(length(min = 1, max = 200))]
    pub role_ids: Vec<Uuid>,
    pub group_id: Option<Uuid>,
}
