//! OIDC identity linking and JIT account creation (F018O).

use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use voxnexus_domain::{Account, DEFAULT_INSTANCE_ID};

use crate::profile::ensure_profile_with_name;
use crate::{insert_auth_identity, AuthError};

/// Verified OIDC subject claims used to resolve or create a local account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OidcIdentity {
    pub issuer: String,
    pub subject: String,
    pub email: Option<String>,
    pub email_verified: bool,
}

/// Resolve an OIDC login to a local account.
///
/// Order: existing `(issuer, subject)` link, optional verified-email link, then JIT when allowed.
///
/// # Errors
///
/// Returns [`AuthError::RegistrationClosed`] when no account can be resolved, identity conflicts,
/// or database errors.
pub async fn resolve_oidc_login(
    pool: &PgPool,
    identity: &OidcIdentity,
    link_by_email: bool,
    allow_jit: bool,
) -> Result<Account, AuthError> {
    if let Some(account) =
        find_account_by_auth_identity(pool, &identity.issuer, &identity.subject).await?
    {
        return Ok(account);
    }

    if link_by_email && identity.email_verified {
        if let Some(email) = identity.email.as_deref() {
            let email = normalize_email(email);
            if let Some(account) = find_account_by_email(pool, &email).await? {
                insert_auth_identity(pool, account.id, &identity.issuer, &identity.subject).await?;
                return get_account(pool, account.id)
                    .await?
                    .ok_or_else(|| AuthError::Db(sqlx::Error::RowNotFound));
            }
        }
    }

    if allow_jit {
        return create_oidc_account(pool, identity).await;
    }

    Err(AuthError::RegistrationClosed)
}

async fn create_oidc_account(pool: &PgPool, identity: &OidcIdentity) -> Result<Account, AuthError> {
    let email = identity
        .email
        .as_deref()
        .map(normalize_email)
        .filter(|value| !value.is_empty());
    let now = Utc::now();
    let id = Uuid::now_v7();

    let mut tx = pool.begin().await?;
    let result = sqlx::query(
        r"
        INSERT INTO accounts (
            id, instance_id, email, password_hash, is_bot, is_instance_admin, created_at, updated_at
        ) VALUES ($1, $2, $3, NULL, FALSE, FALSE, $4, $4)
        ",
    )
    .bind(id)
    .bind(DEFAULT_INSTANCE_ID)
    .bind(&email)
    .bind(now)
    .execute(&mut *tx)
    .await;

    match result {
        Ok(_) => {}
        Err(sqlx::Error::Database(db)) if db.constraint() == Some("accounts_email_unique") => {
            return Err(AuthError::EmailTaken);
        }
        Err(error) => return Err(AuthError::Db(error)),
    }

    insert_auth_identity_tx(&mut tx, id, &identity.issuer, &identity.subject).await?;
    tx.commit().await?;
    let hint = email
        .as_deref()
        .and_then(|value| value.split('@').next())
        .unwrap_or("member");
    ensure_profile_with_name(pool, id, Some(hint)).await?;
    get_account(pool, id)
        .await?
        .ok_or_else(|| AuthError::Db(sqlx::Error::RowNotFound))
}

async fn insert_auth_identity_tx(
    tx: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    issuer: &str,
    subject: &str,
) -> Result<(), AuthError> {
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
    .execute(&mut **tx)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(sqlx::Error::Database(db))
            if db.constraint() == Some("auth_identities_issuer_subject_unique") =>
        {
            Err(AuthError::IdentityTaken)
        }
        Err(error) => Err(AuthError::Db(error)),
    }
}

async fn find_account_by_auth_identity(
    pool: &PgPool,
    issuer: &str,
    subject: &str,
) -> Result<Option<Account>, AuthError> {
    let row = sqlx::query_as::<_, AccountRow>(
        r"
        SELECT a.id, a.instance_id, a.email, a.password_hash, a.is_bot, a.is_instance_admin,
               a.created_at, a.updated_at, a.deleted_at
        FROM accounts a
        INNER JOIN auth_identities i ON i.account_id = a.id
        WHERE i.issuer = $1 AND i.subject = $2 AND a.deleted_at IS NULL
        ",
    )
    .bind(issuer)
    .bind(subject)
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

async fn get_account(pool: &PgPool, id: Uuid) -> Result<Option<Account>, AuthError> {
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
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
    deleted_at: Option<chrono::DateTime<Utc>>,
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
