use std::io;
use std::path::PathBuf;

use thiserror::Error;

/// Configuration failed to load or validate.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required configuration key {key}")]
    MissingKey { key: &'static str },
    #[error("invalid configuration value for {key}: {message}")]
    InvalidValue { key: &'static str, message: String },
    #[error("configuration file {} not found (from VOXNEXUS_CONFIG)", .path.display())]
    FileNotFound { path: PathBuf },
    #[error("configuration file {} must be .toml, .yaml, or .yml", .path.display())]
    UnsupportedFile { path: PathBuf },
    #[error(
        "configuration file {} must have mode 0600 because it may contain secrets (found {mode:o})",
        .path.display()
    )]
    InsecureFile { path: PathBuf, mode: u32 },
    #[error("failed to read configuration file {}: {source}", .path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{0}")]
    Figment(String),
}

impl ConfigError {
    pub(crate) fn from_figment(error: &figment::Error) -> Self {
        if error.missing() {
            if let figment::error::Kind::MissingField(field) = &error.kind {
                return Self::Figment(format!("missing required configuration key {field}"));
            }
        }
        Self::Figment(error.to_string())
    }
}
