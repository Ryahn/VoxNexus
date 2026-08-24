//! Community and membership persistence (F019 / F020).

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use voxnexus_domain::{
    Community, CommunityMember, CommunityMemberRole, JoinMode, DEFAULT_INSTANCE_ID,
};

use crate::AuthError;

/// Member row joined with profile fields for list responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberListItem {
    pub member: CommunityMember,
    pub display_name: String,
    pub has_avatar: bool,
}

/// Cursor page of community members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembersPage {
    pub items: Vec<MemberListItem>,
    pub has_more: bool,
}

/// Fields accepted when creating a community.
#[derive(Debug, Clone)]
pub struct CreateCommunityInput {
    pub name: String,
    pub slug: String,
    pub description: String,
    pub timezone: String,
    pub join_mode: JoinMode,
    pub discoverable_on_instance: bool,
}

/// Patchable community settings (owner only).
#[derive(Debug, Clone, Default)]
pub struct CommunityPatch {
    pub name: Option<String>,
    pub description: Option<String>,
    pub timezone: Option<String>,
    pub join_mode: Option<JoinMode>,
    pub discoverable_on_instance: Option<bool>,
    pub tag_name: Option<String>,
    pub tag_color: Option<String>,
    /// `None` = unchanged; `Some(None)` = clear; `Some(Some(path))` = set slugified path.
    pub invite_path: Option<Option<String>>,
}

/// Count communities on the default instance.
///
/// # Errors
///
/// Returns database errors.
pub async fn count_communities(pool: &PgPool) -> Result<i64, AuthError> {
    let count = sqlx::query_scalar::<_, i64>(
        r"
        SELECT COUNT(*)::bigint FROM communities WHERE instance_id = $1
        ",
    )
    .bind(DEFAULT_INSTANCE_ID)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

/// First instance admin account id, if any.
///
/// # Errors
///
/// Returns database errors.
pub async fn first_instance_admin_id(pool: &PgPool) -> Result<Option<Uuid>, AuthError> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r"
        SELECT id FROM accounts
        WHERE is_instance_admin = TRUE AND deleted_at IS NULL
        ORDER BY created_at ASC
        LIMIT 1
        ",
    )
    .fetch_optional(pool)
    .await?;
    Ok(id)
}

/// Create a community and add the creator as owner member.
///
/// # Errors
///
/// Returns [`AuthError::EmailTaken`]-style conflicts as [`AuthError::Db`] unique violations,
/// or other database errors.
pub async fn create_community(
    pool: &PgPool,
    owner_account_id: Uuid,
    input: CreateCommunityInput,
) -> Result<Community, AuthError> {
    let id = Uuid::now_v7();
    let now = Utc::now();
    let mut tx = pool.begin().await?;

    let result = sqlx::query(
        r"
        INSERT INTO communities (
            id, instance_id, name, slug, description, timezone, join_mode,
            owner_account_id, discoverable_on_instance, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
        ",
    )
    .bind(id)
    .bind(DEFAULT_INSTANCE_ID)
    .bind(&input.name)
    .bind(&input.slug)
    .bind(&input.description)
    .bind(&input.timezone)
    .bind(input.join_mode.as_str())
    .bind(owner_account_id)
    .bind(input.discoverable_on_instance)
    .bind(now)
    .execute(&mut *tx)
    .await;

    if let Err(error) = result {
        if is_unique_violation(&error) {
            return Err(AuthError::SlugTaken);
        }
        return Err(error.into());
    }

    sqlx::query(
        r"
        INSERT INTO community_members (community_id, account_id, role, nickname, joined_at)
        VALUES ($1, $2, $3, '', $4)
        ",
    )
    .bind(id)
    .bind(owner_account_id)
    .bind(CommunityMemberRole::Owner.as_str())
    .bind(now)
    .execute(&mut *tx)
    .await?;

    crate::role::insert_everyone_role(&mut tx, id, now).await?;

    tx.commit().await?;
    get_community(pool, id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Ensure a bootstrap community exists in `single` mode.
///
/// # Errors
///
/// Returns database errors.
pub async fn ensure_bootstrap_community(
    pool: &PgPool,
    owner_account_id: Uuid,
    name: &str,
) -> Result<Option<Community>, AuthError> {
    let count = count_communities(pool).await?;
    if count > 0 {
        return Ok(None);
    }
    let slug = unique_slug(pool, &slugify(name)).await?;
    let community = create_community(
        pool,
        owner_account_id,
        CreateCommunityInput {
            name: name.trim().to_owned(),
            slug,
            description: String::new(),
            timezone: "UTC".to_owned(),
            join_mode: JoinMode::Open,
            discoverable_on_instance: false,
        },
    )
    .await?;
    Ok(Some(community))
}

/// Load one community by id.
///
/// # Errors
///
/// Returns database errors.
pub async fn get_community(pool: &PgPool, id: Uuid) -> Result<Option<Community>, AuthError> {
    let row = sqlx::query_as::<_, CommunityRow>(
        r"
        SELECT id, instance_id, name, slug, description, timezone, join_mode,
               owner_account_id, icon_object_id, banner_object_id,
               tag_name, tag_color, tag_badge_object_id, invite_splash_object_id, invite_path,
               discoverable_on_instance, created_at, updated_at
        FROM communities
        WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    row.map(CommunityRow::into_community).transpose()
}

/// List communities the account belongs to (newest first).
///
/// # Errors
///
/// Returns database errors.
pub async fn list_communities_for_account(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Vec<Community>, AuthError> {
    let rows = sqlx::query_as::<_, CommunityRow>(
        r"
        SELECT c.id, c.instance_id, c.name, c.slug, c.description, c.timezone, c.join_mode,
               c.owner_account_id, c.icon_object_id, c.banner_object_id,
               c.tag_name, c.tag_color, c.tag_badge_object_id, c.invite_splash_object_id, c.invite_path,
               c.discoverable_on_instance, c.created_at, c.updated_at
        FROM communities c
        INNER JOIN community_members m ON m.community_id = c.id
        WHERE m.account_id = $1
        ORDER BY c.created_at DESC
        ",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(CommunityRow::into_community).collect()
}

/// Membership row for an account in a community, if any.
///
/// # Errors
///
/// Returns database errors.
pub async fn get_membership(
    pool: &PgPool,
    community_id: Uuid,
    account_id: Uuid,
) -> Result<Option<CommunityMember>, AuthError> {
    let row = sqlx::query_as::<_, MemberRow>(
        r"
        SELECT community_id, account_id, role, nickname, joined_at
        FROM community_members
        WHERE community_id = $1 AND account_id = $2
        ",
    )
    .bind(community_id)
    .bind(account_id)
    .fetch_optional(pool)
    .await?;
    row.map(MemberRow::into_member).transpose()
}

/// Join an open community as a regular member.
///
/// # Errors
///
/// Returns [`AuthError::JoinNotAllowed`] when `join_mode` is not open,
/// [`AuthError::AlreadyMember`] when already joined, or not-found / database errors.
pub async fn join_community(
    pool: &PgPool,
    community_id: Uuid,
    account_id: Uuid,
) -> Result<CommunityMember, AuthError> {
    let community = get_community(pool, community_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    if community.join_mode != JoinMode::Open {
        return Err(AuthError::JoinNotAllowed);
    }
    if get_membership(pool, community_id, account_id)
        .await?
        .is_some()
    {
        return Err(AuthError::AlreadyMember);
    }

    let now = Utc::now();
    let result = sqlx::query(
        r"
        INSERT INTO community_members (community_id, account_id, role, nickname, joined_at)
        VALUES ($1, $2, $3, '', $4)
        ",
    )
    .bind(community_id)
    .bind(account_id)
    .bind(CommunityMemberRole::Member.as_str())
    .bind(now)
    .execute(pool)
    .await;

    if let Err(error) = result {
        if is_unique_violation(&error) {
            return Err(AuthError::AlreadyMember);
        }
        return Err(error.into());
    }

    get_membership(pool, community_id, account_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Leave a community. Owners cannot leave (transfer is F025).
///
/// # Errors
///
/// Returns [`AuthError::NotMember`], [`AuthError::OwnerCannotLeave`], or database errors.
pub async fn leave_community(
    pool: &PgPool,
    community_id: Uuid,
    account_id: Uuid,
) -> Result<(), AuthError> {
    let membership = get_membership(pool, community_id, account_id)
        .await?
        .ok_or(AuthError::NotMember)?;
    if membership.role == CommunityMemberRole::Owner {
        return Err(AuthError::OwnerCannotLeave);
    }
    let result = sqlx::query(
        r"
        DELETE FROM community_members
        WHERE community_id = $1 AND account_id = $2
        ",
    )
    .bind(community_id)
    .bind(account_id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AuthError::NotMember);
    }
    Ok(())
}

/// Paginated members for a community (`after` / `before` on `account_id`).
///
/// # Errors
///
/// Returns database errors.
pub async fn list_members(
    pool: &PgPool,
    community_id: Uuid,
    after: Option<Uuid>,
    before: Option<Uuid>,
    limit: u16,
) -> Result<MembersPage, AuthError> {
    let fetch = i64::from(limit) + 1;
    let rows = if let Some(after_id) = after {
        sqlx::query_as::<_, MemberListRow>(
            r"
            SELECT m.community_id, m.account_id, m.role, m.nickname, m.joined_at,
                   p.display_name, (p.avatar_object_id IS NOT NULL) AS has_avatar
            FROM community_members m
            INNER JOIN profiles p ON p.account_id = m.account_id
            WHERE m.community_id = $1 AND m.account_id > $2
            ORDER BY m.account_id ASC
            LIMIT $3
            ",
        )
        .bind(community_id)
        .bind(after_id)
        .bind(fetch)
        .fetch_all(pool)
        .await?
    } else if let Some(before_id) = before {
        let mut rows = sqlx::query_as::<_, MemberListRow>(
            r"
            SELECT m.community_id, m.account_id, m.role, m.nickname, m.joined_at,
                   p.display_name, (p.avatar_object_id IS NOT NULL) AS has_avatar
            FROM community_members m
            INNER JOIN profiles p ON p.account_id = m.account_id
            WHERE m.community_id = $1 AND m.account_id < $2
            ORDER BY m.account_id DESC
            LIMIT $3
            ",
        )
        .bind(community_id)
        .bind(before_id)
        .bind(fetch)
        .fetch_all(pool)
        .await?;
        rows.reverse();
        rows
    } else {
        sqlx::query_as::<_, MemberListRow>(
            r"
            SELECT m.community_id, m.account_id, m.role, m.nickname, m.joined_at,
                   p.display_name, (p.avatar_object_id IS NOT NULL) AS has_avatar
            FROM community_members m
            INNER JOIN profiles p ON p.account_id = m.account_id
            WHERE m.community_id = $1
            ORDER BY m.account_id ASC
            LIMIT $2
            ",
        )
        .bind(community_id)
        .bind(fetch)
        .fetch_all(pool)
        .await?
    };

    let has_more = rows.len() > usize::from(limit);
    let items = rows
        .into_iter()
        .take(usize::from(limit))
        .map(MemberListRow::into_item)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MembersPage { items, has_more })
}

/// Account ids of every member (for gateway fanout).
///
/// # Errors
///
/// Returns database errors.
pub async fn list_member_account_ids(
    pool: &PgPool,
    community_id: Uuid,
) -> Result<Vec<Uuid>, AuthError> {
    let ids = sqlx::query_scalar::<_, Uuid>(
        r"
        SELECT account_id FROM community_members
        WHERE community_id = $1
        ORDER BY account_id ASC
        ",
    )
    .bind(community_id)
    .fetch_all(pool)
    .await?;
    Ok(ids)
}

/// Update the caller's community nickname.
///
/// # Errors
///
/// Returns [`AuthError::NotMember`] or database errors.
pub async fn set_nickname(
    pool: &PgPool,
    community_id: Uuid,
    account_id: Uuid,
    nickname: &str,
) -> Result<CommunityMember, AuthError> {
    let result = sqlx::query(
        r"
        UPDATE community_members SET nickname = $3
        WHERE community_id = $1 AND account_id = $2
        ",
    )
    .bind(community_id)
    .bind(account_id)
    .bind(nickname)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AuthError::NotMember);
    }
    get_membership(pool, community_id, account_id)
        .await?
        .ok_or(AuthError::NotMember)
}

/// Update community settings.
///
/// # Errors
///
/// Returns database errors.
pub async fn update_community(
    pool: &PgPool,
    community_id: Uuid,
    patch: CommunityPatch,
) -> Result<Community, AuthError> {
    let current = get_community(pool, community_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    let name = patch.name.unwrap_or(current.name);
    let description = patch.description.unwrap_or(current.description);
    let timezone = patch.timezone.unwrap_or(current.timezone);
    let join_mode = patch.join_mode.unwrap_or(current.join_mode);
    let discoverable = patch
        .discoverable_on_instance
        .unwrap_or(current.discoverable_on_instance);
    let tag_name = patch.tag_name.unwrap_or(current.tag_name);
    let tag_color = patch.tag_color.unwrap_or(current.tag_color);
    let invite_path = match patch.invite_path {
        None => current.invite_path,
        Some(None) => None,
        Some(Some(path)) => {
            let normalized = slugify(&path);
            if normalized.is_empty() {
                None
            } else {
                Some(normalized)
            }
        }
    };
    let now = Utc::now();
    let result = sqlx::query(
        r"
        UPDATE communities
        SET name = $2, description = $3, timezone = $4, join_mode = $5,
            discoverable_on_instance = $6, tag_name = $7, tag_color = $8, invite_path = $9,
            updated_at = $10
        WHERE id = $1
        ",
    )
    .bind(community_id)
    .bind(&name)
    .bind(&description)
    .bind(&timezone)
    .bind(join_mode.as_str())
    .bind(discoverable)
    .bind(&tag_name)
    .bind(&tag_color)
    .bind(&invite_path)
    .bind(now)
    .execute(pool)
    .await;
    if let Err(error) = result {
        if is_unique_violation(&error) {
            return Err(AuthError::SlugTaken);
        }
        return Err(error.into());
    }
    get_community(pool, community_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Transfer community ownership to an existing member (F025).
///
/// # Errors
///
/// Returns [`AuthError::NotCommunityOwner`], [`AuthError::NotMember`], or database errors.
pub async fn transfer_community(
    pool: &PgPool,
    community_id: Uuid,
    current_owner_id: Uuid,
    new_owner_id: Uuid,
) -> Result<Community, AuthError> {
    if current_owner_id == new_owner_id {
        return Err(AuthError::NotMember);
    }
    let community = get_community(pool, community_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    if community.owner_account_id != current_owner_id {
        return Err(AuthError::NotCommunityOwner);
    }
    if get_membership(pool, community_id, new_owner_id)
        .await?
        .is_none()
    {
        return Err(AuthError::NotMember);
    }
    let now = Utc::now();
    let mut tx = pool.begin().await?;
    sqlx::query(
        r"
        UPDATE communities
        SET owner_account_id = $2, updated_at = $3
        WHERE id = $1
        ",
    )
    .bind(community_id)
    .bind(new_owner_id)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r"
        UPDATE community_members
        SET role = $3
        WHERE community_id = $1 AND account_id = $2
        ",
    )
    .bind(community_id)
    .bind(current_owner_id)
    .bind(CommunityMemberRole::Member.as_str())
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r"
        UPDATE community_members
        SET role = $3
        WHERE community_id = $1 AND account_id = $2
        ",
    )
    .bind(community_id)
    .bind(new_owner_id)
    .bind(CommunityMemberRole::Owner.as_str())
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    get_community(pool, community_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Delete a community and cascaded rows (members, invites, spaces) (F025).
///
/// # Errors
///
/// Returns [`AuthError::NotCommunityOwner`] or database errors.
pub async fn delete_community(
    pool: &PgPool,
    community_id: Uuid,
    owner_account_id: Uuid,
) -> Result<(), AuthError> {
    let community = get_community(pool, community_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    if community.owner_account_id != owner_account_id {
        return Err(AuthError::NotCommunityOwner);
    }
    let result = sqlx::query("DELETE FROM communities WHERE id = $1")
        .bind(community_id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AuthError::Db(sqlx::Error::RowNotFound));
    }
    Ok(())
}

/// Set community icon object; returns previous object id.
///
/// # Errors
///
/// Returns database errors.
pub async fn set_community_icon(
    pool: &PgPool,
    community_id: Uuid,
    object_id: Uuid,
) -> Result<Option<Uuid>, AuthError> {
    let previous = sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT icon_object_id FROM communities WHERE id = $1",
    )
    .bind(community_id)
    .fetch_one(pool)
    .await?;
    let now = Utc::now();
    sqlx::query(
        r"
        UPDATE communities SET icon_object_id = $2, updated_at = $3 WHERE id = $1
        ",
    )
    .bind(community_id)
    .bind(object_id)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(previous)
}

/// Set community banner object; returns previous object id.
///
/// # Errors
///
/// Returns database errors.
pub async fn set_community_banner(
    pool: &PgPool,
    community_id: Uuid,
    object_id: Uuid,
) -> Result<Option<Uuid>, AuthError> {
    let previous = sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT banner_object_id FROM communities WHERE id = $1",
    )
    .bind(community_id)
    .fetch_one(pool)
    .await?;
    let now = Utc::now();
    sqlx::query(
        r"
        UPDATE communities SET banner_object_id = $2, updated_at = $3 WHERE id = $1
        ",
    )
    .bind(community_id)
    .bind(object_id)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(previous)
}

/// Set community tag badge object; returns previous object id.
///
/// # Errors
///
/// Returns database errors.
pub async fn set_community_tag_badge(
    pool: &PgPool,
    community_id: Uuid,
    object_id: Uuid,
) -> Result<Option<Uuid>, AuthError> {
    let previous = sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT tag_badge_object_id FROM communities WHERE id = $1",
    )
    .bind(community_id)
    .fetch_one(pool)
    .await?;
    let now = Utc::now();
    sqlx::query(
        r"
        UPDATE communities SET tag_badge_object_id = $2, updated_at = $3 WHERE id = $1
        ",
    )
    .bind(community_id)
    .bind(object_id)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(previous)
}

/// Set community invite splash object; returns previous object id.
///
/// # Errors
///
/// Returns database errors.
pub async fn set_community_invite_splash(
    pool: &PgPool,
    community_id: Uuid,
    object_id: Uuid,
) -> Result<Option<Uuid>, AuthError> {
    let previous = sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT invite_splash_object_id FROM communities WHERE id = $1",
    )
    .bind(community_id)
    .fetch_one(pool)
    .await?;
    let now = Utc::now();
    sqlx::query(
        r"
        UPDATE communities SET invite_splash_object_id = $2, updated_at = $3 WHERE id = $1
        ",
    )
    .bind(community_id)
    .bind(object_id)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(previous)
}

/// Whether `slug` is already taken on this instance.
///
/// # Errors
///
/// Returns database errors.
pub async fn slug_taken(pool: &PgPool, slug: &str) -> Result<bool, AuthError> {
    let exists = sqlx::query_scalar::<_, bool>(
        r"
        SELECT EXISTS(
            SELECT 1 FROM communities WHERE instance_id = $1 AND slug = $2
        )
        ",
    )
    .bind(DEFAULT_INSTANCE_ID)
    .bind(slug)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// Derive a URL slug from a display name.
#[must_use]
pub fn slugify(name: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in name.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            out.push(lower);
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "community".to_owned()
    } else {
        out.chars().take(48).collect()
    }
}

/// Pick a unique slug, appending `-2`, `-3`, … when needed.
///
/// # Errors
///
/// Returns database errors.
pub async fn unique_slug(pool: &PgPool, base: &str) -> Result<String, AuthError> {
    let base = if base.is_empty() {
        "community".to_owned()
    } else {
        base.to_owned()
    };
    if !slug_taken(pool, &base).await? {
        return Ok(base);
    }
    for n in 2..10_000 {
        let candidate = format!("{base}-{n}");
        let clipped: String = candidate.chars().take(48).collect();
        if !slug_taken(pool, &clipped).await? {
            return Ok(clipped);
        }
    }
    Ok(format!(
        "{}-{}",
        &base[..base.len().min(40)],
        Uuid::now_v7().simple()
    ))
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .is_some_and(|code| code == "23505")
}

#[derive(Debug, sqlx::FromRow)]
struct CommunityRow {
    id: Uuid,
    instance_id: Uuid,
    name: String,
    slug: String,
    description: String,
    timezone: String,
    join_mode: String,
    owner_account_id: Uuid,
    icon_object_id: Option<Uuid>,
    banner_object_id: Option<Uuid>,
    tag_name: String,
    tag_color: String,
    tag_badge_object_id: Option<Uuid>,
    invite_splash_object_id: Option<Uuid>,
    invite_path: Option<String>,
    discoverable_on_instance: bool,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl CommunityRow {
    fn into_community(self) -> Result<Community, AuthError> {
        let join_mode = JoinMode::parse(&self.join_mode).ok_or_else(|| {
            AuthError::Db(sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid join_mode {}", self.join_mode),
            ))))
        })?;
        Ok(Community {
            id: self.id,
            instance_id: self.instance_id,
            name: self.name,
            slug: self.slug,
            description: self.description,
            timezone: self.timezone,
            join_mode,
            owner_account_id: self.owner_account_id,
            icon_object_id: self.icon_object_id,
            banner_object_id: self.banner_object_id,
            tag_name: self.tag_name,
            tag_color: self.tag_color,
            tag_badge_object_id: self.tag_badge_object_id,
            invite_splash_object_id: self.invite_splash_object_id,
            invite_path: self.invite_path,
            discoverable_on_instance: self.discoverable_on_instance,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
struct MemberRow {
    community_id: Uuid,
    account_id: Uuid,
    role: String,
    nickname: String,
    joined_at: chrono::DateTime<Utc>,
}

impl MemberRow {
    fn into_member(self) -> Result<CommunityMember, AuthError> {
        let role = CommunityMemberRole::parse(&self.role).ok_or_else(|| {
            AuthError::Db(sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid member role {}", self.role),
            ))))
        })?;
        Ok(CommunityMember {
            community_id: self.community_id,
            account_id: self.account_id,
            role,
            nickname: self.nickname,
            joined_at: self.joined_at,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
struct MemberListRow {
    community_id: Uuid,
    account_id: Uuid,
    role: String,
    nickname: String,
    joined_at: DateTime<Utc>,
    display_name: String,
    has_avatar: bool,
}

impl MemberListRow {
    fn into_item(self) -> Result<MemberListItem, AuthError> {
        let role = CommunityMemberRole::parse(&self.role).ok_or_else(|| {
            AuthError::Db(sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid member role {}", self.role),
            ))))
        })?;
        Ok(MemberListItem {
            member: CommunityMember {
                community_id: self.community_id,
                account_id: self.account_id,
                role,
                nickname: self.nickname,
                joined_at: self.joined_at,
            },
            display_name: self.display_name,
            has_avatar: self.has_avatar,
        })
    }
}
