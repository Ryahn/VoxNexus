//! Organizational role groups (F030-A).

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use voxnexus_domain::CommunityRoleGroup;

use crate::AuthError;

/// Create a role group.
///
/// # Errors
///
/// Returns database errors or unique name violations.
pub async fn create_role_group(
    pool: &PgPool,
    community_id: Uuid,
    name: String,
) -> Result<CommunityRoleGroup, AuthError> {
    let id = Uuid::now_v7();
    let now = Utc::now();
    let display_order = next_group_order(pool, community_id).await?;
    let result = sqlx::query(
        r"
        INSERT INTO community_role_groups (id, community_id, name, display_order, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $5)
        ",
    )
    .bind(id)
    .bind(community_id)
    .bind(&name)
    .bind(display_order)
    .bind(now)
    .execute(pool)
    .await;
    if let Err(error) = result {
        if is_unique(&error) {
            return Err(AuthError::RoleGroupNameTaken);
        }
        return Err(error.into());
    }
    get_role_group(pool, id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Fetch a role group.
///
/// # Errors
///
/// Returns database errors.
pub async fn get_role_group(
    pool: &PgPool,
    group_id: Uuid,
) -> Result<Option<CommunityRoleGroup>, AuthError> {
    let row = sqlx::query_as::<_, GroupRow>(
        r"
        SELECT id, community_id, name, display_order, created_at, updated_at
        FROM community_role_groups
        WHERE id = $1
        ",
    )
    .bind(group_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(GroupRow::into_group))
}

/// List role groups for a community.
///
/// # Errors
///
/// Returns database errors.
pub async fn list_role_groups(
    pool: &PgPool,
    community_id: Uuid,
) -> Result<Vec<CommunityRoleGroup>, AuthError> {
    let rows = sqlx::query_as::<_, GroupRow>(
        r"
        SELECT id, community_id, name, display_order, created_at, updated_at
        FROM community_role_groups
        WHERE community_id = $1
        ORDER BY display_order ASC, created_at ASC
        ",
    )
    .bind(community_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(GroupRow::into_group).collect())
}

/// Rename or reorder a group.
///
/// # Errors
///
/// Returns database errors.
pub async fn update_role_group(
    pool: &PgPool,
    group_id: Uuid,
    name: Option<String>,
    display_order: Option<i32>,
) -> Result<CommunityRoleGroup, AuthError> {
    let current = get_role_group(pool, group_id)
        .await?
        .ok_or(AuthError::RoleGroupNotFound)?;
    let name = name.unwrap_or(current.name);
    let display_order = display_order.unwrap_or(current.display_order);
    let now = Utc::now();
    let result = sqlx::query(
        r"
        UPDATE community_role_groups
        SET name = $2, display_order = $3, updated_at = $4
        WHERE id = $1
        ",
    )
    .bind(group_id)
    .bind(&name)
    .bind(display_order)
    .bind(now)
    .execute(pool)
    .await;
    if let Err(error) = result {
        if is_unique(&error) {
            return Err(AuthError::RoleGroupNameTaken);
        }
        return Err(error.into());
    }
    get_role_group(pool, group_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Delete a group (roles become ungrouped).
///
/// # Errors
///
/// Returns database errors.
pub async fn delete_role_group(pool: &PgPool, group_id: Uuid) -> Result<bool, AuthError> {
    let result = sqlx::query("DELETE FROM community_role_groups WHERE id = $1")
        .bind(group_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

async fn next_group_order(pool: &PgPool, community_id: Uuid) -> Result<i32, AuthError> {
    let max: Option<i32> = sqlx::query_scalar(
        r"
        SELECT MAX(display_order) FROM community_role_groups WHERE community_id = $1
        ",
    )
    .bind(community_id)
    .fetch_one(pool)
    .await?;
    Ok(max.map_or(0, |value| value.saturating_add(1)))
}

fn is_unique(error: &sqlx::Error) -> bool {
    matches!(
        error,
        sqlx::Error::Database(db) if db.code().as_deref() == Some("23505")
    )
}

#[derive(Debug, sqlx::FromRow)]
struct GroupRow {
    id: Uuid,
    community_id: Uuid,
    name: String,
    display_order: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl GroupRow {
    fn into_group(self) -> CommunityRoleGroup {
        CommunityRoleGroup {
            id: self.id,
            community_id: self.community_id,
            name: self.name,
            display_order: self.display_order,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
