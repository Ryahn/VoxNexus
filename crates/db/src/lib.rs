//! PostgreSQL pool, versioned SQLx migrations, and connectivity checks.

use std::env;
use std::time::Duration;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

pub use sqlx::migrate::Migrator;
pub use sqlx::PgPool;

/// Embedded migrator for the repository-root `migrations/` directory.
pub static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

/// Environment variable for live Postgres tests. Unset tests skip instead of failing.
pub const TEST_DATABASE_URL_ENV: &str = "DATABASE_URL_TEST";

const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_CONNECTIONS: u32 = 10;

/// Database connection or migration failure.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("PostgreSQL error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("failed to run database migrations: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

/// Read `DATABASE_URL_TEST` for integration tests. `None` means skip live DB tests.
#[must_use]
pub fn test_database_url() -> Option<String> {
    env::var(TEST_DATABASE_URL_ENV).ok().and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

/// Open a PostgreSQL pool with connect and acquire timeouts.
///
/// # Errors
///
/// Returns an error when the URL is invalid or PostgreSQL cannot be reached
/// within the pool acquire timeout.
pub async fn connect(database_url: &str) -> Result<PgPool, DbError> {
    let options: PgConnectOptions = database_url.parse()?;
    let options = options.application_name("voxnexus");
    let pool = PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect_with(options)
        .await?;
    Ok(pool)
}

/// Apply all pending migrations from [`MIGRATOR`].
///
/// # Errors
///
/// Returns an error when a migration fails to apply.
pub async fn migrate(pool: &PgPool) -> Result<(), DbError> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

/// Connect and apply migrations.
///
/// # Errors
///
/// Returns an error when the pool cannot be opened or migrations fail.
pub async fn connect_and_migrate(database_url: &str) -> Result<PgPool, DbError> {
    let pool = connect(database_url).await?;
    migrate(&pool).await?;
    Ok(pool)
}

/// `SELECT 1` health check.
///
/// # Errors
///
/// Returns an error when the query fails or does not return `1`.
pub async fn ping(pool: &PgPool) -> Result<(), DbError> {
    let value: i32 = sqlx::query_scalar("SELECT 1").fetch_one(pool).await?;
    if value == 1 {
        Ok(())
    } else {
        Err(sqlx::Error::Protocol(format!("unexpected ping result: {value}")).into())
    }
}

/// Revert migrations down to `target` version (`0` reverts all).
///
/// # Errors
///
/// Returns an error when revert fails.
pub async fn revert_to(pool: &PgPool, target: i64) -> Result<(), DbError> {
    MIGRATOR.undo(pool, target).await?;
    Ok(())
}
