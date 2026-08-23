//! Channel persistence (F027).

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use voxnexus_domain::{Channel, ChannelType};

use crate::category::get_category;
use crate::space::get_space;
use crate::AuthError;

/// Input for creating a channel.
#[derive(Debug, Clone)]
pub struct CreateChannelInput {
    pub name: String,
    pub channel_type: ChannelType,
    pub topic: String,
    pub space_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub config: serde_json::Value,
}

/// Partial update for a channel.
#[derive(Debug, Clone, Default)]
pub struct ChannelPatch {
    pub name: Option<String>,
    pub topic: Option<String>,
    pub position: Option<i32>,
    pub space_id: Option<Option<Uuid>>,
    pub category_id: Option<Option<Uuid>>,
    pub config: Option<serde_json::Value>,
}

/// Create a channel in `community_id`.
///
/// # Errors
///
/// Returns [`AuthError::ChannelScopeMismatch`] when scope ids cross communities.
pub async fn create_channel(
    pool: &PgPool,
    community_id: Uuid,
    input: CreateChannelInput,
) -> Result<Channel, AuthError> {
    validate_scope(pool, community_id, input.space_id, input.category_id).await?;
    let id = Uuid::now_v7();
    let now = Utc::now();
    let position = next_position(pool, community_id, input.space_id, input.category_id).await?;
    sqlx::query(
        r"
        INSERT INTO channels (
            id, community_id, space_id, category_id, channel_type, name, topic,
            position, config, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
        ",
    )
    .bind(id)
    .bind(community_id)
    .bind(input.space_id)
    .bind(input.category_id)
    .bind(input.channel_type.as_str())
    .bind(&input.name)
    .bind(&input.topic)
    .bind(position)
    .bind(&input.config)
    .bind(now)
    .execute(pool)
    .await?;
    get_channel(pool, id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Fetch a channel by id.
///
/// # Errors
///
/// Returns database errors.
pub async fn get_channel(pool: &PgPool, channel_id: Uuid) -> Result<Option<Channel>, AuthError> {
    let row = sqlx::query_as::<_, ChannelRow>(
        r"
        SELECT id, community_id, space_id, category_id, channel_type, name, topic,
               position, archived_at, config, created_at, updated_at
        FROM channels
        WHERE id = $1
        ",
    )
    .bind(channel_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(ChannelRow::into_channel))
}

/// List channels in a community scope.
///
/// # Errors
///
/// Returns database errors.
#[allow(clippy::too_many_lines)]
pub async fn list_channels(
    pool: &PgPool,
    community_id: Uuid,
    space_id: Option<Uuid>,
    category_id: Option<Uuid>,
    include_archived: bool,
) -> Result<Vec<Channel>, AuthError> {
    let rows = if let Some(space_id) = space_id {
        if let Some(category_id) = category_id {
            if include_archived {
                sqlx::query_as::<_, ChannelRow>(
                    r"
                    SELECT id, community_id, space_id, category_id, channel_type, name, topic,
                           position, archived_at, config, created_at, updated_at
                    FROM channels
                    WHERE community_id = $1 AND space_id = $2 AND category_id = $3
                    ORDER BY position ASC, created_at ASC
                    ",
                )
                .bind(community_id)
                .bind(space_id)
                .bind(category_id)
                .fetch_all(pool)
                .await?
            } else {
                sqlx::query_as::<_, ChannelRow>(
                    r"
                    SELECT id, community_id, space_id, category_id, channel_type, name, topic,
                           position, archived_at, config, created_at, updated_at
                    FROM channels
                    WHERE community_id = $1 AND space_id = $2 AND category_id = $3
                      AND archived_at IS NULL
                    ORDER BY position ASC, created_at ASC
                    ",
                )
                .bind(community_id)
                .bind(space_id)
                .bind(category_id)
                .fetch_all(pool)
                .await?
            }
        } else if include_archived {
            sqlx::query_as::<_, ChannelRow>(
                r"
                SELECT id, community_id, space_id, category_id, channel_type, name, topic,
                       position, archived_at, config, created_at, updated_at
                FROM channels
                WHERE community_id = $1 AND space_id = $2
                ORDER BY position ASC, created_at ASC
                ",
            )
            .bind(community_id)
            .bind(space_id)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, ChannelRow>(
                r"
                SELECT id, community_id, space_id, category_id, channel_type, name, topic,
                       position, archived_at, config, created_at, updated_at
                FROM channels
                WHERE community_id = $1 AND space_id = $2 AND archived_at IS NULL
                ORDER BY position ASC, created_at ASC
                ",
            )
            .bind(community_id)
            .bind(space_id)
            .fetch_all(pool)
            .await?
        }
    } else if let Some(category_id) = category_id {
        if include_archived {
            sqlx::query_as::<_, ChannelRow>(
                r"
                SELECT id, community_id, space_id, category_id, channel_type, name, topic,
                       position, archived_at, config, created_at, updated_at
                FROM channels
                WHERE community_id = $1 AND space_id IS NULL AND category_id = $2
                ORDER BY position ASC, created_at ASC
                ",
            )
            .bind(community_id)
            .bind(category_id)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, ChannelRow>(
                r"
                SELECT id, community_id, space_id, category_id, channel_type, name, topic,
                       position, archived_at, config, created_at, updated_at
                FROM channels
                WHERE community_id = $1 AND space_id IS NULL AND category_id = $2
                  AND archived_at IS NULL
                ORDER BY position ASC, created_at ASC
                ",
            )
            .bind(community_id)
            .bind(category_id)
            .fetch_all(pool)
            .await?
        }
    } else if include_archived {
        sqlx::query_as::<_, ChannelRow>(
            r"
            SELECT id, community_id, space_id, category_id, channel_type, name, topic,
                   position, archived_at, config, created_at, updated_at
            FROM channels
            WHERE community_id = $1 AND space_id IS NULL
            ORDER BY position ASC, created_at ASC
            ",
        )
        .bind(community_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, ChannelRow>(
            r"
            SELECT id, community_id, space_id, category_id, channel_type, name, topic,
                   position, archived_at, config, created_at, updated_at
            FROM channels
            WHERE community_id = $1 AND space_id IS NULL AND archived_at IS NULL
            ORDER BY position ASC, created_at ASC
            ",
        )
        .bind(community_id)
        .fetch_all(pool)
        .await?
    };
    Ok(rows.into_iter().map(ChannelRow::into_channel).collect())
}

/// Update channel fields.
///
/// # Errors
///
/// Returns [`AuthError::ChannelScopeMismatch`] when scope ids cross communities.
pub async fn update_channel(
    pool: &PgPool,
    channel_id: Uuid,
    patch: ChannelPatch,
) -> Result<Channel, AuthError> {
    let current = get_channel(pool, channel_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    let name = patch.name.unwrap_or(current.name);
    let topic = patch.topic.unwrap_or(current.topic);
    let position = patch.position.unwrap_or(current.position);
    let space_id = match patch.space_id {
        Some(value) => value,
        None => current.space_id,
    };
    let category_id = match patch.category_id {
        Some(value) => value,
        None => current.category_id,
    };
    let config = patch.config.unwrap_or(current.config);
    validate_scope(pool, current.community_id, space_id, category_id).await?;
    let now = Utc::now();
    sqlx::query(
        r"
        UPDATE channels
        SET name = $2, topic = $3, position = $4, space_id = $5, category_id = $6,
            config = $7, updated_at = $8
        WHERE id = $1
        ",
    )
    .bind(channel_id)
    .bind(&name)
    .bind(&topic)
    .bind(position)
    .bind(space_id)
    .bind(category_id)
    .bind(&config)
    .bind(now)
    .execute(pool)
    .await?;
    get_channel(pool, channel_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Hard-delete a channel.
///
/// # Errors
///
/// Returns database errors.
pub async fn delete_channel(pool: &PgPool, channel_id: Uuid) -> Result<bool, AuthError> {
    let result = sqlx::query("DELETE FROM channels WHERE id = $1")
        .bind(channel_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Archive a channel (hide from default lists).
///
/// # Errors
///
/// Returns database errors.
pub async fn archive_channel(pool: &PgPool, channel_id: Uuid) -> Result<Channel, AuthError> {
    let now = Utc::now();
    sqlx::query(
        r"
        UPDATE channels
        SET archived_at = $2, updated_at = $2
        WHERE id = $1
        ",
    )
    .bind(channel_id)
    .bind(now)
    .execute(pool)
    .await?;
    get_channel(pool, channel_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Restore an archived channel.
///
/// # Errors
///
/// Returns database errors.
pub async fn restore_channel(pool: &PgPool, channel_id: Uuid) -> Result<Channel, AuthError> {
    let now = Utc::now();
    sqlx::query(
        r"
        UPDATE channels
        SET archived_at = NULL, updated_at = $2
        WHERE id = $1
        ",
    )
    .bind(channel_id)
    .bind(now)
    .execute(pool)
    .await?;
    get_channel(pool, channel_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Clone channel shell (name, type, topic, config) without messages.
///
/// # Errors
///
/// Returns database errors or scope mismatch errors.
pub async fn clone_channel(pool: &PgPool, channel_id: Uuid) -> Result<Channel, AuthError> {
    let source = get_channel(pool, channel_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    let name = if source.name.len() + 6 <= 100 {
        format!("{} copy", source.name)
    } else {
        format!("{}… copy", source.name.chars().take(94).collect::<String>())
    };
    create_channel(
        pool,
        source.community_id,
        CreateChannelInput {
            name,
            channel_type: source.channel_type,
            topic: source.topic.clone(),
            space_id: source.space_id,
            category_id: source.category_id,
            config: source.config.clone(),
        },
    )
    .await
}

async fn validate_scope(
    pool: &PgPool,
    community_id: Uuid,
    space_id: Option<Uuid>,
    category_id: Option<Uuid>,
) -> Result<(), AuthError> {
    if let Some(space_id) = space_id {
        let space = get_space(pool, space_id)
            .await?
            .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
        if space.community_id != community_id {
            return Err(AuthError::ChannelScopeMismatch);
        }
    }
    if let Some(category_id) = category_id {
        let category = get_category(pool, category_id)
            .await?
            .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
        if category.community_id != community_id {
            return Err(AuthError::ChannelScopeMismatch);
        }
        if category.space_id != space_id {
            return Err(AuthError::ChannelScopeMismatch);
        }
    }
    Ok(())
}

async fn next_position(
    pool: &PgPool,
    community_id: Uuid,
    space_id: Option<Uuid>,
    category_id: Option<Uuid>,
) -> Result<i32, AuthError> {
    let max: Option<i32> = if let Some(space_id) = space_id {
        if let Some(category_id) = category_id {
            sqlx::query_scalar(
                r"
                SELECT MAX(position) FROM channels
                WHERE community_id = $1 AND space_id = $2 AND category_id = $3
                ",
            )
            .bind(community_id)
            .bind(space_id)
            .bind(category_id)
            .fetch_one(pool)
            .await?
        } else {
            sqlx::query_scalar(
                r"
                SELECT MAX(position) FROM channels
                WHERE community_id = $1 AND space_id = $2
                ",
            )
            .bind(community_id)
            .bind(space_id)
            .fetch_one(pool)
            .await?
        }
    } else if let Some(category_id) = category_id {
        sqlx::query_scalar(
            r"
            SELECT MAX(position) FROM channels
            WHERE community_id = $1 AND space_id IS NULL AND category_id = $2
            ",
        )
        .bind(community_id)
        .bind(category_id)
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query_scalar(
            r"
            SELECT MAX(position) FROM channels
            WHERE community_id = $1 AND space_id IS NULL
            ",
        )
        .bind(community_id)
        .fetch_one(pool)
        .await?
    };
    Ok(max.map_or(0, |value| value.saturating_add(1)))
}

#[derive(Debug, sqlx::FromRow)]
struct ChannelRow {
    id: Uuid,
    community_id: Uuid,
    space_id: Option<Uuid>,
    category_id: Option<Uuid>,
    channel_type: String,
    name: String,
    topic: String,
    position: i32,
    archived_at: Option<chrono::DateTime<Utc>>,
    config: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl ChannelRow {
    fn into_channel(self) -> Channel {
        let channel_type = ChannelType::parse(&self.channel_type).unwrap_or(ChannelType::Text);
        Channel {
            id: self.id,
            community_id: self.community_id,
            space_id: self.space_id,
            category_id: self.category_id,
            channel_type,
            name: self.name,
            topic: self.topic,
            position: self.position,
            archived_at: self.archived_at,
            config: self.config,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
