//! Space persistence (F022).

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use voxnexus_domain::{Space, SpaceVisibility};

use crate::AuthError;

/// Input for creating a Space.
#[derive(Debug, Clone)]
pub struct CreateSpaceInput {
    pub name: String,
    pub description: String,
    pub topic: String,
    pub game: String,
    pub visibility: SpaceVisibility,
}

/// Partial update for a Space.
#[derive(Debug, Clone, Default)]
pub struct SpacePatch {
    pub name: Option<String>,
    pub description: Option<String>,
    pub topic: Option<String>,
    pub game: Option<String>,
    pub visibility: Option<SpaceVisibility>,
    pub position: Option<i32>,
}

/// Create a Space in `community_id`. Spaces cannot nest (no parent column).
///
/// # Errors
///
/// Returns database errors.
pub async fn create_space(
    pool: &PgPool,
    community_id: Uuid,
    input: CreateSpaceInput,
) -> Result<Space, AuthError> {
    let id = Uuid::now_v7();
    let now = Utc::now();
    let position = next_position(pool, community_id).await?;
    sqlx::query(
        r"
        INSERT INTO spaces (
            id, community_id, name, description, topic, game, visibility,
            position, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
        ",
    )
    .bind(id)
    .bind(community_id)
    .bind(&input.name)
    .bind(&input.description)
    .bind(&input.topic)
    .bind(&input.game)
    .bind(input.visibility.as_str())
    .bind(position)
    .bind(now)
    .execute(pool)
    .await?;
    get_space(pool, id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Fetch a Space by id.
///
/// # Errors
///
/// Returns database errors.
pub async fn get_space(pool: &PgPool, space_id: Uuid) -> Result<Option<Space>, AuthError> {
    let row = sqlx::query_as::<_, SpaceRow>(
        r"
        SELECT id, community_id, name, description, topic, game, visibility,
               icon_object_id, position, created_at, updated_at
        FROM spaces
        WHERE id = $1
        ",
    )
    .bind(space_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(SpaceRow::into_space))
}

/// List Spaces for a community ordered by position then created_at.
///
/// # Errors
///
/// Returns database errors.
pub async fn list_spaces(pool: &PgPool, community_id: Uuid) -> Result<Vec<Space>, AuthError> {
    let rows = sqlx::query_as::<_, SpaceRow>(
        r"
        SELECT id, community_id, name, description, topic, game, visibility,
               icon_object_id, position, created_at, updated_at
        FROM spaces
        WHERE community_id = $1
        ORDER BY position ASC, created_at ASC
        ",
    )
    .bind(community_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(SpaceRow::into_space).collect())
}

/// Update Space fields.
///
/// # Errors
///
/// Returns database errors.
pub async fn update_space(
    pool: &PgPool,
    space_id: Uuid,
    patch: SpacePatch,
) -> Result<Space, AuthError> {
    let current = get_space(pool, space_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    let now = Utc::now();
    let name = patch.name.unwrap_or(current.name);
    let description = patch.description.unwrap_or(current.description);
    let topic = patch.topic.unwrap_or(current.topic);
    let game = patch.game.unwrap_or(current.game);
    let visibility = patch.visibility.unwrap_or(current.visibility);
    let position = patch.position.unwrap_or(current.position);
    sqlx::query(
        r"
        UPDATE spaces
        SET name = $2, description = $3, topic = $4, game = $5,
            visibility = $6, position = $7, updated_at = $8
        WHERE id = $1
        ",
    )
    .bind(space_id)
    .bind(&name)
    .bind(&description)
    .bind(&topic)
    .bind(&game)
    .bind(visibility.as_str())
    .bind(position)
    .bind(now)
    .execute(pool)
    .await?;
    get_space(pool, space_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Delete a Space.
///
/// # Errors
///
/// Returns database errors.
pub async fn delete_space(pool: &PgPool, space_id: Uuid) -> Result<bool, AuthError> {
    let result = sqlx::query("DELETE FROM spaces WHERE id = $1")
        .bind(space_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

async fn next_position(pool: &PgPool, community_id: Uuid) -> Result<i32, AuthError> {
    let max: Option<i32> = sqlx::query_scalar(
        r"
        SELECT MAX(position) FROM spaces WHERE community_id = $1
        ",
    )
    .bind(community_id)
    .fetch_one(pool)
    .await?;
    Ok(max.map_or(0, |value| value.saturating_add(1)))
}

#[derive(Debug, sqlx::FromRow)]
struct SpaceRow {
    id: Uuid,
    community_id: Uuid,
    name: String,
    description: String,
    topic: String,
    game: String,
    visibility: String,
    icon_object_id: Option<Uuid>,
    position: i32,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl SpaceRow {
    fn into_space(self) -> Space {
        Space {
            id: self.id,
            community_id: self.community_id,
            name: self.name,
            description: self.description,
            topic: self.topic,
            game: self.game,
            visibility: SpaceVisibility::parse(&self.visibility).unwrap_or(SpaceVisibility::Open),
            icon_object_id: self.icon_object_id,
            position: self.position,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
