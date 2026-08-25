//! Message attachment metadata (F038).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Attachment bound to a channel (and optionally a message).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub id: Uuid,
    pub message_id: Option<Uuid>,
    pub channel_id: Uuid,
    pub community_id: Uuid,
    pub object_id: Uuid,
    pub thumbnail_object_id: Option<Uuid>,
    pub filename: String,
    pub content_type: String,
    pub byte_size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}
