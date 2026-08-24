//! View As simulation HTTP DTOs (F032).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::channel::ChannelResponse;

/// How to build the synthetic actor for View As.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ViewAsMode {
    /// Simulate a real community member (including their roles and member overrides).
    Member,
    /// Simulate `@everyone` plus the given role ids (no member overrides).
    Roles,
    /// Simulate a non-member visitor.
    Visitor,
}

/// Request a simulated channel list for an admin preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct ViewAsChannelsRequest {
    pub community_id: Uuid,
    /// Limit simulation to one Space (recommended for the sidebar).
    pub space_id: Option<Uuid>,
    pub mode: ViewAsMode,
    /// Required when `mode` is `member`.
    pub account_id: Option<Uuid>,
    /// Extra role ids when `mode` is `roles` (`@everyone` is always included).
    #[serde(default)]
    pub role_ids: Vec<Uuid>,
}

/// Simulated channel visibility for View As (read-only; does not change the session).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ViewAsChannelsResponse {
    pub mode: ViewAsMode,
    /// Short label for the toolbar (e.g. member display name or role names).
    pub label: String,
    pub channels: Vec<ChannelResponse>,
}
