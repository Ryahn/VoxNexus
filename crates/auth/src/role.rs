//! Community role persistence (F028).

use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use voxnexus_domain::CommunityRole;

use crate::community::get_community;
use crate::AuthError;

/// Input for creating a role.
#[derive(Debug, Clone)]
pub struct CreateRoleInput {
    pub name: String,
    pub color: String,
    pub hoist: bool,
    pub mentionable: bool,
    pub permissions: Value,
}

/// Partial update for a role.
#[derive(Debug, Clone, Default)]
pub struct RolePatch {
    pub name: Option<String>,
    pub position: Option<i32>,
    pub color: Option<String>,
    pub hoist: Option<bool>,
    pub mentionable: Option<bool>,
    pub permissions: Option<Value>,
}

/// Role-management authority for an account in a community.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleActor {
    pub is_owner: bool,
    pub can_manage_roles: bool,
    pub max_position: i32,
}

/// Insert the `@everyone` role for a new community inside a transaction.
///
/// # Errors
///
/// Returns database errors.
pub async fn insert_everyone_role(
    tx: &mut Transaction<'_, Postgres>,
    community_id: Uuid,
    now: DateTime<Utc>,
) -> Result<Uuid, AuthError> {
    let id = Uuid::now_v7();
    sqlx::query(
        r"
        INSERT INTO community_roles (
            id, community_id, name, position, color, hoist, mentionable, permissions,
            is_everyone, created_at, updated_at
        )
        VALUES ($1, $2, '@everyone', 0, '141 152 173', FALSE, FALSE, '{}', TRUE, $3, $3)
        ",
    )
    .bind(id)
    .bind(community_id)
    .bind(now)
    .execute(&mut **tx)
    .await?;
    Ok(id)
}

/// Create a custom role (not `@everyone`).
///
/// # Errors
///
/// Returns database errors or unique name violations.
pub async fn create_role(
    pool: &PgPool,
    community_id: Uuid,
    input: CreateRoleInput,
) -> Result<CommunityRole, AuthError> {
    let id = Uuid::now_v7();
    let now = Utc::now();
    let position = next_role_position(pool, community_id).await?;
    let result = sqlx::query(
        r"
        INSERT INTO community_roles (
            id, community_id, name, position, color, hoist, mentionable, permissions,
            is_everyone, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, FALSE, $9, $9)
        ",
    )
    .bind(id)
    .bind(community_id)
    .bind(&input.name)
    .bind(position)
    .bind(&input.color)
    .bind(input.hoist)
    .bind(input.mentionable)
    .bind(&input.permissions)
    .bind(now)
    .execute(pool)
    .await;
    if let Err(error) = result {
        if is_unique_violation(&error) {
            return Err(AuthError::RoleNameTaken);
        }
        return Err(error.into());
    }
    get_role(pool, id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Fetch a role by id.
///
/// # Errors
///
/// Returns database errors.
pub async fn get_role(pool: &PgPool, role_id: Uuid) -> Result<Option<CommunityRole>, AuthError> {
    let row = sqlx::query_as::<_, RoleRow>(
        r"
        SELECT id, community_id, name, position, color, hoist, mentionable, permissions,
               is_everyone, created_at, updated_at
        FROM community_roles
        WHERE id = $1
        ",
    )
    .bind(role_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(RoleRow::into_role))
}

/// List roles in a community ordered by position (ascending).
///
/// # Errors
///
/// Returns database errors.
pub async fn list_roles(
    pool: &PgPool,
    community_id: Uuid,
) -> Result<Vec<CommunityRole>, AuthError> {
    let rows = sqlx::query_as::<_, RoleRow>(
        r"
        SELECT id, community_id, name, position, color, hoist, mentionable, permissions,
               is_everyone, created_at, updated_at
        FROM community_roles
        WHERE community_id = $1
        ORDER BY position ASC, created_at ASC
        ",
    )
    .bind(community_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(RoleRow::into_role).collect())
}

/// Update role fields.
///
/// # Errors
///
/// Returns database errors.
pub async fn update_role(
    pool: &PgPool,
    role_id: Uuid,
    patch: RolePatch,
) -> Result<CommunityRole, AuthError> {
    let current = get_role(pool, role_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    let name = patch.name.unwrap_or(current.name);
    let position = patch.position.unwrap_or(current.position);
    let color = patch.color.unwrap_or(current.color);
    let hoist = patch.hoist.unwrap_or(current.hoist);
    let mentionable = patch.mentionable.unwrap_or(current.mentionable);
    let permissions = patch.permissions.unwrap_or(current.permissions);
    let now = Utc::now();
    sqlx::query(
        r"
        UPDATE community_roles
        SET name = $2, position = $3, color = $4, hoist = $5, mentionable = $6,
            permissions = $7, updated_at = $8
        WHERE id = $1
        ",
    )
    .bind(role_id)
    .bind(&name)
    .bind(position)
    .bind(&color)
    .bind(hoist)
    .bind(mentionable)
    .bind(&permissions)
    .bind(now)
    .execute(pool)
    .await?;
    get_role(pool, role_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Delete a custom role (not `@everyone`).
///
/// # Errors
///
/// Returns database errors.
pub async fn delete_role(pool: &PgPool, role_id: Uuid) -> Result<bool, AuthError> {
    let role = get_role(pool, role_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    if role.is_everyone {
        return Err(AuthError::EveryoneRoleImmutable);
    }
    let result = sqlx::query("DELETE FROM community_roles WHERE id = $1")
        .bind(role_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Clone a role shell (permissions and flags, not assignments).
///
/// # Errors
///
/// Returns database errors.
pub async fn clone_role(pool: &PgPool, role_id: Uuid) -> Result<CommunityRole, AuthError> {
    let source = get_role(pool, role_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    let name = if source.name.len() + 6 <= 100 {
        format!("{} copy", source.name)
    } else {
        format!("{}… copy", source.name.chars().take(94).collect::<String>())
    };
    create_role(
        pool,
        source.community_id,
        CreateRoleInput {
            name,
            color: source.color.clone(),
            hoist: source.hoist,
            mentionable: source.mentionable,
            permissions: source.permissions.clone(),
        },
    )
    .await
}

/// List roles assigned to a member (excludes implicit `@everyone`).
///
/// # Errors
///
/// Returns database errors.
pub async fn list_member_roles(
    pool: &PgPool,
    community_id: Uuid,
    account_id: Uuid,
) -> Result<Vec<CommunityRole>, AuthError> {
    let rows = sqlx::query_as::<_, RoleRow>(
        r"
        SELECT r.id, r.community_id, r.name, r.position, r.color, r.hoist, r.mentionable,
               r.permissions, r.is_everyone, r.created_at, r.updated_at
        FROM community_role_assignments a
        INNER JOIN community_roles r ON r.id = a.role_id
        WHERE a.community_id = $1 AND a.account_id = $2 AND r.is_everyone = FALSE
        ORDER BY r.position ASC, r.created_at ASC
        ",
    )
    .bind(community_id)
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(RoleRow::into_role).collect())
}

/// Assign a custom role to a member.
///
/// # Errors
///
/// Returns database errors.
pub async fn assign_role(
    pool: &PgPool,
    community_id: Uuid,
    account_id: Uuid,
    role_id: Uuid,
) -> Result<(), AuthError> {
    let role = get_role(pool, role_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    if role.community_id != community_id || role.is_everyone {
        return Err(AuthError::RoleScopeMismatch);
    }
    let now = Utc::now();
    sqlx::query(
        r"
        INSERT INTO community_role_assignments (community_id, account_id, role_id, assigned_at)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (account_id, role_id) DO NOTHING
        ",
    )
    .bind(community_id)
    .bind(account_id)
    .bind(role_id)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove a custom role from a member.
///
/// # Errors
///
/// Returns database errors.
pub async fn remove_role_assignment(
    pool: &PgPool,
    community_id: Uuid,
    account_id: Uuid,
    role_id: Uuid,
) -> Result<bool, AuthError> {
    let result = sqlx::query(
        r"
        DELETE FROM community_role_assignments
        WHERE community_id = $1 AND account_id = $2 AND role_id = $3
        ",
    )
    .bind(community_id)
    .bind(account_id)
    .bind(role_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Resolve role-management authority for an account.
///
/// # Errors
///
/// Returns database errors.
pub async fn role_actor(
    pool: &PgPool,
    community_id: Uuid,
    account_id: Uuid,
) -> Result<RoleActor, AuthError> {
    let community = get_community(pool, community_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    if community.owner_account_id == account_id {
        return Ok(RoleActor {
            is_owner: true,
            can_manage_roles: true,
            max_position: i32::MAX,
        });
    }
    let rows = sqlx::query_as::<_, RoleRow>(
        r"
        SELECT r.id, r.community_id, r.name, r.position, r.color, r.hoist, r.mentionable,
               r.permissions, r.is_everyone, r.created_at, r.updated_at
        FROM community_role_assignments a
        INNER JOIN community_roles r ON r.id = a.role_id
        WHERE a.community_id = $1 AND a.account_id = $2 AND r.is_everyone = FALSE
        ",
    )
    .bind(community_id)
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    let mut max_position = -1;
    let mut can_manage_roles = false;
    for row in rows {
        let role = row.into_role();
        if role.position > max_position {
            max_position = role.position;
        }
        if permissions_manage_roles(&role.permissions) {
            can_manage_roles = true;
        }
    }
    Ok(RoleActor {
        is_owner: false,
        can_manage_roles,
        max_position: max_position.max(0),
    })
}

/// Whether `actor` may manage a role at `target_position`.
#[must_use]
pub fn can_manage_role_position(actor: RoleActor, target_position: i32) -> bool {
    if actor.is_owner {
        return true;
    }
    if !actor.can_manage_roles {
        return false;
    }
    target_position < actor.max_position
}

/// Whether permissions JSON grants role management.
#[must_use]
pub fn permissions_manage_roles(permissions: &Value) -> bool {
    permissions
        .get("manage_roles")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

/// Build permissions JSON with `manage_roles` flag.
#[must_use]
pub fn permissions_with_manage_roles(manage_roles: bool) -> Value {
    json!({ "manage_roles": manage_roles })
}

async fn next_role_position(pool: &PgPool, community_id: Uuid) -> Result<i32, AuthError> {
    let max: Option<i32> = sqlx::query_scalar(
        r"
        SELECT MAX(position) FROM community_roles
        WHERE community_id = $1 AND is_everyone = FALSE
        ",
    )
    .bind(community_id)
    .fetch_one(pool)
    .await?;
    Ok(max.map_or(1, |value| value.saturating_add(1)))
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(
        error,
        sqlx::Error::Database(db) if db.code().as_deref() == Some("23505")
    )
}

#[derive(Debug, sqlx::FromRow)]
struct RoleRow {
    id: Uuid,
    community_id: Uuid,
    name: String,
    position: i32,
    color: String,
    hoist: bool,
    mentionable: bool,
    permissions: Value,
    is_everyone: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl RoleRow {
    fn into_role(self) -> CommunityRole {
        CommunityRole {
            id: self.id,
            community_id: self.community_id,
            name: self.name,
            position: self.position,
            color: self.color,
            hoist: self.hoist,
            mentionable: self.mentionable,
            permissions: self.permissions,
            is_everyone: self.is_everyone,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
