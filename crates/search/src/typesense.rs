//! Typesense HTTP client (`reqwest` + API key).

use std::env;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};
use url::Url;

use crate::schema::all_collection_schemas;
use crate::{SearchEngine, SearchError};

/// Environment variable that enables live Typesense integration tests.
pub const TEST_TYPESENSE_URL_ENV: &str = "TYPESENSE_URL_TEST";
/// Optional API key for live tests (defaults to `xyz`).
pub const TEST_TYPESENSE_API_KEY_ENV: &str = "TYPESENSE_API_KEY_TEST";

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Connection settings for Typesense.
#[derive(Debug, Clone)]
pub struct TypesenseConfig {
    pub base_url: Url,
    pub api_key: String,
}

/// Read live-test config. `None` means skip integration tests.
#[must_use]
pub fn test_typesense_config() -> Option<TypesenseConfig> {
    let url = env::var(TEST_TYPESENSE_URL_ENV).ok()?;
    let url = url.trim();
    if url.is_empty() {
        return None;
    }
    let base_url = Url::parse(url).ok()?;
    let api_key = env::var(TEST_TYPESENSE_API_KEY_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| env::var("TYPESENSE_API_KEY").ok())
        .unwrap_or_else(|| "xyz".to_owned());
    Some(TypesenseConfig { base_url, api_key })
}

/// Production Typesense backend.
#[derive(Debug, Clone)]
pub struct TypesenseClient {
    http: Client,
    base_url: Url,
    api_key: String,
}

impl TypesenseClient {
    /// Build a client with connect/request timeouts.
    ///
    /// # Errors
    ///
    /// Returns when the HTTP client cannot be constructed.
    pub fn new(config: TypesenseConfig) -> Result<Self, SearchError> {
        let http = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()?;
        Ok(Self {
            http,
            base_url: config.base_url,
            api_key: config.api_key,
        })
    }

    fn url(&self, path: &str) -> Result<Url, SearchError> {
        let base = self.base_url.as_str().trim_end_matches('/');
        let path = if path.starts_with('/') {
            path.to_owned()
        } else {
            format!("/{path}")
        };
        Url::parse(&format!("{base}{path}"))
            .map_err(|error| SearchError::InvalidDocument(error.to_string()))
    }

    async fn ensure_ok(
        &self,
        response: reqwest::Response,
    ) -> Result<reqwest::Response, SearchError> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let body = response.text().await.unwrap_or_default();
        Err(SearchError::Api {
            status: status.as_u16(),
            body,
        })
    }
}

#[derive(Debug, Deserialize)]
struct HealthBody {
    ok: bool,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    hits: Vec<SearchHit>,
}

#[derive(Debug, Deserialize)]
struct SearchHit {
    document: SearchHitDoc,
}

#[derive(Debug, Deserialize)]
struct SearchHitDoc {
    id: String,
}

#[async_trait]
impl SearchEngine for TypesenseClient {
    async fn ping(&self) -> Result<(), SearchError> {
        let url = self.url("/health")?;
        let response = self
            .http
            .get(url)
            .header("X-TYPESENSE-API-KEY", &self.api_key)
            .send()
            .await?;
        let response = self.ensure_ok(response).await?;
        let body: HealthBody = response.json().await?;
        if body.ok {
            Ok(())
        } else {
            Err(SearchError::NotReady(
                "Typesense health reported not ok".into(),
            ))
        }
    }

    async fn ensure_collections(&self) -> Result<(), SearchError> {
        for schema in all_collection_schemas() {
            let get_url = self.url(&format!("/collections/{}", schema.name))?;
            let existing = self
                .http
                .get(get_url)
                .header("X-TYPESENSE-API-KEY", &self.api_key)
                .send()
                .await?;
            let existing_status = existing.status();
            if existing_status == StatusCode::OK {
                continue;
            }
            if existing_status != StatusCode::NOT_FOUND {
                let body = existing.text().await.unwrap_or_default();
                return Err(SearchError::Api {
                    status: existing_status.as_u16(),
                    body,
                });
            }
            let create_url = self.url("/collections")?;
            let response = self
                .http
                .post(create_url)
                .header("X-TYPESENSE-API-KEY", &self.api_key)
                .json(&schema)
                .send()
                .await?;
            // Concurrent create can race to 409 — treat as success.
            if response.status() == StatusCode::CONFLICT {
                continue;
            }
            self.ensure_ok(response).await?;
            tracing::info!(collection = %schema.name, "typesense collection ensured");
        }
        Ok(())
    }

    async fn upsert_document(&self, collection: &str, document: Value) -> Result<(), SearchError> {
        if document.get("id").and_then(Value::as_str).is_none() {
            return Err(SearchError::InvalidDocument("missing id".into()));
        }
        let url = self.url(&format!("/collections/{collection}/documents"))?;
        let response = self
            .http
            .post(url)
            .query(&[("action", "upsert")])
            .header("X-TYPESENSE-API-KEY", &self.api_key)
            .json(&document)
            .send()
            .await?;
        self.ensure_ok(response).await?;
        Ok(())
    }

    async fn delete_document(&self, collection: &str, id: &str) -> Result<(), SearchError> {
        let url = self.url(&format!("/collections/{collection}/documents/{id}"))?;
        let response = self
            .http
            .delete(url)
            .header("X-TYPESENSE-API-KEY", &self.api_key)
            .send()
            .await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(());
        }
        self.ensure_ok(response).await?;
        Ok(())
    }

    async fn search(
        &self,
        collection: &str,
        query: &str,
        query_by: &str,
    ) -> Result<Vec<String>, SearchError> {
        let url = self.url(&format!("/collections/{collection}/documents/search"))?;
        let response = self
            .http
            .get(url)
            .query(&[("q", query), ("query_by", query_by), ("per_page", "20")])
            .header("X-TYPESENSE-API-KEY", &self.api_key)
            .send()
            .await?;
        let response = self.ensure_ok(response).await?;
        let body: SearchResponse = response.json().await?;
        Ok(body.hits.into_iter().map(|hit| hit.document.id).collect())
    }
}

/// Convenience: build a probe document for the messages collection.
#[must_use]
pub fn probe_message_document(id: &str, body: &str) -> Value {
    json!({
        "id": id,
        "community_id": "vn-probe",
        "channel_id": "vn-probe",
        "author_id": "vn-probe",
        "body": body,
        "created_at": 0_i64,
        "schema_version": crate::SCHEMA_VERSION,
    })
}
