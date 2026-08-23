//! Community and membership domain types (F019 / F020 / F021 / F022).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// How accounts may join a community (invites/applications expand in later features).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum JoinMode {
    Open,
    Invite,
    Application,
}

impl JoinMode {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Invite => "invite",
            Self::Application => "application",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "open" => Some(Self::Open),
            "invite" => Some(Self::Invite),
            "application" => Some(Self::Application),
            _ => None,
        }
    }
}

/// Who can see / enter a Space (F023 membership expands access rules).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SpaceVisibility {
    Open,
    Restricted,
}

impl SpaceVisibility {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Restricted => "restricted",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "open" => Some(Self::Open),
            "restricted" => Some(Self::Restricted),
            _ => None,
        }
    }
}

/// Membership role within a community.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CommunityMemberRole {
    Owner,
    Member,
}

impl CommunityMemberRole {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Member => "member",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "owner" => Some(Self::Owner),
            "member" => Some(Self::Member),
            _ => None,
        }
    }
}

/// A community (Discord-style server) on this instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Community {
    pub id: Uuid,
    pub instance_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub timezone: String,
    pub join_mode: JoinMode,
    pub owner_account_id: Uuid,
    pub icon_object_id: Option<Uuid>,
    pub banner_object_id: Option<Uuid>,
    pub discoverable_on_instance: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Account membership in a community.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunityMember {
    pub community_id: Uuid,
    pub account_id: Uuid,
    pub role: CommunityMemberRole,
    pub nickname: String,
    pub joined_at: DateTime<Utc>,
}

/// Invite link into a community (F021).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunityInvite {
    pub id: Uuid,
    pub community_id: Uuid,
    pub code: String,
    pub created_by: Uuid,
    pub max_uses: Option<i32>,
    pub uses: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub paused: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl CommunityInvite {
    /// Whether this invite can still be accepted right now.
    #[must_use]
    pub fn is_acceptably_active(&self, now: DateTime<Utc>) -> bool {
        self.revoked_at.is_none()
            && !self.paused
            && self.expires_at.is_none_or(|expires| expires > now)
            && self.max_uses.is_none_or(|max| self.uses < max)
    }
}

/// A Space within a community (Guilded-style group). Spaces are flat — never nested.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Space {
    pub id: Uuid,
    pub community_id: Uuid,
    pub name: String,
    pub description: String,
    pub topic: String,
    pub game: String,
    pub visibility: SpaceVisibility,
    pub icon_object_id: Option<Uuid>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Membership in a Space (F023).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceMember {
    pub space_id: Uuid,
    pub account_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

/// Sidebar category grouping channels (F026). Scoped to a community or a Space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelCategory {
    pub id: Uuid,
    pub community_id: Uuid,
    pub space_id: Option<Uuid>,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Channel type registry (F027). Typed features attach in later milestones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    Text,
    Voice,
    Forum,
    Announcement,
    Calendar,
    Scheduling,
    Docs,
    Tasks,
    Media,
    Stage,
    Streaming,
}

impl ChannelType {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Voice => "voice",
            Self::Forum => "forum",
            Self::Announcement => "announcement",
            Self::Calendar => "calendar",
            Self::Scheduling => "scheduling",
            Self::Docs => "docs",
            Self::Tasks => "tasks",
            Self::Media => "media",
            Self::Stage => "stage",
            Self::Streaming => "streaming",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "text" => Some(Self::Text),
            "voice" => Some(Self::Voice),
            "forum" => Some(Self::Forum),
            "announcement" => Some(Self::Announcement),
            "calendar" => Some(Self::Calendar),
            "scheduling" => Some(Self::Scheduling),
            "docs" => Some(Self::Docs),
            "tasks" => Some(Self::Tasks),
            "media" => Some(Self::Media),
            "stage" => Some(Self::Stage),
            "streaming" => Some(Self::Streaming),
            _ => None,
        }
    }
}

/// A channel within a community (optional Space / category scope).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Channel {
    pub id: Uuid,
    pub community_id: Uuid,
    pub space_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub channel_type: ChannelType,
    pub name: String,
    pub topic: String,
    pub position: i32,
    pub archived_at: Option<DateTime<Utc>>,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Custom role within a community (F028). `@everyone` is implicit for all members.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunityRole {
    pub id: Uuid,
    pub community_id: Uuid,
    pub name: String,
    pub position: i32,
    pub color: String,
    pub hoist: bool,
    pub mentionable: bool,
    pub permissions: serde_json::Value,
    pub is_everyone: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
