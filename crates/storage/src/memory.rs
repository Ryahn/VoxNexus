//! In-memory object store for unit tests.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use url::Url;

use crate::{ObjectKey, ObjectStore, StorageError, StoredObject};

#[derive(Debug, Clone)]
struct Entry {
    bytes: Bytes,
    #[allow(dead_code)]
    content_type: String,
}

/// Process-local store. Never used in Compose or production.
#[derive(Debug, Default)]
pub struct MemoryObjectStore {
    objects: Mutex<HashMap<String, Entry>>,
    bucket_ready: Mutex<bool>,
}

impl MemoryObjectStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Like [`Self::new`] but already passing [`ObjectStore::head_bucket`].
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[must_use]
    pub fn new_ready() -> Self {
        let store = Self::new();
        *store.bucket_ready.lock().expect("memory store lock") = true;
        store
    }
}

#[async_trait]
impl ObjectStore for MemoryObjectStore {
    async fn put(
        &self,
        key: ObjectKey,
        bytes: Bytes,
        content_type: &str,
    ) -> Result<StoredObject, StorageError> {
        let byte_size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        let content_type = content_type.to_owned();
        self.objects.lock().expect("memory store lock").insert(
            key.as_str().to_owned(),
            Entry {
                bytes,
                content_type: content_type.clone(),
            },
        );
        Ok(StoredObject {
            key,
            content_type,
            byte_size,
        })
    }

    async fn get(&self, key: &ObjectKey) -> Result<Bytes, StorageError> {
        self.objects
            .lock()
            .expect("memory store lock")
            .get(key.as_str())
            .map(|entry| entry.bytes.clone())
            .ok_or_else(|| StorageError::NotFound(key.as_str().to_owned()))
    }

    async fn delete(&self, key: &ObjectKey) -> Result<(), StorageError> {
        self.objects
            .lock()
            .expect("memory store lock")
            .remove(key.as_str());
        Ok(())
    }

    async fn presign_get(&self, _key: &ObjectKey, _ttl: Duration) -> Result<Url, StorageError> {
        Err(StorageError::PresignUnsupported)
    }

    async fn head_bucket(&self) -> Result<(), StorageError> {
        if *self.bucket_ready.lock().expect("memory store lock") {
            Ok(())
        } else {
            Err(StorageError::S3("bucket not ensured".into()))
        }
    }

    async fn ensure_bucket(&self) -> Result<(), StorageError> {
        *self.bucket_ready.lock().expect("memory store lock") = true;
        self.head_bucket().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn put_get_delete_round_trip() {
        let store = MemoryObjectStore::new();
        store.ensure_bucket().await.expect("ensure");
        let key = ObjectKey::parse("vn/2026/test-obj").expect("key");
        store
            .put(key.clone(), Bytes::from_static(b"hello"), "text/plain")
            .await
            .expect("put");
        let got = store.get(&key).await.expect("get");
        assert_eq!(got.as_ref(), b"hello");
        store.delete(&key).await.expect("delete");
        assert!(matches!(
            store.get(&key).await,
            Err(StorageError::NotFound(_))
        ));
    }
}
