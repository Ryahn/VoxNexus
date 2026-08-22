//! Storage errors.

use thiserror::Error;

/// Object storage failure.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("invalid object key: {0}")]
    InvalidKey(String),
    #[error("object not found: {0}")]
    NotFound(String),
    #[error("presign is not supported by this store")]
    PresignUnsupported,
    #[error("S3 error: {0}")]
    S3(String),
}
