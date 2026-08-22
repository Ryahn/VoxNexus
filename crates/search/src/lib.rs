//! Typesense search trait and client (Feature Task F008S).

mod error;
mod memory;
mod schema;
mod typesense;

pub use error::SearchError;
pub use memory::MemorySearchEngine;
pub use schema::{
    all_collection_schemas, CollectionSchema, COLLECTION_CHANNELS, COLLECTION_MESSAGES,
    COLLECTION_USERS, SCHEMA_VERSION,
};
pub use typesense::{
    probe_message_document, test_typesense_config, TypesenseClient, TypesenseConfig,
    TEST_TYPESENSE_API_KEY_ENV, TEST_TYPESENSE_URL_ENV,
};

use async_trait::async_trait;
use serde_json::Value;

/// Abstraction over the derived search index (Typesense in production).
#[async_trait]
pub trait SearchEngine: Send + Sync {
    /// `GET /health` (or equivalent) against the search backend.
    async fn ping(&self) -> Result<(), SearchError>;

    /// Create versioned collections when missing (messages, users, channels).
    async fn ensure_collections(&self) -> Result<(), SearchError>;

    /// Upsert a JSON document into `collection` (must include `id`).
    async fn upsert_document(&self, collection: &str, document: Value) -> Result<(), SearchError>;

    /// Delete a document by id (idempotent if missing).
    async fn delete_document(&self, collection: &str, id: &str) -> Result<(), SearchError>;

    /// Simple keyword search; returns matching document ids.
    async fn search(
        &self,
        collection: &str,
        query: &str,
        query_by: &str,
    ) -> Result<Vec<String>, SearchError>;
}

pub const CRATE_NAME: &str = "voxnexus-search";
