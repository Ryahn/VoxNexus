//! Message persistence (F034–F037).

use chrono::Utc;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use voxnexus_domain::Message;

use crate::AuthError;

/// Max message body length (chars), matching DB check constraint.
pub const MESSAGE_CONTENT_MAX: usize = 4000;

/// Max client nonce length.
pub const MESSAGE_NONCE_MAX: usize = 128;

/// Optional author edit window after `created_at`. `None` means unlimited (F036 default).
pub const MESSAGE_EDIT_WINDOW_SECS: Option<i64> = None;

/// Excerpt length for reply previews.
pub const REPLY_EXCERPT_MAX: usize = 120;

/// Cursor page of messages (newest first when listing without `before`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessagesPage {
    pub items: Vec<MessageWithAuthor>,
    pub has_more: bool,
}

/// Joined reply preview for API/gateway payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageReplyPreview {
    pub message_id: Uuid,
    pub author_id: Uuid,
    pub author_display_name: String,
    pub excerpt: String,
    pub deleted: bool,
}

/// Message plus author display name (and optional reply preview).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageWithAuthor {
    pub message: Message,
    pub author_display_name: String,
    pub reply: Option<MessageReplyPreview>,
}

/// Create a text message. Idempotent when `nonce` is set (same channel+author+nonce).
///
/// Returns `(row, created)` where `created` is false on idempotent replay.
///
/// # Errors
///
/// Returns database errors, [`AuthError::InvalidMessage`], or [`AuthError::InvalidReplyTarget`].
pub async fn create_message(
    pool: &PgPool,
    channel_id: Uuid,
    community_id: Uuid,
    author_id: Uuid,
    content: &str,
    nonce: Option<&str>,
    referenced_message_id: Option<Uuid>,
) -> Result<(MessageWithAuthor, bool), AuthError> {
    let content = content.trim();
    if content.is_empty() || content.chars().count() > MESSAGE_CONTENT_MAX {
        return Err(AuthError::InvalidMessage);
    }
    let nonce = nonce
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    if let Some(ref value) = nonce {
        if value.len() > MESSAGE_NONCE_MAX {
            return Err(AuthError::InvalidMessage);
        }
        if let Some(existing) = get_message_by_nonce(pool, channel_id, author_id, value).await? {
            return Ok((existing, false));
        }
    }

    if let Some(ref_id) = referenced_message_id {
        validate_reply_target(pool, channel_id, ref_id).await?;
    }

    let id = Uuid::now_v7();
    let now = Utc::now();
    let insert = sqlx::query(
        r"
        INSERT INTO messages (
            id, channel_id, community_id, author_id, content, nonce,
            referenced_message_id, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ",
    )
    .bind(id)
    .bind(channel_id)
    .bind(community_id)
    .bind(author_id)
    .bind(content)
    .bind(&nonce)
    .bind(referenced_message_id)
    .bind(now)
    .execute(pool)
    .await;

    match insert {
        Ok(_) => {}
        Err(sqlx::Error::Database(db))
            if db.constraint() == Some("messages_channel_author_nonce_unique") =>
        {
            let nonce = nonce.ok_or(AuthError::InvalidMessage)?;
            let existing = get_message_by_nonce(pool, channel_id, author_id, &nonce)
                .await?
                .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
            return Ok((existing, false));
        }
        Err(error) => return Err(AuthError::Db(error)),
    }

    get_message(pool, id, None)
        .await?
        .map(|row| (row, true))
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// List messages in a channel, newest first.
///
/// # Errors
///
/// Returns database errors.
pub async fn list_messages(
    pool: &PgPool,
    channel_id: Uuid,
    before: Option<Uuid>,
    after: Option<Uuid>,
    limit: u16,
) -> Result<MessagesPage, AuthError> {
    let fetch = i64::from(limit) + 1;

    let rows = if let Some(before_id) = before {
        sqlx::query_as::<_, MessageAuthorRow>(&format!(
            "{MESSAGE_SELECT_BASE} AND m.id < $2 ORDER BY m.id DESC LIMIT $3"
        ))
        .bind(channel_id)
        .bind(before_id)
        .bind(fetch)
        .fetch_all(pool)
        .await?
    } else if let Some(after_id) = after {
        let mut rows = sqlx::query_as::<_, MessageAuthorRow>(&format!(
            "{MESSAGE_SELECT_BASE} AND m.id > $2 ORDER BY m.id ASC LIMIT $3"
        ))
        .bind(channel_id)
        .bind(after_id)
        .bind(fetch)
        .fetch_all(pool)
        .await?;
        rows.reverse();
        rows
    } else {
        sqlx::query_as::<_, MessageAuthorRow>(&format!(
            "{MESSAGE_SELECT_BASE} ORDER BY m.id DESC LIMIT $2"
        ))
        .bind(channel_id)
        .bind(fetch)
        .fetch_all(pool)
        .await?
    };

    let has_more = rows.len() > usize::from(limit);
    let items = rows
        .into_iter()
        .take(usize::from(limit))
        .map(MessageAuthorRow::into_with_author)
        .collect();
    Ok(MessagesPage { items, has_more })
}

/// Load a non-deleted message by id (optionally scoped to a channel).
///
/// # Errors
///
/// Returns database errors.
pub async fn get_message(
    pool: &PgPool,
    message_id: Uuid,
    channel_id: Option<Uuid>,
) -> Result<Option<MessageWithAuthor>, AuthError> {
    let row = sqlx::query_as::<_, MessageAuthorRow>(
        r"
        SELECT m.id, m.channel_id, m.community_id, m.author_id, m.content, m.nonce,
               m.referenced_message_id, m.created_at, m.edited_at, m.deleted_at,
               COALESCE(NULLIF(p.display_name, ''), 'Member') AS author_display_name,
               r.id AS reply_id,
               r.author_id AS reply_author_id,
               r.content AS reply_content,
               r.deleted_at AS reply_deleted_at,
               COALESCE(NULLIF(rp.display_name, ''), 'Member') AS reply_author_display_name
        FROM messages m
        LEFT JOIN profiles p ON p.account_id = m.author_id
        LEFT JOIN messages r ON r.id = m.referenced_message_id
        LEFT JOIN profiles rp ON rp.account_id = r.author_id
        WHERE m.id = $1
          AND m.deleted_at IS NULL
          AND ($2::uuid IS NULL OR m.channel_id = $2)
        ",
    )
    .bind(message_id)
    .bind(channel_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(MessageAuthorRow::into_with_author))
}

/// Update message content and bump `edited_at`.
///
/// # Errors
///
/// Returns [`AuthError::InvalidMessage`] for empty/oversized content, or database errors.
pub async fn update_message(
    pool: &PgPool,
    message_id: Uuid,
    content: &str,
) -> Result<MessageWithAuthor, AuthError> {
    let content = content.trim();
    if content.is_empty() || content.chars().count() > MESSAGE_CONTENT_MAX {
        return Err(AuthError::InvalidMessage);
    }
    let now = Utc::now();
    let result = sqlx::query(
        r"
        UPDATE messages
        SET content = $2, edited_at = $3
        WHERE id = $1 AND deleted_at IS NULL
        ",
    )
    .bind(message_id)
    .bind(content)
    .bind(now)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AuthError::Db(sqlx::Error::RowNotFound));
    }
    get_message(pool, message_id, None)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Soft-delete a message (`deleted_at = now`).
///
/// # Errors
///
/// Returns database errors. `Ok(false)` when the message was missing or already deleted.
pub async fn soft_delete_message(pool: &PgPool, message_id: Uuid) -> Result<bool, AuthError> {
    let now = Utc::now();
    let result = sqlx::query(
        r"
        UPDATE messages
        SET deleted_at = $2
        WHERE id = $1 AND deleted_at IS NULL
        ",
    )
    .bind(message_id)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

async fn validate_reply_target(
    pool: &PgPool,
    channel_id: Uuid,
    referenced_message_id: Uuid,
) -> Result<(), AuthError> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        r"
        SELECT id
        FROM messages
        WHERE id = $1
          AND channel_id = $2
          AND deleted_at IS NULL
        ",
    )
    .bind(referenced_message_id)
    .bind(channel_id)
    .fetch_optional(pool)
    .await?;
    if row.is_none() {
        return Err(AuthError::InvalidReplyTarget);
    }
    Ok(())
}

async fn get_message_by_nonce(
    pool: &PgPool,
    channel_id: Uuid,
    author_id: Uuid,
    nonce: &str,
) -> Result<Option<MessageWithAuthor>, AuthError> {
    let row = sqlx::query_as::<_, MessageAuthorRow>(
        r"
        SELECT m.id, m.channel_id, m.community_id, m.author_id, m.content, m.nonce,
               m.referenced_message_id, m.created_at, m.edited_at, m.deleted_at,
               COALESCE(NULLIF(p.display_name, ''), 'Member') AS author_display_name,
               r.id AS reply_id,
               r.author_id AS reply_author_id,
               r.content AS reply_content,
               r.deleted_at AS reply_deleted_at,
               COALESCE(NULLIF(rp.display_name, ''), 'Member') AS reply_author_display_name
        FROM messages m
        LEFT JOIN profiles p ON p.account_id = m.author_id
        LEFT JOIN messages r ON r.id = m.referenced_message_id
        LEFT JOIN profiles rp ON rp.account_id = r.author_id
        WHERE m.channel_id = $1
          AND m.author_id = $2
          AND m.nonce = $3
          AND m.deleted_at IS NULL
        ",
    )
    .bind(channel_id)
    .bind(author_id)
    .bind(nonce)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(MessageAuthorRow::into_with_author))
}

const MESSAGE_SELECT_BASE: &str = r"
            SELECT m.id, m.channel_id, m.community_id, m.author_id, m.content, m.nonce,
                   m.referenced_message_id, m.created_at, m.edited_at, m.deleted_at,
                   COALESCE(NULLIF(p.display_name, ''), 'Member') AS author_display_name,
                   r.id AS reply_id,
                   r.author_id AS reply_author_id,
                   r.content AS reply_content,
                   r.deleted_at AS reply_deleted_at,
                   COALESCE(NULLIF(rp.display_name, ''), 'Member') AS reply_author_display_name
            FROM messages m
            LEFT JOIN profiles p ON p.account_id = m.author_id
            LEFT JOIN messages r ON r.id = m.referenced_message_id
            LEFT JOIN profiles rp ON rp.account_id = r.author_id
            WHERE m.channel_id = $1
              AND m.deleted_at IS NULL
        ";

fn excerpt(content: &str, deleted: bool) -> String {
    if deleted {
        return "Original message was deleted".to_owned();
    }
    let trimmed = content.trim();
    let count = trimmed.chars().count();
    if count <= REPLY_EXCERPT_MAX {
        return trimmed.to_owned();
    }
    let mut out: String = trimmed
        .chars()
        .take(REPLY_EXCERPT_MAX.saturating_sub(1))
        .collect();
    out.push('…');
    out
}

#[derive(Debug, FromRow)]
struct MessageAuthorRow {
    id: Uuid,
    channel_id: Uuid,
    community_id: Uuid,
    author_id: Uuid,
    content: String,
    nonce: Option<String>,
    referenced_message_id: Option<Uuid>,
    created_at: chrono::DateTime<Utc>,
    edited_at: Option<chrono::DateTime<Utc>>,
    deleted_at: Option<chrono::DateTime<Utc>>,
    author_display_name: String,
    reply_id: Option<Uuid>,
    reply_author_id: Option<Uuid>,
    reply_content: Option<String>,
    reply_deleted_at: Option<chrono::DateTime<Utc>>,
    reply_author_display_name: Option<String>,
}

impl MessageAuthorRow {
    fn into_with_author(self) -> MessageWithAuthor {
        let reply = match (
            self.reply_id,
            self.reply_author_id,
            self.reply_content,
            self.reply_author_display_name,
        ) {
            (Some(message_id), Some(author_id), Some(content), Some(author_display_name)) => {
                let deleted = self.reply_deleted_at.is_some();
                Some(MessageReplyPreview {
                    message_id,
                    author_id,
                    author_display_name: if deleted {
                        "Unknown".to_owned()
                    } else {
                        author_display_name
                    },
                    excerpt: excerpt(&content, deleted),
                    deleted,
                })
            }
            (Some(message_id), _, _, _) => Some(MessageReplyPreview {
                message_id,
                author_id: Uuid::nil(),
                author_display_name: "Unknown".to_owned(),
                excerpt: excerpt("", true),
                deleted: true,
            }),
            _ => None,
        };

        MessageWithAuthor {
            message: Message {
                id: self.id,
                channel_id: self.channel_id,
                community_id: self.community_id,
                author_id: self.author_id,
                content: self.content,
                nonce: self.nonce,
                referenced_message_id: self.referenced_message_id,
                created_at: self.created_at,
                edited_at: self.edited_at,
                deleted_at: self.deleted_at,
            },
            author_display_name: self.author_display_name,
            reply,
        }
    }
}
