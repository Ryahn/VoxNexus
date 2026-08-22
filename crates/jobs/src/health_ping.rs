//! Sample `HealthPing` job — payload is a small ID, never a blob.

use apalis::prelude::{Error as ApalisError, Storage};
use apalis_redis::RedisStorage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::JobsError;

/// Typed sample job used to prove the Redis worker path.
///
/// Later jobs (thumbnails, unfurl, index) follow the same shape: IDs into Postgres,
/// not attachment bytes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthPing {
    pub id: String,
}

impl HealthPing {
    /// Fresh ping with a UUIDv7 id.
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: Uuid::now_v7().to_string(),
        }
    }
}

impl Default for HealthPing {
    fn default() -> Self {
        Self::new()
    }
}

/// Apalis handler: log and succeed.
///
/// # Errors
///
/// Never fails in the sample path; real jobs return [`ApalisError`].
pub async fn process_health_ping(job: HealthPing) -> Result<(), ApalisError> {
    tracing::info!(job_id = %job.id, "health ping processed");
    Ok(())
}

/// Push a [`HealthPing`] onto the Redis queue.
///
/// # Errors
///
/// Returns when Redis rejects the push.
pub async fn enqueue_health_ping(
    storage: &mut RedisStorage<HealthPing>,
    job: HealthPing,
) -> Result<(), JobsError> {
    storage
        .push(job)
        .await
        .map_err(|error| JobsError::Enqueue(error.to_string()))?;
    Ok(())
}
