//! Community invite persistence (F021).

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{DateTime, Utc};
use rand::RngCore;
use sqlx::PgPool;
use uuid::Uuid;
use voxnexus_domain::{CommunityInvite, CommunityMember, CommunityMemberRole, JoinMode};

use crate::community::{get_community, get_membership};
use crate::profile::get_profile;
use crate::{AuthError, MemberListItem};

/// Options when creating an invite.
#[derive(Debug, Clone, Default)]
pub struct CreateInviteInput {
    pub max_uses: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Patchable invite fields (owner / manage_invites).
#[derive(Debug, Clone, Default)]
pub struct InvitePatch {
    pub paused: Option<bool>,
}

/// Create an unguessable invite code for a community.
///
/// # Errors
///
/// Returns database errors.
pub async fn create_invite(
    pool: &PgPool,
    community_id: Uuid,
    created_by: Uuid,
    input: CreateInviteInput,
) -> Result<CommunityInvite, AuthError> {
    let _ = get_community(pool, community_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    let id = Uuid::now_v7();
    let now = Utc::now();
    let code = new_invite_code();
    sqlx::query(
        r"
        INSERT INTO community_invites (
            id, community_id, code, created_by, max_uses, uses, expires_at, paused, created_at
        ) VALUES ($1, $2, $3, $4, $5, 0, $6, FALSE, $7)
        ",
    )
    .bind(id)
    .bind(community_id)
    .bind(&code)
    .bind(created_by)
    .bind(input.max_uses)
    .bind(input.expires_at)
    .bind(now)
    .execute(pool)
    .await?;
    get_invite(pool, id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// List non-revoked invites for a community (newest first).
///
/// # Errors
///
/// Returns database errors.
pub async fn list_invites(
    pool: &PgPool,
    community_id: Uuid,
) -> Result<Vec<CommunityInvite>, AuthError> {
    let rows = sqlx::query_as::<_, InviteRow>(
        r"
        SELECT id, community_id, code, created_by, max_uses, uses, expires_at, paused,
               revoked_at, created_at
        FROM community_invites
        WHERE community_id = $1 AND revoked_at IS NULL
        ORDER BY created_at DESC
        ",
    )
    .bind(community_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(InviteRow::into_invite).collect())
}

/// Load one invite by id.
///
/// # Errors
///
/// Returns database errors.
pub async fn get_invite(pool: &PgPool, id: Uuid) -> Result<Option<CommunityInvite>, AuthError> {
    let row = sqlx::query_as::<_, InviteRow>(
        r"
        SELECT id, community_id, code, created_by, max_uses, uses, expires_at, paused,
               revoked_at, created_at
        FROM community_invites
        WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(InviteRow::into_invite))
}

/// Load one invite by public code.
///
/// # Errors
///
/// Returns database errors.
pub async fn get_invite_by_code(
    pool: &PgPool,
    code: &str,
) -> Result<Option<CommunityInvite>, AuthError> {
    let row = sqlx::query_as::<_, InviteRow>(
        r"
        SELECT id, community_id, code, created_by, max_uses, uses, expires_at, paused,
               revoked_at, created_at
        FROM community_invites
        WHERE code = $1
        ",
    )
    .bind(code)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(InviteRow::into_invite))
}

/// Soft-revoke an invite.
///
/// # Errors
///
/// Returns [`AuthError::InviteNotFound`] or database errors.
pub async fn revoke_invite(pool: &PgPool, invite_id: Uuid) -> Result<CommunityInvite, AuthError> {
    let now = Utc::now();
    let result = sqlx::query(
        r"
        UPDATE community_invites SET revoked_at = $2
        WHERE id = $1 AND revoked_at IS NULL
        ",
    )
    .bind(invite_id)
    .bind(now)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AuthError::InviteNotFound);
    }
    get_invite(pool, invite_id)
        .await?
        .ok_or(AuthError::InviteNotFound)
}

/// Pause or unpause an invite.
///
/// # Errors
///
/// Returns [`AuthError::InviteNotFound`] or database errors.
pub async fn update_invite(
    pool: &PgPool,
    invite_id: Uuid,
    patch: InvitePatch,
) -> Result<CommunityInvite, AuthError> {
    let current = get_invite(pool, invite_id)
        .await?
        .ok_or(AuthError::InviteNotFound)?;
    if current.revoked_at.is_some() {
        return Err(AuthError::InviteNotFound);
    }
    let paused = patch.paused.unwrap_or(current.paused);
    sqlx::query(
        r"
        UPDATE community_invites SET paused = $2 WHERE id = $1
        ",
    )
    .bind(invite_id)
    .bind(paused)
    .execute(pool)
    .await?;
    get_invite(pool, invite_id)
        .await?
        .ok_or(AuthError::InviteNotFound)
}

/// Accept an invite: validate, join as member, increment uses.
///
/// Works for open and invite join modes. Application mode is not supported here.
///
/// # Errors
///
/// Returns invite-state errors, [`AuthError::AlreadyMember`], [`AuthError::JoinNotAllowed`],
/// or database errors.
pub async fn accept_invite(
    pool: &PgPool,
    code: &str,
    account_id: Uuid,
) -> Result<(CommunityInvite, CommunityMember, MemberListItem), AuthError> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query_as::<_, InviteRow>(
        r"
        SELECT id, community_id, code, created_by, max_uses, uses, expires_at, paused,
               revoked_at, created_at
        FROM community_invites
        WHERE code = $1
        FOR UPDATE
        ",
    )
    .bind(code)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AuthError::InviteNotFound)?;
    let invite = row.into_invite();
    let now = Utc::now();
    if invite.revoked_at.is_some() {
        return Err(AuthError::InviteNotFound);
    }
    if invite.paused {
        return Err(AuthError::InvitePaused);
    }
    if invite.expires_at.is_some_and(|expires| expires <= now) {
        return Err(AuthError::InviteExpired);
    }
    if invite.max_uses.is_some_and(|max| invite.uses >= max) {
        return Err(AuthError::InviteExhausted);
    }

    let community = get_community(pool, invite.community_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    if community.join_mode == JoinMode::Application {
        return Err(AuthError::JoinNotAllowed);
    }
    if get_membership(pool, invite.community_id, account_id)
        .await?
        .is_some()
    {
        return Err(AuthError::AlreadyMember);
    }

    sqlx::query(
        r"
        INSERT INTO community_members (community_id, account_id, role, nickname, joined_at)
        VALUES ($1, $2, $3, '', $4)
        ",
    )
    .bind(invite.community_id)
    .bind(account_id)
    .bind(CommunityMemberRole::Member.as_str())
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|error| {
        if is_unique_violation(&error) {
            AuthError::AlreadyMember
        } else {
            AuthError::Db(error)
        }
    })?;

    sqlx::query(
        r"
        UPDATE community_invites SET uses = uses + 1 WHERE id = $1
        ",
    )
    .bind(invite.id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let member = get_membership(pool, invite.community_id, account_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    let invite = get_invite(pool, invite.id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    let profile = get_profile(pool, account_id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))?;
    Ok((
        invite,
        member.clone(),
        MemberListItem {
            member,
            display_name: profile.display_name,
            has_avatar: profile.avatar_object_id.is_some(),
        },
    ))
}

fn new_invite_code() -> String {
    let mut bytes = [0_u8; 12];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .is_some_and(|code| code == "23505")
}

#[derive(Debug, sqlx::FromRow)]
struct InviteRow {
    id: Uuid,
    community_id: Uuid,
    code: String,
    created_by: Uuid,
    max_uses: Option<i32>,
    uses: i32,
    expires_at: Option<DateTime<Utc>>,
    paused: bool,
    revoked_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl InviteRow {
    fn into_invite(self) -> CommunityInvite {
        CommunityInvite {
            id: self.id,
            community_id: self.community_id,
            code: self.code,
            created_by: self.created_by,
            max_uses: self.max_uses,
            uses: self.uses,
            expires_at: self.expires_at,
            paused: self.paused,
            revoked_at: self.revoked_at,
            created_at: self.created_at,
        }
    }
}
