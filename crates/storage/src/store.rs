//! Shared object-store trait.

use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use url::Url;

use crate::{ObjectKey, StorageError};

/// Result of a successful put.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredObject {
    pub key: ObjectKey,
    pub content_type: String,
    pub byte_size: u64,
}

/// Backend for attachment bytes (SeaweedFS S3 in production).
#[async_trait]
pub trait ObjectStore: Send + Sync {
    /// Store bytes under `key`.
    async fn put(
        &self,
        key: ObjectKey,
        bytes: Bytes,
        content_type: &str,
    ) -> Result<StoredObject, StorageError>;

    /// Fetch object bytes.
    async fn get(&self, key: &ObjectKey) -> Result<Bytes, StorageError>;

    /// Delete object (idempotent if missing).
    async fn delete(&self, key: &ObjectKey) -> Result<(), StorageError>;

    /// Short-lived GET URL when the store supports presigning.
    async fn presign_get(&self, key: &ObjectKey, ttl: Duration) -> Result<Url, StorageError>;

    /// Probe that the configured bucket is reachable.
    async fn head_bucket(&self) -> Result<(), StorageError>;

    /// Create the bucket when missing, then verify with [`Self::head_bucket`].
    async fn ensure_bucket(&self) -> Result<(), StorageError>;
}
