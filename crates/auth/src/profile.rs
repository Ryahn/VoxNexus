//! Profile and object metadata persistence.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use voxnexus_domain::{ObjectMeta, Profile};

use crate::AuthError;

/// Ensure a profile row exists for `account_id`.
///
/// # Errors
///
/// Returns database errors.
pub async fn ensure_profile(pool: &PgPool, account_id: Uuid) -> Result<Profile, AuthError> {
    sqlx::query(
        r"
        INSERT INTO profiles (account_id)
        VALUES ($1)
        ON CONFLICT (account_id) DO NOTHING
        ",
    )
    .bind(account_id)
    .execute(pool)
    .await?;
    get_profile(pool, account_id)
        .await?
        .ok_or_else(|| AuthError::Db(sqlx::Error::RowNotFound))
}

/// Load a profile by account id.
///
/// # Errors
///
/// Returns database errors.
pub async fn get_profile(pool: &PgPool, account_id: Uuid) -> Result<Option<Profile>, AuthError> {
    let row = sqlx::query_as::<_, ProfileRow>(
        r"
        SELECT account_id, display_name, bio, avatar_object_id, banner_object_id, updated_at
        FROM profiles
        WHERE account_id = $1
        ",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(ProfileRow::into_profile))
}

/// Update display name and/or bio.
///
/// # Errors
///
/// Returns database errors.
pub async fn update_profile(
    pool: &PgPool,
    account_id: Uuid,
    display_name: Option<&str>,
    bio: Option<&str>,
) -> Result<Profile, AuthError> {
    ensure_profile(pool, account_id).await?;
    let now = Utc::now();
    sqlx::query(
        r"
        UPDATE profiles
        SET
            display_name = COALESCE($2, display_name),
            bio = COALESCE($3, bio),
            updated_at = $4
        WHERE account_id = $1
        ",
    )
    .bind(account_id)
    .bind(display_name)
    .bind(bio)
    .bind(now)
    .execute(pool)
    .await?;
    get_profile(pool, account_id)
        .await?
        .ok_or_else(|| AuthError::Db(sqlx::Error::RowNotFound))
}

/// Insert object metadata.
///
/// # Errors
///
/// Returns database errors.
pub async fn insert_object(
    pool: &PgPool,
    id: Uuid,
    storage_key: &str,
    sha256: &[u8],
    mime: &str,
    byte_size: i64,
    created_by: Uuid,
) -> Result<ObjectMeta, AuthError> {
    let now = Utc::now();
    sqlx::query(
        r"
        INSERT INTO objects (id, storage_key, sha256, mime, byte_size, created_by, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ",
    )
    .bind(id)
    .bind(storage_key)
    .bind(sha256)
    .bind(mime)
    .bind(byte_size)
    .bind(created_by)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(ObjectMeta {
        id,
        storage_key: storage_key.to_owned(),
        mime: mime.to_owned(),
        byte_size,
        created_by,
        created_at: now,
    })
}

/// Load object metadata.
///
/// # Errors
///
/// Returns database errors.
pub async fn get_object(pool: &PgPool, id: Uuid) -> Result<Option<ObjectMeta>, AuthError> {
    let row = sqlx::query_as::<_, ObjectRow>(
        r"
        SELECT id, storage_key, mime, byte_size, created_by, created_at
        FROM objects
        WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(ObjectRow::into_meta))
}

/// Set avatar object id for the account. Returns previous object id if any.
///
/// # Errors
///
/// Returns database errors.
pub async fn set_avatar_object(
    pool: &PgPool,
    account_id: Uuid,
    object_id: Uuid,
) -> Result<Option<Uuid>, AuthError> {
    ensure_profile(pool, account_id).await?;
    let previous = sqlx::query_scalar::<_, Option<Uuid>>(
        r"
        SELECT avatar_object_id FROM profiles WHERE account_id = $1
        ",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await?;
    let now = Utc::now();
    sqlx::query(
        r"
        UPDATE profiles
        SET avatar_object_id = $2, updated_at = $3
        WHERE account_id = $1
        ",
    )
    .bind(account_id)
    .bind(object_id)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(previous)
}

/// Set banner object id for the account. Returns previous object id if any.
///
/// # Errors
///
/// Returns database errors.
pub async fn set_banner_object(
    pool: &PgPool,
    account_id: Uuid,
    object_id: Uuid,
) -> Result<Option<Uuid>, AuthError> {
    ensure_profile(pool, account_id).await?;
    let previous = sqlx::query_scalar::<_, Option<Uuid>>(
        r"
        SELECT banner_object_id FROM profiles WHERE account_id = $1
        ",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await?;
    let now = Utc::now();
    sqlx::query(
        r"
        UPDATE profiles
        SET banner_object_id = $2, updated_at = $3
        WHERE account_id = $1
        ",
    )
    .bind(account_id)
    .bind(object_id)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(previous)
}

/// Delete object metadata row (storage delete is caller's job).
///
/// # Errors
///
/// Returns database errors.
pub async fn delete_object_meta(pool: &PgPool, id: Uuid) -> Result<(), AuthError> {
    sqlx::query("DELETE FROM objects WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
struct ProfileRow {
    account_id: Uuid,
    display_name: String,
    bio: String,
    avatar_object_id: Option<Uuid>,
    banner_object_id: Option<Uuid>,
    updated_at: DateTime<Utc>,
}

impl ProfileRow {
    fn into_profile(self) -> Profile {
        Profile {
            account_id: self.account_id,
            display_name: self.display_name,
            bio: self.bio,
            avatar_object_id: self.avatar_object_id,
            banner_object_id: self.banner_object_id,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ObjectRow {
    id: Uuid,
    storage_key: String,
    mime: String,
    byte_size: i64,
    created_by: Uuid,
    created_at: DateTime<Utc>,
}

impl ObjectRow {
    fn into_meta(self) -> ObjectMeta {
        ObjectMeta {
            id: self.id,
            storage_key: self.storage_key,
            mime: self.mime,
            byte_size: self.byte_size,
            created_by: self.created_by,
            created_at: self.created_at,
        }
    }
}
