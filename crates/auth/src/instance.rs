//! Singleton instance row persistence.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use voxnexus_domain::{CommunityCreationMode, Instance, RegistrationMode, DEFAULT_INSTANCE_ID};

/// Instance persistence errors.
#[derive(Debug, thiserror::Error)]
pub enum InstanceError {
    #[error("invalid instance settings in database")]
    InvalidRow,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// Values used to seed the singleton row on first boot.
#[derive(Debug, Clone)]
pub struct InstanceSeed {
    pub name: String,
    pub public_url: String,
    pub registration_mode: RegistrationMode,
    pub community_creation_mode: CommunityCreationMode,
    pub oidc_enabled: bool,
    pub oidc_issuer: Option<String>,
    pub oidc_client_id: Option<String>,
}

/// Partial instance settings update from an admin.
#[derive(Debug, Clone, Default)]
pub struct InstancePatch {
    pub name: Option<String>,
    pub public_url: Option<String>,
    pub registration_mode: Option<RegistrationMode>,
    pub community_creation_mode: Option<CommunityCreationMode>,
    pub oidc_enabled: Option<bool>,
    pub oidc_issuer: Option<Option<String>>,
    pub oidc_client_id: Option<Option<String>>,
}

/// Insert the singleton instance row when missing.
///
/// # Errors
///
/// Returns database errors if the insert fails.
pub async fn ensure_instance(pool: &PgPool, seed: &InstanceSeed) -> Result<(), InstanceError> {
    let now = Utc::now();
    sqlx::query(
        r"
        INSERT INTO instances (
            id, name, public_url, registration_mode, community_creation_mode,
            oidc_enabled, oidc_issuer, oidc_client_id, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
        ON CONFLICT (id) DO NOTHING
        ",
    )
    .bind(DEFAULT_INSTANCE_ID)
    .bind(&seed.name)
    .bind(&seed.public_url)
    .bind(seed.registration_mode.as_str())
    .bind(seed.community_creation_mode.as_str())
    .bind(seed.oidc_enabled)
    .bind(&seed.oidc_issuer)
    .bind(&seed.oidc_client_id)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

/// Apply operator-configured community creation mode when PATCH is locked.
///
/// Called on every startup after [`ensure_instance`] so `COMMUNITY_CREATION_MODE` in
/// config/env stays authoritative without wiping the database.
///
/// # Errors
///
/// Returns database errors, or [`InstanceError::InvalidRow`] if the singleton row is missing.
pub async fn sync_locked_community_creation_mode(
    pool: &PgPool,
    mode: CommunityCreationMode,
) -> Result<(), InstanceError> {
    let now = Utc::now();
    let result = sqlx::query(
        r"
        UPDATE instances
        SET community_creation_mode = $2, updated_at = $3
        WHERE id = $1
        ",
    )
    .bind(DEFAULT_INSTANCE_ID)
    .bind(mode.as_str())
    .bind(now)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(InstanceError::Db(sqlx::Error::RowNotFound));
    }
    Ok(())
}

/// Load the singleton instance row.
///
/// # Errors
///
/// Returns [`InstanceError::InvalidRow`] when stored enums are unknown, or database errors.
pub async fn get_instance(pool: &PgPool) -> Result<Instance, InstanceError> {
    let row = sqlx::query_as::<_, InstanceRow>(
        r"
        SELECT id, name, public_url, registration_mode, community_creation_mode,
               oidc_enabled, oidc_issuer, oidc_client_id, created_at, updated_at
        FROM instances
        WHERE id = $1
        ",
    )
    .bind(DEFAULT_INSTANCE_ID)
    .fetch_optional(pool)
    .await?;

    row.map(InstanceRow::into_instance)
        .transpose()?
        .ok_or_else(|| InstanceError::Db(sqlx::Error::RowNotFound))
}

/// Apply a partial update to the singleton instance row.
///
/// # Errors
///
/// Returns database errors or [`InstanceError::InvalidRow`] when the row cannot be read back.
pub async fn update_instance(
    pool: &PgPool,
    patch: InstancePatch,
) -> Result<Instance, InstanceError> {
    let current = get_instance(pool).await?;
    let now = Utc::now();
    let name = patch.name.unwrap_or_else(|| current.name.clone());
    let public_url = patch
        .public_url
        .unwrap_or_else(|| current.public_url.clone());
    let registration_mode = patch.registration_mode.unwrap_or(current.registration_mode);
    let community_creation_mode = patch
        .community_creation_mode
        .unwrap_or(current.community_creation_mode);
    let oidc_enabled = patch.oidc_enabled.unwrap_or(current.oidc_enabled);
    let oidc_issuer = match patch.oidc_issuer {
        Some(value) => value,
        None => current.oidc_issuer.clone(),
    };
    let oidc_client_id = match patch.oidc_client_id {
        Some(value) => value,
        None => current.oidc_client_id.clone(),
    };

    sqlx::query(
        r"
        UPDATE instances
        SET name = $2,
            public_url = $3,
            registration_mode = $4,
            community_creation_mode = $5,
            oidc_enabled = $6,
            oidc_issuer = $7,
            oidc_client_id = $8,
            updated_at = $9
        WHERE id = $1
        ",
    )
    .bind(DEFAULT_INSTANCE_ID)
    .bind(&name)
    .bind(&public_url)
    .bind(registration_mode.as_str())
    .bind(community_creation_mode.as_str())
    .bind(oidc_enabled)
    .bind(&oidc_issuer)
    .bind(&oidc_client_id)
    .bind(now)
    .execute(pool)
    .await?;

    get_instance(pool).await
}

#[derive(Debug, sqlx::FromRow)]
struct InstanceRow {
    id: Uuid,
    name: String,
    public_url: String,
    registration_mode: String,
    community_creation_mode: String,
    oidc_enabled: bool,
    oidc_issuer: Option<String>,
    oidc_client_id: Option<String>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl InstanceRow {
    fn into_instance(self) -> Result<Instance, InstanceError> {
        let registration_mode =
            RegistrationMode::parse(&self.registration_mode).ok_or(InstanceError::InvalidRow)?;
        let community_creation_mode = CommunityCreationMode::parse(&self.community_creation_mode)
            .ok_or(InstanceError::InvalidRow)?;
        Ok(Instance {
            id: self.id,
            name: self.name,
            public_url: self.public_url,
            registration_mode,
            community_creation_mode,
            oidc_enabled: self.oidc_enabled,
            oidc_issuer: self.oidc_issuer,
            oidc_client_id: self.oidc_client_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
