//! Community HTTP DTOs (F019 / F020).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use voxnexus_domain::{CommunityMemberRole, JoinMode};

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
    pub tag_name: String,
    pub tag_color: String,
    pub tag_badge_url: Option<String>,
    pub invite_splash_url: Option<String>,
    pub invite_path: Option<String>,
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
    #[validate(length(max = 8))]
    pub tag_name: Option<String>,
    #[validate(length(max = 32))]
    pub tag_color: Option<String>,
    #[validate(length(max = 48))]
    pub invite_path: Option<String>,
}

/// List wrapper for communities the caller belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CommunityListResponse {
    pub communities: Vec<CommunityResponse>,
}

/// One community member with profile display fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CommunityMemberResponse {
    pub community_id: uuid::Uuid,
    pub account_id: uuid::Uuid,
    pub role: CommunityMemberRole,
    pub nickname: String,
    pub display_name: String,
    pub has_avatar: bool,
    pub avatar_url: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub joined_at: DateTime<Utc>,
}

/// Cursor page of community members.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CommunityMemberListResponse {
    pub items: Vec<CommunityMemberResponse>,
    pub has_more: bool,
}

/// Update the caller's nickname in a community.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateNicknameRequest {
    #[validate(length(max = 32))]
    pub nickname: String,
}

/// Create a community invite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateInviteRequest {
    /// `null` / omitted = unlimited. Otherwise `1..=1000`.
    #[validate(range(min = 1, max = 1000))]
    pub max_uses: Option<i32>,
    /// Relative expiry from creation time. Omit for no expiry.
    pub expire_after: Option<InviteExpireAfter>,
}

/// Relative invite lifetime chosen before code generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct InviteExpireAfter {
    pub unit: InviteExpireUnit,
    pub value: u32,
}

/// Units for [`InviteExpireAfter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum InviteExpireUnit {
    Hours,
    Days,
    Months,
}

/// Pause / unpause an invite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateInviteRequest {
    pub paused: Option<bool>,
}

/// Public invite representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct InviteResponse {
    pub id: uuid::Uuid,
    pub community_id: uuid::Uuid,
    pub code: String,
    pub created_by: uuid::Uuid,
    pub max_uses: Option<i32>,
    pub uses: i32,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub expires_at: Option<DateTime<Utc>>,
    pub paused: bool,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub revoked_at: Option<DateTime<Utc>>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

/// List of invites for a community.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct InviteListResponse {
    pub invites: Vec<InviteResponse>,
}

/// Preview for join-by-code UI (no sensitive creator detail beyond community).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct InvitePreviewResponse {
    pub code: String,
    pub community_id: uuid::Uuid,
    pub community_name: String,
    pub community_slug: String,
    pub paused: bool,
    pub expired: bool,
    pub exhausted: bool,
}

/// Transfer community ownership to another member (F025).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct TransferCommunityRequest {
    pub account_id: uuid::Uuid,
}

/// Confirm community deletion by typing the community name (F025).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct DeleteCommunityRequest {
    #[validate(length(min = 1, max = 100))]
    pub confirm_name: String,
}
