//! SeaweedFS / S3-compatible client.

use std::time::Duration;

use async_trait::async_trait;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::{
    BehaviorVersion, Region, RequestChecksumCalculation, ResponseChecksumValidation,
};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use bytes::Bytes;
use url::Url;

use crate::{ObjectKey, ObjectStore, StorageError, StoredObject};

/// Connection settings for an S3-compatible endpoint.
#[derive(Debug, Clone)]
pub struct S3ObjectStoreConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
}

/// AWS SDK S3 client pointed at SeaweedFS (or LocalStack).
#[derive(Debug, Clone)]
pub struct S3ObjectStore {
    client: Client,
    bucket: String,
}

impl S3ObjectStore {
    /// Build a path-style S3 client for the given endpoint.
    #[must_use]
    pub fn new(config: S3ObjectStoreConfig) -> Self {
        let credentials = Credentials::new(
            config.access_key,
            config.secret_key,
            None,
            None,
            "voxnexus-config",
        );
        let s3_config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .endpoint_url(config.endpoint)
            .region(Region::new(config.region))
            .credentials_provider(credentials)
            .force_path_style(true)
            // SeaweedFS (and most S3-compatible stores) do not implement AWS's newer default checksums.
            .request_checksum_calculation(RequestChecksumCalculation::WhenRequired)
            .response_checksum_validation(ResponseChecksumValidation::WhenRequired)
            .build();
        Self {
            client: Client::from_conf(s3_config),
            bucket: config.bucket,
        }
    }

    fn map_sdk(error: impl std::fmt::Debug) -> StorageError {
        StorageError::S3(format!("{error:?}"))
    }

    fn looks_like_missing_bucket(message: &str) -> bool {
        let lower = message.to_ascii_lowercase();
        lower.contains("nosuchbucket")
            || lower.contains("not found")
            || lower.contains("404")
            || lower.contains("no such bucket")
    }

    fn looks_like_missing_object(message: &str) -> bool {
        let lower = message.to_ascii_lowercase();
        lower.contains("nosuchkey")
            || lower.contains("notfound")
            || lower.contains("no such key")
            || lower.contains("404")
    }
}

#[async_trait]
impl ObjectStore for S3ObjectStore {
    async fn put(
        &self,
        key: ObjectKey,
        bytes: Bytes,
        content_type: &str,
    ) -> Result<StoredObject, StorageError> {
        let byte_size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        let content_type_owned = content_type.to_owned();
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key.as_str())
            .content_type(content_type)
            .body(ByteStream::from(bytes))
            .send()
            .await
            .map_err(Self::map_sdk)?;
        Ok(StoredObject {
            key,
            content_type: content_type_owned,
            byte_size,
        })
    }

    async fn get(&self, key: &ObjectKey) -> Result<Bytes, StorageError> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key.as_str())
            .send()
            .await
            .map_err(|error| {
                let message = format!("{error:?}");
                if Self::looks_like_missing_object(&message) {
                    StorageError::NotFound(key.as_str().to_owned())
                } else {
                    Self::map_sdk(error)
                }
            })?;
        output
            .body
            .collect()
            .await
            .map(aws_sdk_s3::primitives::AggregatedBytes::into_bytes)
            .map_err(Self::map_sdk)
    }

    async fn delete(&self, key: &ObjectKey) -> Result<(), StorageError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key.as_str())
            .send()
            .await
            .map_err(Self::map_sdk)?;
        Ok(())
    }

    async fn presign_get(&self, key: &ObjectKey, ttl: Duration) -> Result<Url, StorageError> {
        let presigning = PresigningConfig::expires_in(ttl).map_err(Self::map_sdk)?;
        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key.as_str())
            .presigned(presigning)
            .await
            .map_err(Self::map_sdk)?;
        Url::parse(presigned.uri()).map_err(|error| StorageError::S3(error.to_string()))
    }

    async fn head_bucket(&self) -> Result<(), StorageError> {
        self.client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .map_err(Self::map_sdk)?;
        Ok(())
    }

    async fn ensure_bucket(&self) -> Result<(), StorageError> {
        match self.head_bucket().await {
            Ok(()) => Ok(()),
            Err(StorageError::S3(message)) if Self::looks_like_missing_bucket(&message) => {
                self.client
                    .create_bucket()
                    .bucket(&self.bucket)
                    .send()
                    .await
                    .map_err(Self::map_sdk)?;
                self.head_bucket().await
            }
            Err(error) => Err(error),
        }
    }
}
