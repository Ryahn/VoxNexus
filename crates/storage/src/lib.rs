//! S3-compatible object storage (SeaweedFS in production, in-memory in tests).

mod error;
mod key;
mod memory;
mod s3;
mod store;

pub use error::StorageError;
pub use key::ObjectKey;
pub use memory::MemoryObjectStore;
pub use s3::{S3ObjectStore, S3ObjectStoreConfig};
pub use store::{ObjectStore, StoredObject};

/// Environment variable that enables live S3 integration tests when set to a non-empty endpoint URL.
pub const TEST_S3_ENDPOINT_ENV: &str = "S3_ENDPOINT_TEST";

/// Live S3 test settings when [`TEST_S3_ENDPOINT_ENV`] is set.
#[derive(Debug, Clone)]
pub struct TestS3Config {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
}

/// Read optional live S3 test configuration. `None` means skip integration tests.
///
/// Credential lookup order: `S3_*_TEST`, then app `S3_*` env vars, then `any` / `voxnexus-test`
/// (anonymous-friendly stores only — SeaweedFS with IAM needs real keys).
#[must_use]
pub fn test_s3_config() -> Option<TestS3Config> {
    let endpoint = std::env::var(TEST_S3_ENDPOINT_ENV).ok()?;
    let endpoint = endpoint.trim();
    if endpoint.is_empty() {
        return None;
    }
    Some(TestS3Config {
        endpoint: endpoint.to_owned(),
        access_key: env_first(&["S3_ACCESS_KEY_TEST", "S3_ACCESS_KEY"])
            .unwrap_or_else(|| "any".to_owned()),
        secret_key: env_first(&["S3_SECRET_KEY_TEST", "S3_SECRET_KEY"])
            .unwrap_or_else(|| "any".to_owned()),
        bucket: env_first(&["S3_BUCKET_TEST", "S3_BUCKET"])
            .unwrap_or_else(|| "voxnexus-test".to_owned()),
    })
}

fn env_first(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        std::env::var(name).ok().and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            }
        })
    })
}
