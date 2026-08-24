//! Permission override persistence (F030).

use chrono::Utc;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;
use voxnexus_permissions::{parse_role_permissions, OverrideBundle};

use crate::AuthError;

/// Stored permission override row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionOverride {
    pub id: Uuid,
    pub community_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub role_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub permissions: Value,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// List overrides scoped to a category.
///
/// # Errors
///
/// Returns database errors.
pub async fn list_category_overrides(
    pool: &PgPool,
    category_id: Uuid,
) -> Result<Vec<PermissionOverride>, AuthError> {
    let rows = sqlx::query_as::<_, OverrideRow>(
        r"
        SELECT id, community_id, channel_id, category_id, role_id, account_id,
               permissions, created_at, updated_at
        FROM permission_overrides
        WHERE category_id = $1
        ORDER BY created_at ASC
        ",
    )
    .bind(category_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(OverrideRow::into_override).collect())
}

/// List overrides for a channel (includes category overrides when `category_id` is set).
///
/// # Errors
///
/// Returns database errors.
pub async fn list_channel_overrides(
    pool: &PgPool,
    channel_id: Uuid,
    category_id: Option<Uuid>,
) -> Result<Vec<PermissionOverride>, AuthError> {
    let mut rows = sqlx::query_as::<_, OverrideRow>(
        r"
        SELECT id, community_id, channel_id, category_id, role_id, account_id,
               permissions, created_at, updated_at
        FROM permission_overrides
        WHERE channel_id = $1
        ORDER BY created_at ASC
        ",
    )
    .bind(channel_id)
    .fetch_all(pool)
    .await?;

    if let Some(category_id) = category_id {
        let category_rows = sqlx::query_as::<_, OverrideRow>(
            r"
            SELECT id, community_id, channel_id, category_id, role_id, account_id,
                   permissions, created_at, updated_at
            FROM permission_overrides
            WHERE category_id = $1
            ORDER BY created_at ASC
            ",
        )
        .bind(category_id)
        .fetch_all(pool)
        .await?;
        rows.extend(category_rows);
    }

    Ok(rows.into_iter().map(OverrideRow::into_override).collect())
}

/// Build an evaluation bundle for a channel + optional category.
///
/// # Errors
///
/// Returns database errors.
pub async fn override_bundle_for_channel(
    pool: &PgPool,
    channel_id: Uuid,
    category_id: Option<Uuid>,
    actor_account_id: Uuid,
) -> Result<OverrideBundle, AuthError> {
    let rows = list_channel_overrides(pool, channel_id, category_id).await?;
    let mut bundle = OverrideBundle::default();
    for row in rows {
        let perms = parse_role_permissions(&row.permissions);
        if row.channel_id.is_some() {
            if let Some(role_id) = row.role_id {
                bundle.channel_roles.push((role_id, perms));
            } else if row.account_id == Some(actor_account_id) {
                bundle.channel_member = Some(perms);
            }
        } else if row.category_id.is_some() {
            if let Some(role_id) = row.role_id {
                bundle.category_roles.push((role_id, perms));
            } else if row.account_id == Some(actor_account_id) {
                bundle.category_member = Some(perms);
            }
        }
    }
    Ok(bundle)
}

/// Upsert a role override on a category.
///
/// # Errors
///
/// Returns database errors or unique violations.
pub async fn upsert_category_role_override(
    pool: &PgPool,
    community_id: Uuid,
    category_id: Uuid,
    role_id: Uuid,
    permissions: Value,
) -> Result<PermissionOverride, AuthError> {
    upsert_override(
        pool,
        community_id,
        OverrideScope::Category(category_id),
        Some(role_id),
        None,
        permissions,
    )
    .await
}

/// Upsert a member override on a category.
///
/// # Errors
///
/// Returns database errors or unique violations.
pub async fn upsert_category_member_override(
    pool: &PgPool,
    community_id: Uuid,
    category_id: Uuid,
    account_id: Uuid,
    permissions: Value,
) -> Result<PermissionOverride, AuthError> {
    upsert_override(
        pool,
        community_id,
        OverrideScope::Category(category_id),
        None,
        Some(account_id),
        permissions,
    )
    .await
}

/// Upsert a role override on a channel.
///
/// # Errors
///
/// Returns database errors or unique violations.
pub async fn upsert_channel_role_override(
    pool: &PgPool,
    community_id: Uuid,
    channel_id: Uuid,
    role_id: Uuid,
    permissions: Value,
) -> Result<PermissionOverride, AuthError> {
    upsert_override(
        pool,
        community_id,
        OverrideScope::Channel(channel_id),
        Some(role_id),
        None,
        permissions,
    )
    .await
}

/// Upsert a member override on a channel.
///
/// # Errors
///
/// Returns database errors or unique violations.
pub async fn upsert_channel_member_override(
    pool: &PgPool,
    community_id: Uuid,
    channel_id: Uuid,
    account_id: Uuid,
    permissions: Value,
) -> Result<PermissionOverride, AuthError> {
    upsert_override(
        pool,
        community_id,
        OverrideScope::Channel(channel_id),
        None,
        Some(account_id),
        permissions,
    )
    .await
}

/// Delete an override by id scoped to a community.
///
/// # Errors
///
/// Returns database errors.
pub async fn delete_override(
    pool: &PgPool,
    community_id: Uuid,
    override_id: Uuid,
) -> Result<bool, AuthError> {
    let result = sqlx::query(
        r"
        DELETE FROM permission_overrides
        WHERE id = $1 AND community_id = $2
        ",
    )
    .bind(override_id)
    .bind(community_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

async fn upsert_override(
    pool: &PgPool,
    community_id: Uuid,
    scope: OverrideScope,
    role_id: Option<Uuid>,
    account_id: Option<Uuid>,
    permissions: Value,
) -> Result<PermissionOverride, AuthError> {
    let now = Utc::now();
    let id = Uuid::now_v7();
    let row = match (scope, role_id, account_id) {
        (OverrideScope::Channel(channel_id), Some(role_id), None) => {
            sqlx::query_as::<_, OverrideRow>(
                r"
                INSERT INTO permission_overrides (
                    id, community_id, channel_id, category_id, role_id, account_id,
                    permissions, created_at, updated_at
                ) VALUES ($1, $2, $3, NULL, $4, NULL, $5, $6, $6)
                ON CONFLICT (channel_id, role_id) WHERE channel_id IS NOT NULL AND role_id IS NOT NULL
                DO UPDATE SET permissions = EXCLUDED.permissions, updated_at = EXCLUDED.updated_at
                RETURNING id, community_id, channel_id, category_id, role_id, account_id,
                          permissions, created_at, updated_at
                ",
            )
            .bind(id)
            .bind(community_id)
            .bind(channel_id)
            .bind(role_id)
            .bind(&permissions)
            .bind(now)
            .fetch_one(pool)
            .await?
        }
        (OverrideScope::Channel(channel_id), None, Some(account_id)) => {
            sqlx::query_as::<_, OverrideRow>(
                r"
                INSERT INTO permission_overrides (
                    id, community_id, channel_id, category_id, role_id, account_id,
                    permissions, created_at, updated_at
                ) VALUES ($1, $2, $3, NULL, NULL, $4, $5, $6, $6)
                ON CONFLICT (channel_id, account_id) WHERE channel_id IS NOT NULL AND account_id IS NOT NULL
                DO UPDATE SET permissions = EXCLUDED.permissions, updated_at = EXCLUDED.updated_at
                RETURNING id, community_id, channel_id, category_id, role_id, account_id,
                          permissions, created_at, updated_at
                ",
            )
            .bind(id)
            .bind(community_id)
            .bind(channel_id)
            .bind(account_id)
            .bind(&permissions)
            .bind(now)
            .fetch_one(pool)
            .await?
        }
        (OverrideScope::Category(category_id), Some(role_id), None) => {
            sqlx::query_as::<_, OverrideRow>(
                r"
                INSERT INTO permission_overrides (
                    id, community_id, channel_id, category_id, role_id, account_id,
                    permissions, created_at, updated_at
                ) VALUES ($1, $2, NULL, $3, $4, NULL, $5, $6, $6)
                ON CONFLICT (category_id, role_id) WHERE category_id IS NOT NULL AND role_id IS NOT NULL
                DO UPDATE SET permissions = EXCLUDED.permissions, updated_at = EXCLUDED.updated_at
                RETURNING id, community_id, channel_id, category_id, role_id, account_id,
                          permissions, created_at, updated_at
                ",
            )
            .bind(id)
            .bind(community_id)
            .bind(category_id)
            .bind(role_id)
            .bind(&permissions)
            .bind(now)
            .fetch_one(pool)
            .await?
        }
        (OverrideScope::Category(category_id), None, Some(account_id)) => {
            sqlx::query_as::<_, OverrideRow>(
                r"
                INSERT INTO permission_overrides (
                    id, community_id, channel_id, category_id, role_id, account_id,
                    permissions, created_at, updated_at
                ) VALUES ($1, $2, NULL, $3, NULL, $4, $5, $6, $6)
                ON CONFLICT (category_id, account_id) WHERE category_id IS NOT NULL AND account_id IS NOT NULL
                DO UPDATE SET permissions = EXCLUDED.permissions, updated_at = EXCLUDED.updated_at
                RETURNING id, community_id, channel_id, category_id, role_id, account_id,
                          permissions, created_at, updated_at
                ",
            )
            .bind(id)
            .bind(community_id)
            .bind(category_id)
            .bind(account_id)
            .bind(&permissions)
            .bind(now)
            .fetch_one(pool)
            .await?
        }
        _ => return Err(AuthError::Db(sqlx::Error::RowNotFound)),
    };
    Ok(row.into_override())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverrideScope {
    Channel(Uuid),
    Category(Uuid),
}

#[derive(Debug, sqlx::FromRow)]
struct OverrideRow {
    id: Uuid,
    community_id: Uuid,
    channel_id: Option<Uuid>,
    category_id: Option<Uuid>,
    role_id: Option<Uuid>,
    account_id: Option<Uuid>,
    permissions: Value,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl OverrideRow {
    fn into_override(self) -> PermissionOverride {
        PermissionOverride {
            id: self.id,
            community_id: self.community_id,
            channel_id: self.channel_id,
            category_id: self.category_id,
            role_id: self.role_id,
            account_id: self.account_id,
            permissions: self.permissions,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
