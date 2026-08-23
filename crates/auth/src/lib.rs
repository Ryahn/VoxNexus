//! Passwords, sessions, and account persistence.

mod community;
mod instance;
mod oidc;
mod password;
mod profile;
mod session;

use chrono::{DateTime, Duration, Utc};
use sqlx::postgres::Postgres;
use sqlx::{PgPool, Transaction};
use uuid::Uuid;
use voxnexus_domain::{Account, AuthIdentity, Session, DEFAULT_INSTANCE_ID};

pub use community::{
    count_communities, create_community, ensure_bootstrap_community, first_instance_admin_id,
    get_community, get_membership, list_communities_for_account, set_community_banner,
    set_community_icon, slug_taken, slugify, unique_slug, update_community, CommunityPatch,
    CreateCommunityInput,
};
pub use instance::{
    ensure_instance, get_instance, sync_locked_community_creation_mode, sync_oidc_from_config,
    update_instance, InstanceError, InstancePatch, InstanceSeed,
};
pub use oidc::{resolve_oidc_login, OidcIdentity};
pub use password::{hash_password, verify_password, PasswordError};
pub use profile::{
    delete_object_meta, ensure_profile, get_object, get_profile, insert_object, set_avatar_object,
    set_banner_object, update_profile,
};
pub use session::{
    clear_session_cookie, hash_session_token, new_session_token, session_cookie,
    session_cookie_name, SessionCookieOptions, SESSION_TTL,
};

/// Product crate name.
pub const CRATE_NAME: &str = "voxnexus-auth";

/// Advisory lock key for one-shot instance admin bootstrap (transaction-scoped).
const INSTANCE_BOOTSTRAP_ADVISORY_LOCK: i64 = 0x766f_786e_6578_7573; // "voxnexus"

/// Outcome of [`bootstrap_instance_admin`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapResult {
    /// A new or existing account was promoted to instance admin.
    Created,
    /// An instance admin already exists; bootstrap was skipped.
    AlreadyBootstrapped,
}

/// Auth and account persistence errors.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error(transparent)]
    Password(#[from] PasswordError),
    #[error("email already registered")]
    EmailTaken,
    #[error("community slug already taken")]
    SlugTaken,
    #[error("issuer and subject already linked")]
    IdentityTaken,
    #[error("registration is closed")]
    RegistrationClosed,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// Create a local password account.
///
/// # Errors
///
/// Returns [`AuthError::RegistrationClosed`], [`AuthError::EmailTaken`], password hash failures, or database errors.
pub async fn create_local_account(
    pool: &PgPool,
    email: &str,
    password: &str,
    registration_open: bool,
) -> Result<Account, AuthError> {
    if !registration_open {
        return Err(AuthError::RegistrationClosed);
    }
    let email = normalize_email(email);
    let password_hash = hash_password(password)?;
    let now = Utc::now();
    let id = Uuid::now_v7();

    let mut tx = pool.begin().await?;
    acquire_bootstrap_lock(&mut tx).await?;
    let is_instance_admin = !instance_admin_exists(&mut tx).await?;

    let result = sqlx::query(
        r"
        INSERT INTO accounts (
            id, instance_id, email, password_hash, is_bot, is_instance_admin, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, FALSE, $5, $6, $6)
        ",
    )
    .bind(id)
    .bind(DEFAULT_INSTANCE_ID)
    .bind(&email)
    .bind(&password_hash)
    .bind(is_instance_admin)
    .bind(now)
    .execute(&mut *tx)
    .await;

    match result {
        Ok(_) => {
            tx.commit().await?;
            ensure_profile(pool, id).await?;
            get_account(pool, id)
                .await?
                .ok_or_else(|| AuthError::Db(sqlx::Error::RowNotFound))
        }
        Err(sqlx::Error::Database(db)) if db.constraint() == Some("accounts_email_unique") => {
            Err(AuthError::EmailTaken)
        }
        Err(error) => Err(AuthError::Db(error)),
    }
}

/// Create or promote the instance admin from environment credentials when no admin exists yet.
///
/// # Errors
///
/// Returns password hash failures or database errors.
pub async fn bootstrap_instance_admin(
    pool: &PgPool,
    email: &str,
    password: &str,
) -> Result<BootstrapResult, AuthError> {
    let email = normalize_email(email);
    let password_hash = hash_password(password)?;
    let now = Utc::now();

    let mut tx = pool.begin().await?;
    acquire_bootstrap_lock(&mut tx).await?;
    if instance_admin_exists(&mut tx).await? {
        tx.commit().await?;
        return Ok(BootstrapResult::AlreadyBootstrapped);
    }

    if let Some(existing) = fetch_account_by_email_tx(&mut tx, &email).await? {
        sqlx::query(
            r"
            UPDATE accounts
            SET is_instance_admin = TRUE, password_hash = $2, updated_at = $3
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(existing.id)
        .bind(&password_hash)
        .bind(now)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Ok(BootstrapResult::Created);
    }

    let id = Uuid::now_v7();
    sqlx::query(
        r"
        INSERT INTO accounts (
            id, instance_id, email, password_hash, is_bot, is_instance_admin, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, FALSE, TRUE, $5, $5)
        ",
    )
    .bind(id)
    .bind(DEFAULT_INSTANCE_ID)
    .bind(&email)
    .bind(&password_hash)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    ensure_profile(pool, id).await?;
    Ok(BootstrapResult::Created)
}

/// Link an OIDC (or other) identity to an account.
///
/// # Errors
///
/// Returns [`AuthError::IdentityTaken`] on unique conflict, or database errors.
pub async fn insert_auth_identity(
    pool: &PgPool,
    account_id: Uuid,
    issuer: &str,
    subject: &str,
) -> Result<AuthIdentity, AuthError> {
    let id = Uuid::now_v7();
    let now = Utc::now();
    let result = sqlx::query(
        r"
        INSERT INTO auth_identities (id, account_id, issuer, subject, created_at)
        VALUES ($1, $2, $3, $4, $5)
        ",
    )
    .bind(id)
    .bind(account_id)
    .bind(issuer)
    .bind(subject)
    .bind(now)
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(AuthIdentity {
            id,
            account_id,
            issuer: issuer.to_owned(),
            subject: subject.to_owned(),
            created_at: now,
        }),
        Err(sqlx::Error::Database(db))
            if db.constraint() == Some("auth_identities_issuer_subject_unique") =>
        {
            Err(AuthError::IdentityTaken)
        }
        Err(error) => Err(AuthError::Db(error)),
    }
}

/// Verify email/password. Always performs a password verify for timing safety.
///
/// # Errors
///
/// Returns [`AuthError::InvalidCredentials`] on mismatch, or database / hash errors.
pub async fn authenticate_local(
    pool: &PgPool,
    email: &str,
    password: &str,
) -> Result<Account, AuthError> {
    let email = normalize_email(email);
    let account = find_account_by_email(pool, &email).await?;
    let hash = account
        .as_ref()
        .and_then(|row| row.password_hash.as_deref());
    let ok = verify_password(password, hash)?;
    if !ok {
        return Err(AuthError::InvalidCredentials);
    }
    account.ok_or(AuthError::InvalidCredentials)
}

/// Create a session and return `(session, raw_token_for_cookie)`.
///
/// # Errors
///
/// Returns database errors if the insert fails.
pub async fn create_session(
    pool: &PgPool,
    account_id: Uuid,
    user_agent: Option<&str>,
    created_ip: Option<&str>,
) -> Result<(Session, String), AuthError> {
    let id = Uuid::now_v7();
    let token = new_session_token();
    let token_hash = hash_session_token(&token);
    let now = Utc::now();
    let expires_at = now + SESSION_TTL;

    sqlx::query(
        r"
        INSERT INTO sessions (
            id, account_id, token_hash, expires_at, user_agent, created_ip, created_at, last_seen_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
        ",
    )
    .bind(id)
    .bind(account_id)
    .bind(&token_hash[..])
    .bind(expires_at)
    .bind(user_agent)
    .bind(created_ip)
    .bind(now)
    .execute(pool)
    .await?;

    Ok((
        Session {
            id,
            account_id,
            expires_at,
            user_agent: user_agent.map(str::to_owned),
            created_at: now,
            last_seen_at: now,
        },
        token,
    ))
}

/// Resolve a live session from a raw cookie token. Touches `last_seen_at` and slides expiry.
///
/// # Errors
///
/// Returns database errors if lookup or update fails.
pub async fn resolve_session(
    pool: &PgPool,
    raw_token: &str,
) -> Result<Option<(Session, Account)>, AuthError> {
    let token_hash = hash_session_token(raw_token);
    let now = Utc::now();
    let row = sqlx::query_as::<_, SessionAccountRow>(
        r"
        SELECT
            s.id AS session_id,
            s.account_id,
            s.expires_at,
            s.user_agent,
            s.created_at AS session_created_at,
            s.last_seen_at,
            a.id AS account_id_full,
            a.instance_id,
            a.email,
            a.password_hash,
            a.is_bot,
            a.is_instance_admin,
            a.created_at AS account_created_at,
            a.updated_at,
            a.deleted_at
        FROM sessions s
        INNER JOIN accounts a ON a.id = s.account_id
        WHERE s.token_hash = $1
          AND s.expires_at > $2
          AND a.deleted_at IS NULL
        ",
    )
    .bind(&token_hash[..])
    .bind(now)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let new_expires = now + SESSION_TTL;
    sqlx::query(
        r"
        UPDATE sessions
        SET last_seen_at = $2, expires_at = $3
        WHERE id = $1
        ",
    )
    .bind(row.session_id)
    .bind(now)
    .bind(new_expires)
    .execute(pool)
    .await?;

    Ok(Some((
        Session {
            id: row.session_id,
            account_id: row.account_id,
            expires_at: new_expires,
            user_agent: row.user_agent,
            created_at: row.session_created_at,
            last_seen_at: now,
        },
        Account {
            id: row.account_id_full,
            instance_id: row.instance_id,
            email: row.email,
            password_hash: row.password_hash,
            is_bot: row.is_bot,
            is_instance_admin: row.is_instance_admin,
            created_at: row.account_created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        },
    )))
}

/// Delete a session by raw cookie token.
///
/// # Errors
///
/// Returns database errors if the delete fails.
pub async fn revoke_session(pool: &PgPool, raw_token: &str) -> Result<bool, AuthError> {
    let token_hash = hash_session_token(raw_token);
    let result = sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
        .bind(&token_hash[..])
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Delete all sessions for an account except one (F015 revoke-other on password change).
///
/// # Errors
///
/// Returns database errors if the delete fails.
pub async fn revoke_other_sessions(
    pool: &PgPool,
    account_id: Uuid,
    except_session_id: Uuid,
) -> Result<u64, AuthError> {
    let result = sqlx::query("DELETE FROM sessions WHERE account_id = $1 AND id != $2")
        .bind(account_id)
        .bind(except_session_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Change password after verifying the current one.
///
/// # Errors
///
/// Returns [`AuthError::InvalidCredentials`] when the current password is wrong, or database / hash errors.
pub async fn change_password(
    pool: &PgPool,
    account_id: Uuid,
    current_password: &str,
    new_password: &str,
) -> Result<(), AuthError> {
    let account = get_account(pool, account_id)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;
    let hash = account.password_hash.as_deref();
    if !verify_password(current_password, hash)? {
        return Err(AuthError::InvalidCredentials);
    }
    let new_hash = hash_password(new_password)?;
    let now = Utc::now();
    sqlx::query(
        r"
        UPDATE accounts
        SET password_hash = $2, updated_at = $3
        WHERE id = $1 AND deleted_at IS NULL
        ",
    )
    .bind(account_id)
    .bind(&new_hash)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

/// Change email after verifying the current password (immediate until F117 confirmation mail).
///
/// # Errors
///
/// Returns [`AuthError::InvalidCredentials`], [`AuthError::EmailTaken`], or database errors.
pub async fn change_email(
    pool: &PgPool,
    account_id: Uuid,
    email: &str,
    current_password: &str,
) -> Result<Account, AuthError> {
    let account = get_account(pool, account_id)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;
    let hash = account.password_hash.as_deref();
    if !verify_password(current_password, hash)? {
        return Err(AuthError::InvalidCredentials);
    }
    let email = normalize_email(email);
    let now = Utc::now();
    let result = sqlx::query(
        r"
        UPDATE accounts
        SET email = $2, updated_at = $3
        WHERE id = $1 AND deleted_at IS NULL
        ",
    )
    .bind(account_id)
    .bind(&email)
    .bind(now)
    .execute(pool)
    .await;

    match result {
        Ok(_) => get_account(pool, account_id)
            .await?
            .ok_or_else(|| AuthError::Db(sqlx::Error::RowNotFound)),
        Err(sqlx::Error::Database(db)) if db.constraint() == Some("accounts_email_unique") => {
            Err(AuthError::EmailTaken)
        }
        Err(error) => Err(AuthError::Db(error)),
    }
}

/// Load an account by id.
///
/// # Errors
///
/// Returns database errors if the query fails.
pub async fn get_account(pool: &PgPool, id: Uuid) -> Result<Option<Account>, AuthError> {
    let row = sqlx::query_as::<_, AccountRow>(
        r"
        SELECT id, instance_id, email, password_hash, is_bot, is_instance_admin,
               created_at, updated_at, deleted_at
        FROM accounts
        WHERE id = $1 AND deleted_at IS NULL
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(AccountRow::into_account))
}

async fn find_account_by_email(pool: &PgPool, email: &str) -> Result<Option<Account>, AuthError> {
    let row = sqlx::query_as::<_, AccountRow>(
        r"
        SELECT id, instance_id, email, password_hash, is_bot, is_instance_admin,
               created_at, updated_at, deleted_at
        FROM accounts
        WHERE email = $1 AND deleted_at IS NULL
        ",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(AccountRow::into_account))
}

async fn fetch_account_by_email_tx(
    tx: &mut Transaction<'_, Postgres>,
    email: &str,
) -> Result<Option<Account>, AuthError> {
    let row = sqlx::query_as::<_, AccountRow>(
        r"
        SELECT id, instance_id, email, password_hash, is_bot, is_instance_admin,
               created_at, updated_at, deleted_at
        FROM accounts
        WHERE email = $1 AND deleted_at IS NULL
        ",
    )
    .bind(email)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row.map(AccountRow::into_account))
}

async fn acquire_bootstrap_lock(tx: &mut Transaction<'_, Postgres>) -> Result<(), AuthError> {
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(INSTANCE_BOOTSTRAP_ADVISORY_LOCK)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn instance_admin_exists(tx: &mut Transaction<'_, Postgres>) -> Result<bool, AuthError> {
    sqlx::query_scalar(
        r"
        SELECT EXISTS(
            SELECT 1 FROM accounts
            WHERE is_instance_admin = TRUE AND deleted_at IS NULL
        )
        ",
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AuthError::from)
}

fn normalize_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

#[derive(Debug, sqlx::FromRow)]
struct AccountRow {
    id: Uuid,
    instance_id: Uuid,
    email: Option<String>,
    password_hash: Option<String>,
    is_bot: bool,
    is_instance_admin: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl AccountRow {
    fn into_account(self) -> Account {
        Account {
            id: self.id,
            instance_id: self.instance_id,
            email: self.email,
            password_hash: self.password_hash,
            is_bot: self.is_bot,
            is_instance_admin: self.is_instance_admin,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SessionAccountRow {
    session_id: Uuid,
    account_id: Uuid,
    #[allow(dead_code)]
    expires_at: DateTime<Utc>,
    user_agent: Option<String>,
    session_created_at: DateTime<Utc>,
    #[allow(dead_code)]
    last_seen_at: DateTime<Utc>,
    account_id_full: Uuid,
    instance_id: Uuid,
    email: Option<String>,
    password_hash: Option<String>,
    is_bot: bool,
    is_instance_admin: bool,
    account_created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

/// Re-export duration helper for callers that want the default TTL.
#[must_use]
pub fn session_ttl() -> Duration {
    SESSION_TTL
}
