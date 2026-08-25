//! Message attachments (F038).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use chrono::Utc;
use sqlx::FromRow;
use uuid::Uuid;
use voxnexus_domain::MessageAttachment;

use crate::AuthError;
use sqlx::PgPool;

/// Max attachments linked on one message create.
pub const MESSAGE_ATTACHMENTS_MAX: usize = 10;

#[derive(Debug, Clone, FromRow)]
struct AttachmentRow {
    id: Uuid,
    message_id: Option<Uuid>,
    channel_id: Uuid,
    community_id: Uuid,
    object_id: Uuid,
    thumbnail_object_id: Option<Uuid>,
    filename: String,
    content_type: String,
    byte_size: i64,
    width: Option<i32>,
    height: Option<i32>,
    created_by: Uuid,
    created_at: chrono::DateTime<Utc>,
}

impl AttachmentRow {
    fn into_attachment(self) -> MessageAttachment {
        MessageAttachment {
            id: self.id,
            message_id: self.message_id,
            channel_id: self.channel_id,
            community_id: self.community_id,
            object_id: self.object_id,
            thumbnail_object_id: self.thumbnail_object_id,
            filename: self.filename,
            content_type: self.content_type,
            byte_size: self.byte_size,
            width: self.width,
            height: self.height,
            created_by: self.created_by,
            created_at: self.created_at,
        }
    }
}

/// Insert a pending attachment (not yet bound to a message).
#[allow(clippy::too_many_arguments)]
pub async fn create_pending_attachment(
    pool: &PgPool,
    channel_id: Uuid,
    community_id: Uuid,
    object_id: Uuid,
    filename: &str,
    content_type: &str,
    byte_size: i64,
    width: Option<i32>,
    height: Option<i32>,
    created_by: Uuid,
) -> Result<MessageAttachment, AuthError> {
    let id = Uuid::now_v7();
    let now = Utc::now();
    sqlx::query(
        r"
        INSERT INTO message_attachments (
            id, message_id, channel_id, community_id, object_id, thumbnail_object_id,
            filename, content_type, byte_size, width, height, created_by, created_at
        )
        VALUES ($1, NULL, $2, $3, $4, NULL, $5, $6, $7, $8, $9, $10, $11)
        ",
    )
    .bind(id)
    .bind(channel_id)
    .bind(community_id)
    .bind(object_id)
    .bind(filename)
    .bind(content_type)
    .bind(byte_size)
    .bind(width)
    .bind(height)
    .bind(created_by)
    .bind(now)
    .execute(pool)
    .await?;
    get_attachment(pool, id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Load one attachment by id.
pub async fn get_attachment(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<MessageAttachment>, AuthError> {
    let row = sqlx::query_as::<_, AttachmentRow>(
        r"
        SELECT id, message_id, channel_id, community_id, object_id, thumbnail_object_id,
               filename, content_type, byte_size, width, height, created_by, created_at
        FROM message_attachments
        WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(AttachmentRow::into_attachment))
}

/// List attachments for many messages (stable order by created_at).
pub async fn list_attachments_for_messages(
    pool: &PgPool,
    message_ids: &[Uuid],
) -> Result<Vec<MessageAttachment>, AuthError> {
    if message_ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query_as::<_, AttachmentRow>(
        r"
        SELECT id, message_id, channel_id, community_id, object_id, thumbnail_object_id,
               filename, content_type, byte_size, width, height, created_by, created_at
        FROM message_attachments
        WHERE message_id = ANY($1)
        ORDER BY created_at ASC
        ",
    )
    .bind(message_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(AttachmentRow::into_attachment).collect())
}

/// Bind pending attachments owned by `created_by` in `channel_id` to a message.
pub async fn bind_attachments_to_message(
    pool: &PgPool,
    message_id: Uuid,
    channel_id: Uuid,
    created_by: Uuid,
    attachment_ids: &[Uuid],
) -> Result<Vec<MessageAttachment>, AuthError> {
    if attachment_ids.is_empty() {
        return Ok(Vec::new());
    }
    if attachment_ids.len() > MESSAGE_ATTACHMENTS_MAX {
        return Err(AuthError::InvalidAttachment);
    }
    let mut bound = Vec::with_capacity(attachment_ids.len());
    for attachment_id in attachment_ids {
        let result = sqlx::query(
            r"
            UPDATE message_attachments
            SET message_id = $1
            WHERE id = $2
              AND channel_id = $3
              AND created_by = $4
              AND message_id IS NULL
            ",
        )
        .bind(message_id)
        .bind(attachment_id)
        .bind(channel_id)
        .bind(created_by)
        .execute(pool)
        .await?;
        if result.rows_affected() != 1 {
            return Err(AuthError::InvalidAttachment);
        }
        let row = get_attachment(pool, *attachment_id)
            .await?
            .ok_or(AuthError::InvalidAttachment)?;
        bound.push(row);
    }
    Ok(bound)
}

/// Set thumbnail object id when missing.
pub async fn set_attachment_thumbnail(
    pool: &PgPool,
    attachment_id: Uuid,
    thumbnail_object_id: Uuid,
) -> Result<Option<MessageAttachment>, AuthError> {
    sqlx::query(
        r"
        UPDATE message_attachments
        SET thumbnail_object_id = $2
        WHERE id = $1
          AND thumbnail_object_id IS NULL
        ",
    )
    .bind(attachment_id)
    .bind(thumbnail_object_id)
    .execute(pool)
    .await?;
    get_attachment(pool, attachment_id).await
}
