//! Audit log HTTP DTOs (F033).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

/// Query filters for community audit log listing.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema, IntoParams, Validate,
)]
pub struct ListAuditEventsQuery {
    pub before: Option<Uuid>,
    pub after: Option<Uuid>,
    pub limit: Option<u16>,
    pub actor_account_id: Option<Uuid>,
    #[validate(length(max = 64))]
    pub action: Option<String>,
    pub space_id: Option<Uuid>,
}

impl ListAuditEventsQuery {
    /// Limit clamped to `1..=100`, default 50.
    #[must_use]
    pub fn resolved_limit(&self) -> u16 {
        self.limit.unwrap_or(50).clamp(1, 100)
    }
}

/// One audit log entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AuditEventResponse {
    pub id: Uuid,
    pub community_id: Uuid,
    pub actor_account_id: Option<Uuid>,
    pub action: String,
    pub space_id: Option<Uuid>,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub summary: String,
    #[schema(value_type = Object)]
    pub metadata: Value,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

/// Cursor page of audit events (newest first).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AuditEventListResponse {
    pub items: Vec<AuditEventResponse>,
    pub has_more: bool,
}
