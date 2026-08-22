//! Search / Typesense errors.

/// Failures talking to Typesense or validating documents.
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("Typesense HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Typesense returned {status}: {body}")]
    Api { status: u16, body: String },
    #[error("invalid search document: {0}")]
    InvalidDocument(String),
    #[error("search engine not ready: {0}")]
    NotReady(String),
}
