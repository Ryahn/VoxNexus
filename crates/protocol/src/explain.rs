//! Permission explain HTTP DTOs (F031).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Explain why an actor has or lacks a permission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct ExplainPermissionRequest {
    pub community_id: Uuid,
    pub account_id: Uuid,
    #[validate(length(min = 1, max = 64))]
    pub permission: String,
    pub channel_id: Option<Uuid>,
}

/// One step in an explanation chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PermissionExplainStep {
    pub stage: String,
    pub outcome: String,
    pub detail: String,
}

/// Permission explain result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ExplainPermissionResponse {
    pub allowed: bool,
    pub permission: String,
    pub account_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub steps: Vec<PermissionExplainStep>,
}
