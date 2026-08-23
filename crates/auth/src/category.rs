//! Channel category persistence (F026).

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use voxnexus_domain::ChannelCategory;

use crate::space::get_space;
use crate::AuthError;

/// Input for creating a category.
#[derive(Debug, Clone)]
pub struct CreateCategoryInput {
    pub name: String,
    pub space_id: Option<Uuid>,
}

/// Partial update for a category.
#[derive(Debug, Clone, Default)]
pub struct CategoryPatch {
    pub name: Option<String>,
    pub position: Option<i32>,
    pub space_id: Option<Option<Uuid>>,
}

/// Create a category in `community_id` (community-level or space-scoped).
///
/// # Errors
///
/// Returns database errors if `space_id` does not belong to `community_id`.
pub async fn create_category(
    pool: &PgPool,
    community_id: Uuid,
    input: CreateCategoryInput,
) -> Result<ChannelCategory, AuthError> {
    if let Some(space_id) = input.space_id {
        let space = get_space(pool, space_id)
            .await?
            .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
        if space.community_id != community_id {
            return Err(AuthError::CategoryScopeMismatch);
        }
    }
    let id = Uuid::now_v7();
    let now = Utc::now();
    let position = next_position(pool, community_id, input.space_id).await?;
    sqlx::query(
        r"
        INSERT INTO categories (id, community_id, space_id, name, position, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $6)
        ",
    )
    .bind(id)
    .bind(community_id)
    .bind(input.space_id)
    .bind(&input.name)
    .bind(position)
    .bind(now)
    .execute(pool)
    .await?;
    get_category(pool, id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Fetch a category by id.
///
/// # Errors
///
/// Returns database errors.
pub async fn get_category(
    pool: &PgPool,
    category_id: Uuid,
) -> Result<Option<ChannelCategory>, AuthError> {
    let row = sqlx::query_as::<_, CategoryRow>(
        r"
        SELECT id, community_id, space_id, name, position, created_at, updated_at
        FROM categories
        WHERE id = $1
        ",
    )
    .bind(category_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(CategoryRow::into_category))
}

/// List categories for a community scope (community-level or one space).
///
/// # Errors
///
/// Returns database errors.
pub async fn list_categories(
    pool: &PgPool,
    community_id: Uuid,
    space_id: Option<Uuid>,
) -> Result<Vec<ChannelCategory>, AuthError> {
    let rows = if let Some(space_id) = space_id {
        sqlx::query_as::<_, CategoryRow>(
            r"
            SELECT id, community_id, space_id, name, position, created_at, updated_at
            FROM categories
            WHERE community_id = $1 AND space_id = $2
            ORDER BY position ASC, created_at ASC
            ",
        )
        .bind(community_id)
        .bind(space_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, CategoryRow>(
            r"
            SELECT id, community_id, space_id, name, position, created_at, updated_at
            FROM categories
            WHERE community_id = $1 AND space_id IS NULL
            ORDER BY position ASC, created_at ASC
            ",
        )
        .bind(community_id)
        .fetch_all(pool)
        .await?
    };
    Ok(rows.into_iter().map(CategoryRow::into_category).collect())
}

/// Update category fields. Cannot move to a space in another community.
///
/// # Errors
///
/// Returns [`AuthError::CategoryScopeMismatch`] when `space_id` crosses communities.
pub async fn update_category(
    pool: &PgPool,
    category_id: Uuid,
    patch: CategoryPatch,
) -> Result<ChannelCategory, AuthError> {
    let current = get_category(pool, category_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    let name = patch.name.unwrap_or(current.name);
    let position = patch.position.unwrap_or(current.position);
    let space_id = match patch.space_id {
        Some(value) => value,
        None => current.space_id,
    };
    if let Some(target_space_id) = space_id {
        let space = get_space(pool, target_space_id)
            .await?
            .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
        if space.community_id != current.community_id {
            return Err(AuthError::CategoryScopeMismatch);
        }
    }
    let now = Utc::now();
    sqlx::query(
        r"
        UPDATE categories
        SET name = $2, space_id = $3, position = $4, updated_at = $5
        WHERE id = $1
        ",
    )
    .bind(category_id)
    .bind(&name)
    .bind(space_id)
    .bind(position)
    .bind(now)
    .execute(pool)
    .await?;
    get_category(pool, category_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Delete a category.
///
/// # Errors
///
/// Returns database errors.
pub async fn delete_category(pool: &PgPool, category_id: Uuid) -> Result<bool, AuthError> {
    let result = sqlx::query("DELETE FROM categories WHERE id = $1")
        .bind(category_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

async fn next_position(
    pool: &PgPool,
    community_id: Uuid,
    space_id: Option<Uuid>,
) -> Result<i32, AuthError> {
    let max: Option<i32> = if let Some(space_id) = space_id {
        sqlx::query_scalar(
            r"
            SELECT MAX(position) FROM categories
            WHERE community_id = $1 AND space_id = $2
            ",
        )
        .bind(community_id)
        .bind(space_id)
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query_scalar(
            r"
            SELECT MAX(position) FROM categories
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
struct CategoryRow {
    id: Uuid,
    community_id: Uuid,
    space_id: Option<Uuid>,
    name: String,
    position: i32,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl CategoryRow {
    fn into_category(self) -> ChannelCategory {
        ChannelCategory {
            id: self.id,
            community_id: self.community_id,
            space_id: self.space_id,
            name: self.name,
            position: self.position,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
