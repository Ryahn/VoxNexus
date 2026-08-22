//! Redis connectivity for Apalis storage and readiness checks.

use std::env;

use apalis_redis::{connect as apalis_connect, ConnectionManager};

use crate::JobsError;

/// Environment variable that enables live Redis integration tests.
pub const TEST_REDIS_URL_ENV: &str = "REDIS_URL_TEST";

/// Multiplexed Redis connection used by workers and `/ready`.
pub type RedisConn = ConnectionManager;

/// Read `REDIS_URL_TEST`. `None` means skip live Redis tests.
#[must_use]
pub fn test_redis_url() -> Option<String> {
    env::var(TEST_REDIS_URL_ENV).ok().and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}

/// Open a reconnecting Redis connection manager.
///
/// # Errors
///
/// Returns when the URL is invalid or Redis cannot be reached.
pub async fn connect(redis_url: &str) -> Result<RedisConn, JobsError> {
    Ok(apalis_connect(redis_url).await?)
}

/// `PING` health check.
///
/// # Errors
///
/// Returns when Redis does not reply `PONG`.
pub async fn ping(conn: &RedisConn) -> Result<(), JobsError> {
    let mut conn = conn.clone();
    let pong: String = redis::cmd("PING").query_async(&mut conn).await?;
    if pong.eq_ignore_ascii_case("pong") {
        Ok(())
    } else {
        Err(JobsError::Message(format!("unexpected PING reply: {pong}")))
    }
}
