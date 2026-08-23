//! Space persistence and membership (F022 / F023).

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use voxnexus_domain::{CommunityMemberRole, Space, SpaceMember, SpaceVisibility};

use crate::community::get_membership;
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

/// Create a Space in `community_id` and add `creator_account_id` as a member.
/// Spaces cannot nest (no parent column).
///
/// # Errors
///
/// Returns database errors.
pub async fn create_space(
    pool: &PgPool,
    community_id: Uuid,
    creator_account_id: Uuid,
    input: CreateSpaceInput,
) -> Result<Space, AuthError> {
    let id = Uuid::now_v7();
    let now = Utc::now();
    let position = next_position(pool, community_id).await?;
    let mut tx = pool.begin().await?;
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
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r"
        INSERT INTO space_members (space_id, account_id, joined_at)
        VALUES ($1, $2, $3)
        ",
    )
    .bind(id)
    .bind(creator_account_id)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
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

/// List Spaces for a community ordered by position then created_at (unfiltered).
/// Prefer [`list_spaces_visible_to`] for API responses.
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

/// Spaces visible to `account_id`: open spaces, spaces they belong to, or all if community owner.
///
/// # Errors
///
/// Returns database errors.
pub async fn list_spaces_visible_to(
    pool: &PgPool,
    community_id: Uuid,
    account_id: Uuid,
) -> Result<Vec<(Space, bool)>, AuthError> {
    let is_owner = get_membership(pool, community_id, account_id)
        .await?
        .is_some_and(|m| m.role == CommunityMemberRole::Owner);
    let rows = sqlx::query_as::<_, SpaceWithMembershipRow>(
        r"
        SELECT s.id, s.community_id, s.name, s.description, s.topic, s.game, s.visibility,
               s.icon_object_id, s.position, s.created_at, s.updated_at,
               (sm.account_id IS NOT NULL) AS is_member
        FROM spaces s
        LEFT JOIN space_members sm
          ON sm.space_id = s.id AND sm.account_id = $2
        WHERE s.community_id = $1
          AND (
            $3
            OR s.visibility = 'open'
            OR sm.account_id IS NOT NULL
          )
        ORDER BY s.position ASC, s.created_at ASC
        ",
    )
    .bind(community_id)
    .bind(account_id)
    .bind(is_owner)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let is_member = row.is_member;
            (row.into_space(), is_member)
        })
        .collect())
}

/// Whether `account_id` may view this space (404 if not).
///
/// Community owners always can. Open spaces: any community member. Restricted: space members only.
///
/// # Errors
///
/// Returns database errors.
pub async fn can_view_space(
    pool: &PgPool,
    space: &Space,
    account_id: Uuid,
) -> Result<bool, AuthError> {
    if let Some(membership) = get_membership(pool, space.community_id, account_id).await? {
        if membership.role == CommunityMemberRole::Owner {
            return Ok(true);
        }
    } else {
        return Ok(false);
    }
    match space.visibility {
        SpaceVisibility::Open => Ok(true),
        SpaceVisibility::Restricted => is_space_member(pool, space.id, account_id).await,
    }
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

/// Whether `account_id` is a member of the space.
///
/// # Errors
///
/// Returns database errors.
pub async fn is_space_member(
    pool: &PgPool,
    space_id: Uuid,
    account_id: Uuid,
) -> Result<bool, AuthError> {
    let exists: bool = sqlx::query_scalar(
        r"
        SELECT EXISTS(
            SELECT 1 FROM space_members
            WHERE space_id = $1 AND account_id = $2
        )
        ",
    )
    .bind(space_id)
    .bind(account_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// Join an open space. Restricted spaces require an admin add ([`add_space_member`]).
///
/// # Errors
///
/// Returns [`AuthError::SpaceJoinNotAllowed`], [`AuthError::AlreadySpaceMember`],
/// [`AuthError::NotMember`] (not a community member), or database errors.
pub async fn join_space(
    pool: &PgPool,
    space_id: Uuid,
    account_id: Uuid,
) -> Result<SpaceMember, AuthError> {
    let space = get_space(pool, space_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    if get_membership(pool, space.community_id, account_id)
        .await?
        .is_none()
    {
        return Err(AuthError::NotMember);
    }
    if space.visibility != SpaceVisibility::Open {
        return Err(AuthError::SpaceJoinNotAllowed);
    }
    insert_space_member(pool, space_id, account_id).await
}

/// Leave a space.
///
/// # Errors
///
/// Returns [`AuthError::NotSpaceMember`] or database errors.
pub async fn leave_space(pool: &PgPool, space_id: Uuid, account_id: Uuid) -> Result<(), AuthError> {
    let result = sqlx::query(
        r"
        DELETE FROM space_members
        WHERE space_id = $1 AND account_id = $2
        ",
    )
    .bind(space_id)
    .bind(account_id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AuthError::NotSpaceMember);
    }
    Ok(())
}

/// Add a community member to a space (manual membership for restricted spaces).
///
/// # Errors
///
/// Returns [`AuthError::NotMember`] if target is not a community member,
/// [`AuthError::AlreadySpaceMember`], or database errors.
pub async fn add_space_member(
    pool: &PgPool,
    space_id: Uuid,
    account_id: Uuid,
) -> Result<SpaceMember, AuthError> {
    let space = get_space(pool, space_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    if get_membership(pool, space.community_id, account_id)
        .await?
        .is_none()
    {
        return Err(AuthError::NotMember);
    }
    insert_space_member(pool, space_id, account_id).await
}

/// Remove a member from a space.
///
/// # Errors
///
/// Returns [`AuthError::NotSpaceMember`] or database errors.
pub async fn remove_space_member(
    pool: &PgPool,
    space_id: Uuid,
    account_id: Uuid,
) -> Result<(), AuthError> {
    leave_space(pool, space_id, account_id).await
}

/// Space member with profile display fields for API lists.
#[derive(Debug, Clone)]
pub struct SpaceMemberListItem {
    pub member: SpaceMember,
    pub display_name: String,
    pub has_avatar: bool,
}

/// List space members with display names.
///
/// # Errors
///
/// Returns database errors.
pub async fn list_space_members(
    pool: &PgPool,
    space_id: Uuid,
) -> Result<Vec<SpaceMemberListItem>, AuthError> {
    let rows = sqlx::query_as::<_, SpaceMemberListRow>(
        r"
        SELECT sm.space_id, sm.account_id, sm.joined_at,
               COALESCE(NULLIF(TRIM(p.display_name), ''), split_part(a.email, '@', 1)) AS display_name,
               (p.avatar_object_id IS NOT NULL) AS has_avatar
        FROM space_members sm
        INNER JOIN accounts a ON a.id = sm.account_id
        LEFT JOIN profiles p ON p.account_id = sm.account_id
        WHERE sm.space_id = $1
        ORDER BY sm.joined_at ASC
        ",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(SpaceMemberListRow::into_item)
        .collect())
}

async fn insert_space_member(
    pool: &PgPool,
    space_id: Uuid,
    account_id: Uuid,
) -> Result<SpaceMember, AuthError> {
    let now = Utc::now();
    let result = sqlx::query(
        r"
        INSERT INTO space_members (space_id, account_id, joined_at)
        VALUES ($1, $2, $3)
        ",
    )
    .bind(space_id)
    .bind(account_id)
    .bind(now)
    .execute(pool)
    .await;
    match result {
        Ok(_) => Ok(SpaceMember {
            space_id,
            account_id,
            joined_at: now,
        }),
        Err(error) if is_unique_violation(&error) => Err(AuthError::AlreadySpaceMember),
        Err(error) => Err(AuthError::Db(error)),
    }
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

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .is_some_and(|code| code == "23505")
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

#[derive(Debug, sqlx::FromRow)]
struct SpaceWithMembershipRow {
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
    is_member: bool,
}

impl SpaceWithMembershipRow {
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

#[derive(Debug, sqlx::FromRow)]
struct SpaceMemberListRow {
    space_id: Uuid,
    account_id: Uuid,
    joined_at: chrono::DateTime<Utc>,
    display_name: String,
    has_avatar: bool,
}

impl SpaceMemberListRow {
    fn into_item(self) -> SpaceMemberListItem {
        SpaceMemberListItem {
            member: SpaceMember {
                space_id: self.space_id,
                account_id: self.account_id,
                joined_at: self.joined_at,
            },
            display_name: self.display_name,
            has_avatar: self.has_avatar,
        }
    }
}
