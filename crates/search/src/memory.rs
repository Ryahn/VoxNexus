//! In-memory search engine for unit tests (not production).

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use serde_json::Value;

use crate::schema::all_collection_schemas;
use crate::{SearchEngine, SearchError};

/// Process-local fake that always pings ok and stores documents in memory.
#[derive(Debug, Default)]
pub struct MemorySearchEngine {
    ready: Mutex<bool>,
    docs: Mutex<HashMap<String, HashMap<String, Value>>>,
}

impl MemorySearchEngine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Ready for `/ready` without calling [`SearchEngine::ensure_collections`].
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[must_use]
    pub fn new_ready() -> Self {
        let engine = Self::new();
        *engine.ready.lock().expect("memory search lock") = true;
        for schema in all_collection_schemas() {
            engine
                .docs
                .lock()
                .expect("memory search lock")
                .insert(schema.name, HashMap::new());
        }
        engine
    }
}

#[async_trait]
impl SearchEngine for MemorySearchEngine {
    async fn ping(&self) -> Result<(), SearchError> {
        if *self.ready.lock().expect("memory search lock") {
            Ok(())
        } else {
            Err(SearchError::NotReady("collections not ensured".into()))
        }
    }

    async fn ensure_collections(&self) -> Result<(), SearchError> {
        let mut docs = self.docs.lock().expect("memory search lock");
        for schema in all_collection_schemas() {
            docs.entry(schema.name).or_default();
        }
        *self.ready.lock().expect("memory search lock") = true;
        Ok(())
    }

    async fn upsert_document(&self, collection: &str, document: Value) -> Result<(), SearchError> {
        let id = document
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| SearchError::InvalidDocument("missing id".into()))?
            .to_owned();
        let mut docs = self.docs.lock().expect("memory search lock");
        let bucket = docs
            .get_mut(collection)
            .ok_or_else(|| SearchError::NotReady(format!("unknown collection {collection}")))?;
        bucket.insert(id, document);
        Ok(())
    }

    async fn delete_document(&self, collection: &str, id: &str) -> Result<(), SearchError> {
        if let Some(bucket) = self
            .docs
            .lock()
            .expect("memory search lock")
            .get_mut(collection)
        {
            bucket.remove(id);
        }
        Ok(())
    }

    async fn search(
        &self,
        collection: &str,
        query: &str,
        query_by: &str,
    ) -> Result<Vec<String>, SearchError> {
        let docs = self.docs.lock().expect("memory search lock");
        let Some(bucket) = docs.get(collection) else {
            return Ok(Vec::new());
        };
        let needle = query.to_ascii_lowercase();
        let mut hits = Vec::new();
        for (id, doc) in bucket {
            let hay = doc
                .get(query_by)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_ascii_lowercase();
            if hay.contains(&needle) {
                hits.push(id.clone());
            }
        }
        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{COLLECTION_MESSAGES, SCHEMA_VERSION};
    use serde_json::json;

    #[tokio::test]
    async fn memory_upsert_search_delete() {
        let engine = MemorySearchEngine::new();
        engine.ensure_collections().await.expect("ensure");
        engine
            .upsert_document(
                COLLECTION_MESSAGES,
                json!({
                    "id": "m1",
                    "community_id": "c1",
                    "channel_id": "ch1",
                    "author_id": "u1",
                    "body": "hello voxnexus",
                    "created_at": 1,
                    "schema_version": SCHEMA_VERSION,
                }),
            )
            .await
            .expect("upsert");
        let hits = engine
            .search(COLLECTION_MESSAGES, "voxnexus", "body")
            .await
            .expect("search");
        assert_eq!(hits, vec!["m1".to_owned()]);
        engine
            .delete_document(COLLECTION_MESSAGES, "m1")
            .await
            .expect("delete");
        let hits = engine
            .search(COLLECTION_MESSAGES, "voxnexus", "body")
            .await
            .expect("search after delete");
        assert!(hits.is_empty());
    }
}
