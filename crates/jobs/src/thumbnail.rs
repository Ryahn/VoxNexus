//! Redis thumbnail job (F038).

use apalis::prelude::{Error as ApalisError, Storage};
use apalis_redis::RedisStorage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::JobsError;

/// Generate a thumbnail for a message attachment (payload is an id only).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThumbnailJob {
    pub attachment_id: Uuid,
}

impl ThumbnailJob {
    #[must_use]
    pub fn new(attachment_id: Uuid) -> Self {
        Self { attachment_id }
    }
}

/// Apalis no-op handler placeholder — real work runs in the server worker with storage/DB.
///
/// # Errors
///
/// Never fails; the composition root registers a Data-backed handler instead.
pub async fn process_thumbnail_stub(job: ThumbnailJob) -> Result<(), ApalisError> {
    tracing::debug!(attachment_id = %job.attachment_id, "thumbnail stub (use server worker)");
    Ok(())
}

/// Push a [`ThumbnailJob`] onto the Redis queue.
///
/// # Errors
///
/// Returns when Redis rejects the push.
pub async fn enqueue_thumbnail(
    storage: &mut RedisStorage<ThumbnailJob>,
    job: ThumbnailJob,
) -> Result<(), JobsError> {
    storage
        .push(job)
        .await
        .map_err(|error| JobsError::Enqueue(error.to_string()))?;
    Ok(())
}
