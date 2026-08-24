//! Audit event domain type (F033).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Append-only audit log row for a community.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub community_id: Uuid,
    pub actor_account_id: Option<Uuid>,
    pub action: String,
    pub space_id: Option<Uuid>,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub summary: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}
