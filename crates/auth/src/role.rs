//! Community role persistence (F028 / F030-A).

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use voxnexus_domain::CommunityRole;
use voxnexus_permissions::{
    default_everyone_permissions_json, empty_role_permissions_json, parse_role_permissions,
    permissions_with_manage_roles as perm_json_manage_roles, Family,
};

use crate::community::get_community;
use crate::AuthError;

const ROLE_SELECT: &str = r"
    id, community_id, name, position, weight, group_id, color, hoist, mentionable,
    permissions, is_everyone, short_tag, icon_emoji, icon_object_key, gradient,
    role_card, created_at, updated_at
";

/// Input for creating a role.
#[derive(Debug, Clone)]
pub struct CreateRoleInput {
    pub name: String,
    pub color: String,
    pub hoist: bool,
    pub mentionable: bool,
    pub permissions: Value,
    pub weight: Option<i32>,
    pub group_id: Option<Uuid>,
    pub short_tag: String,
    pub icon_emoji: Option<String>,
    pub gradient: Option<String>,
    pub role_card: Value,
}

/// Partial update for a role.
#[derive(Debug, Clone, Default)]
pub struct RolePatch {
    pub name: Option<String>,
    pub position: Option<i32>,
    pub weight: Option<i32>,
    pub group_id: Option<Option<Uuid>>,
    pub color: Option<String>,
    pub hoist: Option<bool>,
    pub mentionable: Option<bool>,
    pub permissions: Option<Value>,
    pub short_tag: Option<String>,
    pub icon_emoji: Option<Option<String>>,
    pub icon_object_key: Option<Option<String>>,
    pub gradient: Option<Option<String>>,
    pub role_card: Option<Value>,
}

/// Role-management authority for an account in a community.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleActor {
    pub is_owner: bool,
    pub can_manage_roles: bool,
    /// Lowest weight among actor roles (lower = higher priority). Owners use 0.
    pub min_weight: i32,
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
            id, community_id, name, position, weight, color, hoist, mentionable, permissions,
            is_everyone, short_tag, role_card, created_at, updated_at
        )
        VALUES ($1, $2, '@everyone', 0, 1000, '141 152 173', FALSE, FALSE, $3, TRUE, '', '{}'::jsonb, $4, $4)
        ",
    )
    .bind(id)
    .bind(community_id)
    .bind(default_everyone_permissions_json())
    .bind(now)
    .execute(&mut **tx)
    .await?;
    Ok(id)
}

/// Create a custom role (not `@everyone`).
///
/// # Errors
///
/// Returns database errors or unique name/weight violations.
pub async fn create_role(
    pool: &PgPool,
    community_id: Uuid,
    input: CreateRoleInput,
) -> Result<CommunityRole, AuthError> {
    let id = Uuid::now_v7();
    let now = Utc::now();
    let position = next_role_position(pool, community_id).await?;
    let weight = match input.weight {
        Some(w) => validate_weight(w)?,
        None => next_role_weight(pool, community_id).await?,
    };
    if let Some(group_id) = input.group_id {
        ensure_group_in_community(pool, community_id, group_id).await?;
    }
    let result = sqlx::query(
        r"
        INSERT INTO community_roles (
            id, community_id, name, position, weight, group_id, color, hoist, mentionable,
            permissions, is_everyone, short_tag, icon_emoji, gradient, role_card,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, FALSE, $11, $12, $13, $14, $15, $15)
        ",
    )
    .bind(id)
    .bind(community_id)
    .bind(&input.name)
    .bind(position)
    .bind(weight)
    .bind(input.group_id)
    .bind(&input.color)
    .bind(input.hoist)
    .bind(input.mentionable)
    .bind(&input.permissions)
    .bind(&input.short_tag)
    .bind(&input.icon_emoji)
    .bind(&input.gradient)
    .bind(&input.role_card)
    .bind(now)
    .execute(pool)
    .await;
    if let Err(error) = result {
        return Err(map_unique(error));
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
    let sql = format!("SELECT {ROLE_SELECT} FROM community_roles WHERE id = $1");
    let row = sqlx::query_as::<_, RoleRow>(&sql)
        .bind(role_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(RoleRow::into_role))
}

/// List roles in a community ordered by display position.
///
/// # Errors
///
/// Returns database errors.
pub async fn list_roles(
    pool: &PgPool,
    community_id: Uuid,
) -> Result<Vec<CommunityRole>, AuthError> {
    let sql = format!(
        "SELECT {ROLE_SELECT} FROM community_roles WHERE community_id = $1 ORDER BY position ASC, created_at ASC"
    );
    let rows = sqlx::query_as::<_, RoleRow>(&sql)
        .bind(community_id)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(RoleRow::into_role).collect())
}

/// Update role fields.
///
/// # Errors
///
/// Returns database errors or unique/weight violations.
pub async fn update_role(
    pool: &PgPool,
    role_id: Uuid,
    patch: RolePatch,
) -> Result<CommunityRole, AuthError> {
    let current = get_role(pool, role_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    if current.is_everyone
        && (patch.name.is_some() || patch.weight.is_some() || patch.group_id.is_some())
    {
        return Err(AuthError::EveryoneRoleImmutable);
    }
    let name = patch.name.unwrap_or(current.name);
    let position = patch.position.unwrap_or(current.position);
    let weight = match patch.weight {
        Some(_w) if current.is_everyone => 1000,
        Some(w) => validate_weight(w)?,
        None => current.weight,
    };
    let group_id = match patch.group_id {
        Some(value) => value,
        None => current.group_id,
    };
    if let Some(gid) = group_id {
        ensure_group_in_community(pool, current.community_id, gid).await?;
    }
    let color = patch.color.unwrap_or(current.color);
    let hoist = patch.hoist.unwrap_or(current.hoist);
    let mentionable = patch.mentionable.unwrap_or(current.mentionable);
    let permissions = patch.permissions.unwrap_or(current.permissions);
    let short_tag = patch.short_tag.unwrap_or(current.short_tag);
    let icon_emoji = match patch.icon_emoji {
        Some(value) => value,
        None => current.icon_emoji,
    };
    let icon_object_key = match patch.icon_object_key {
        Some(value) => value,
        None => current.icon_object_key,
    };
    let gradient = match patch.gradient {
        Some(value) => value,
        None => current.gradient,
    };
    let role_card = patch.role_card.unwrap_or(current.role_card);
    let now = Utc::now();
    let result = sqlx::query(
        r"
        UPDATE community_roles
        SET name = $2, position = $3, weight = $4, group_id = $5, color = $6, hoist = $7,
            mentionable = $8, permissions = $9, short_tag = $10, icon_emoji = $11,
            icon_object_key = $12, gradient = $13, role_card = $14, updated_at = $15
        WHERE id = $1
        ",
    )
    .bind(role_id)
    .bind(&name)
    .bind(position)
    .bind(weight)
    .bind(group_id)
    .bind(&color)
    .bind(hoist)
    .bind(mentionable)
    .bind(&permissions)
    .bind(&short_tag)
    .bind(&icon_emoji)
    .bind(&icon_object_key)
    .bind(&gradient)
    .bind(&role_card)
    .bind(now)
    .execute(pool)
    .await;
    if let Err(error) = result {
        return Err(map_unique(error));
    }
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
            weight: None,
            group_id: source.group_id,
            short_tag: source.short_tag.clone(),
            icon_emoji: source.icon_emoji.clone(),
            gradient: source.gradient.clone(),
            role_card: source.role_card.clone(),
        },
    )
    .await
}

/// List roles that contribute grants for a member (`@everyone` + assignments).
///
/// # Errors
///
/// Returns database errors.
pub async fn member_roles_for_grants(
    pool: &PgPool,
    community_id: Uuid,
    account_id: Uuid,
) -> Result<Vec<CommunityRole>, AuthError> {
    let everyone = get_everyone_role(pool, community_id).await?;
    let assigned = list_member_roles(pool, community_id, account_id).await?;
    Ok(match everyone {
        Some(role) => {
            let mut roles = vec![role];
            roles.extend(assigned);
            roles
        }
        None => assigned,
    })
}

/// Fetch the `@everyone` role for a community.
///
/// # Errors
///
/// Returns database errors.
pub async fn get_everyone_role(
    pool: &PgPool,
    community_id: Uuid,
) -> Result<Option<CommunityRole>, AuthError> {
    let sql = format!(
        "SELECT {ROLE_SELECT} FROM community_roles WHERE community_id = $1 AND is_everyone = TRUE"
    );
    let row = sqlx::query_as::<_, RoleRow>(&sql)
        .bind(community_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(RoleRow::into_role))
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
    let sql = r"
        SELECT r.id, r.community_id, r.name, r.position, r.weight, r.group_id, r.color, r.hoist,
               r.mentionable, r.permissions, r.is_everyone, r.short_tag, r.icon_emoji,
               r.icon_object_key, r.gradient, r.role_card, r.created_at, r.updated_at
        FROM community_role_assignments a
        INNER JOIN community_roles r ON r.id = a.role_id
        WHERE a.community_id = $1 AND a.account_id = $2 AND r.is_everyone = FALSE
        ORDER BY r.weight ASC, r.position ASC, r.created_at ASC
        ";
    let rows = sqlx::query_as::<_, RoleRow>(sql)
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
            min_weight: 0,
        });
    }
    let sql = r"
        SELECT r.id, r.community_id, r.name, r.position, r.weight, r.group_id, r.color, r.hoist,
               r.mentionable, r.permissions, r.is_everyone, r.short_tag, r.icon_emoji,
               r.icon_object_key, r.gradient, r.role_card, r.created_at, r.updated_at
        FROM community_role_assignments a
        INNER JOIN community_roles r ON r.id = a.role_id
        WHERE a.community_id = $1 AND a.account_id = $2 AND r.is_everyone = FALSE
        ";
    let rows = sqlx::query_as::<_, RoleRow>(sql)
        .bind(community_id)
        .bind(account_id)
        .fetch_all(pool)
        .await?;
    let mut min_weight = i32::MAX;
    let mut can_manage_roles = false;
    for row in rows {
        let role = row.into_role();
        if role.weight < min_weight {
            min_weight = role.weight;
        }
        if permissions_manage_roles(&role.permissions) {
            can_manage_roles = true;
        }
    }
    Ok(RoleActor {
        is_owner: false,
        can_manage_roles,
        min_weight: if min_weight == i32::MAX {
            1000
        } else {
            min_weight
        },
    })
}

/// Whether `actor` may manage a role at `target_weight` (must be strictly higher priority).
#[must_use]
pub fn can_manage_role_weight(actor: RoleActor, target_weight: i32) -> bool {
    if actor.is_owner {
        return true;
    }
    if !actor.can_manage_roles {
        return false;
    }
    actor.min_weight < target_weight
}

/// Backward-compatible alias: treat position as weight for callers not yet migrated.
#[must_use]
pub fn can_manage_role_position(actor: RoleActor, target_position: i32) -> bool {
    can_manage_role_weight(actor, target_position)
}

/// Whether permissions JSON grants role management.
#[must_use]
pub fn permissions_manage_roles(permissions: &Value) -> bool {
    let set = parse_role_permissions(permissions);
    set.allow.has(
        Family::Community,
        voxnexus_permissions::community::MANAGE_ROLES,
    ) || permissions
        .get("manage_roles")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// Build permissions JSON with `manage_roles` allow bit.
#[must_use]
pub fn permissions_with_manage_roles(manage_roles: bool) -> Value {
    if manage_roles {
        perm_json_manage_roles(true)
    } else {
        empty_role_permissions_json()
    }
}

/// Set or clear a role's custom icon object id (stored in `icon_object_key`).
///
/// Returns the previous `icon_object_key` when present.
///
/// # Errors
///
/// Returns database errors.
pub async fn set_role_icon(
    pool: &PgPool,
    role_id: Uuid,
    object_id: Option<Uuid>,
) -> Result<Option<String>, AuthError> {
    let current = get_role(pool, role_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    let previous = current.icon_object_key;
    let next = object_id.map(|id| id.to_string());
    let now = Utc::now();
    sqlx::query(
        r"
        UPDATE community_roles
        SET icon_object_key = $2, updated_at = $3
        WHERE id = $1
        ",
    )
    .bind(role_id)
    .bind(&next)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(previous)
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

async fn next_role_weight(pool: &PgPool, community_id: Uuid) -> Result<i32, AuthError> {
    let min: Option<i32> = sqlx::query_scalar(
        r"
        SELECT MIN(weight) FROM community_roles
        WHERE community_id = $1 AND is_everyone = FALSE
        ",
    )
    .bind(community_id)
    .fetch_one(pool)
    .await?;
    let candidate = min.map_or(100, |value| value.saturating_sub(1));
    if candidate >= 1 {
        return Ok(candidate);
    }
    // Find first free slot in 1..=999.
    let used: Vec<i32> = sqlx::query_scalar(
        r"
        SELECT weight FROM community_roles
        WHERE community_id = $1 AND weight BETWEEN 1 AND 999
        ORDER BY weight ASC
        ",
    )
    .bind(community_id)
    .fetch_all(pool)
    .await?;
    let mut expect = 1;
    for weight in used {
        if weight == expect {
            expect += 1;
        } else if weight > expect {
            break;
        }
    }
    if expect > 999 {
        return Err(AuthError::RoleWeightTaken);
    }
    Ok(expect)
}

fn validate_weight(weight: i32) -> Result<i32, AuthError> {
    if (1..=1000).contains(&weight) {
        Ok(weight)
    } else {
        Err(AuthError::InvalidRoleWeight)
    }
}

async fn ensure_group_in_community(
    pool: &PgPool,
    community_id: Uuid,
    group_id: Uuid,
) -> Result<(), AuthError> {
    let found: Option<Uuid> = sqlx::query_scalar(
        r"
        SELECT id FROM community_role_groups
        WHERE id = $1 AND community_id = $2
        ",
    )
    .bind(group_id)
    .bind(community_id)
    .fetch_optional(pool)
    .await?;
    if found.is_some() {
        Ok(())
    } else {
        Err(AuthError::RoleGroupNotFound)
    }
}

fn map_unique(error: sqlx::Error) -> AuthError {
    if is_unique_violation(&error) {
        // Prefer weight conflict when constraint name suggests it; otherwise name.
        if let sqlx::Error::Database(db) = &error {
            let constraint = db.constraint().unwrap_or("");
            if constraint.contains("weight") {
                return AuthError::RoleWeightTaken;
            }
        }
        return AuthError::RoleNameTaken;
    }
    error.into()
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
    weight: i32,
    group_id: Option<Uuid>,
    color: String,
    hoist: bool,
    mentionable: bool,
    permissions: Value,
    is_everyone: bool,
    short_tag: String,
    icon_emoji: Option<String>,
    icon_object_key: Option<String>,
    gradient: Option<String>,
    role_card: Value,
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
            weight: self.weight,
            group_id: self.group_id,
            color: self.color,
            hoist: self.hoist,
            mentionable: self.mentionable,
            permissions: self.permissions,
            is_everyone: self.is_everyone,
            short_tag: self.short_tag,
            icon_emoji: self.icon_emoji,
            icon_object_key: self.icon_object_key,
            gradient: self.gradient,
            role_card: self.role_card,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
