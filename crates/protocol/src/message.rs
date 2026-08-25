//! Message HTTP DTOs (F034–F038).

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

/// Preview of the message being replied to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct MessageReplyPreview {
    pub message_id: Uuid,
    pub author_id: Uuid,
    pub author_display_name: String,
    pub excerpt: String,
    pub deleted: bool,
}

/// Attachment metadata on a message (or pending upload).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct AttachmentResponse {
    pub id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub byte_size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub url: String,
    pub thumbnail_url: Option<String>,
}

/// Public message representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MessageResponse {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub community_id: Uuid,
    pub author_id: Uuid,
    pub author_display_name: String,
    pub content: String,
    pub nonce: Option<String>,
    pub referenced_message_id: Option<Uuid>,
    pub reply_to: Option<MessageReplyPreview>,
    #[serde(default)]
    pub attachments: Vec<AttachmentResponse>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub edited_at: Option<DateTime<Utc>>,
}

/// Create a message in a text channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateMessageRequest {
    #[validate(length(max = 4000))]
    pub content: String,
    #[validate(length(min = 1, max = 128))]
    pub nonce: Option<String>,
    /// Reply target; must be a message in the same channel.
    pub referenced_message_id: Option<Uuid>,
    /// Pending attachment ids from `POST …/attachments` (max 10).
    pub attachment_ids: Option<Vec<Uuid>>,
}

/// Edit a message's content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateMessageRequest {
    #[validate(length(min = 1, max = 4000))]
    pub content: String,
}

/// Cursor page of messages (newest first).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MessageListResponse {
    pub items: Vec<MessageResponse>,
    pub has_more: bool,
}

/// List messages query (`before` = older page, `after` = newer page).
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema, IntoParams, Validate,
)]
pub struct ListMessagesQuery {
    pub before: Option<Uuid>,
    pub after: Option<Uuid>,
    pub limit: Option<u16>,
}

impl ListMessagesQuery {
    /// Limit clamped to `1..=100`, default 50.
    #[must_use]
    pub fn resolved_limit(&self) -> u16 {
        self.limit.unwrap_or(50).clamp(1, 100)
    }
}
