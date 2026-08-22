//! Validated process configuration loaded from an optional file and the environment.

mod error;
mod log_format;
mod log_level;
mod secret;

use std::env;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

use figment::providers::{Format, Serialized, Toml, Yaml};
use figment::Figment;
use serde::Deserialize;

pub use error::ConfigError;
pub use log_format::LogFormat;
pub use log_level::LogLevel;
pub use secret::Secret;
pub use url::Url;

/// Default bind address when `LISTEN_ADDR` is unset.
pub const DEFAULT_LISTEN_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

/// Environment variable that selects an explicit config file path.
pub const CONFIG_FILE_ENV: &str = "VOXNEXUS_CONFIG";

/// Required configuration keys. Environment variables override file values.
pub const REQUIRED_KEYS: &[&str] = &[
    "DATABASE_URL",
    "REDIS_URL",
    "S3_ENDPOINT",
    "S3_ACCESS_KEY",
    "S3_SECRET_KEY",
    "S3_BUCKET",
    "TYPESENSE_URL",
    "TYPESENSE_API_KEY",
    "PUBLIC_URL",
];

const ENV_KEYS: &[&str] = &[
    "DATABASE_URL",
    "REDIS_URL",
    "S3_ENDPOINT",
    "S3_ACCESS_KEY",
    "S3_SECRET_KEY",
    "S3_BUCKET",
    "TYPESENSE_URL",
    "TYPESENSE_API_KEY",
    "PUBLIC_URL",
    "COOKIE_SECURE",
    "LOG_LEVEL",
    "LOG_FORMAT",
    "LISTEN_ADDR",
    "METRICS_ENABLED",
    "GATEWAY_ALLOW_UNAUTH",
    "LIVEKIT_URL",
    "LIVEKIT_API_KEY",
    "LIVEKIT_API_SECRET",
    "OIDC_ISSUER",
    "OIDC_CLIENT_ID",
    "OIDC_CLIENT_SECRET",
    "SMTP_URL",
    "SMTP_FROM",
];

/// Runtime configuration. Secrets are redacted in [`Debug`].
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: Url,
    pub redis_url: Url,
    pub s3_endpoint: Url,
    pub s3_access_key: Secret,
    pub s3_secret_key: Secret,
    pub s3_bucket: String,
    pub typesense_url: Url,
    pub typesense_api_key: Secret,
    pub public_url: Url,
    pub cookie_secure: bool,
    pub log_level: LogLevel,
    pub log_format: LogFormat,
    pub listen_addr: SocketAddr,
    pub metrics_enabled: bool,
    /// When true, `/api/v1/gateway` accepts unauthenticated sessions (dev-only ping). Default false until F013.
    pub gateway_allow_unauth: bool,
    pub livekit_url: Option<Url>,
    pub livekit_api_key: Option<Secret>,
    pub livekit_api_secret: Option<Secret>,
    pub oidc_issuer: Option<Url>,
    pub oidc_client_id: Option<String>,
    pub oidc_client_secret: Option<Secret>,
    pub smtp_url: Option<Url>,
    pub smtp_from: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FlexString {
    Boolean(bool),
    Number(i64),
    Text(String),
}

impl FlexString {
    fn into_string(self) -> String {
        match self {
            Self::Boolean(true) => "true".to_string(),
            Self::Boolean(false) => "false".to_string(),
            Self::Number(value) => value.to_string(),
            Self::Text(value) => value,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawConfig {
    #[serde(rename = "DATABASE_URL")]
    database_url: Option<String>,
    #[serde(rename = "REDIS_URL")]
    redis_url: Option<String>,
    #[serde(rename = "S3_ENDPOINT")]
    s3_endpoint: Option<String>,
    #[serde(rename = "S3_ACCESS_KEY")]
    s3_access_key: Option<String>,
    #[serde(rename = "S3_SECRET_KEY")]
    s3_secret_key: Option<String>,
    #[serde(rename = "S3_BUCKET")]
    s3_bucket: Option<String>,
    #[serde(rename = "TYPESENSE_URL")]
    typesense_url: Option<String>,
    #[serde(rename = "TYPESENSE_API_KEY")]
    typesense_api_key: Option<String>,
    #[serde(rename = "PUBLIC_URL")]
    public_url: Option<String>,
    #[serde(rename = "COOKIE_SECURE")]
    cookie_secure: Option<FlexString>,
    #[serde(rename = "LOG_LEVEL")]
    log_level: Option<String>,
    #[serde(rename = "LOG_FORMAT")]
    log_format: Option<String>,
    #[serde(rename = "LISTEN_ADDR")]
    listen_addr: Option<String>,
    #[serde(rename = "METRICS_ENABLED")]
    metrics_enabled: Option<FlexString>,
    #[serde(rename = "GATEWAY_ALLOW_UNAUTH")]
    gateway_allow_unauth: Option<FlexString>,
    #[serde(rename = "LIVEKIT_URL")]
    livekit_url: Option<String>,
    #[serde(rename = "LIVEKIT_API_KEY")]
    livekit_api_key: Option<String>,
    #[serde(rename = "LIVEKIT_API_SECRET")]
    livekit_api_secret: Option<String>,
    #[serde(rename = "OIDC_ISSUER")]
    oidc_issuer: Option<String>,
    #[serde(rename = "OIDC_CLIENT_ID")]
    oidc_client_id: Option<String>,
    #[serde(rename = "OIDC_CLIENT_SECRET")]
    oidc_client_secret: Option<String>,
    #[serde(rename = "SMTP_URL")]
    smtp_url: Option<String>,
    #[serde(rename = "SMTP_FROM")]
    smtp_from: Option<String>,
}

impl Config {
    /// Load configuration from an optional file, then overlay process environment.
    ///
    /// Search order for the file: `VOXNEXUS_CONFIG`, `./config.toml`, `./config.yaml`,
    /// `./config.yml`. Environment variables always win over file values.
    ///
    /// # Errors
    ///
    /// Returns an error when a required key is missing, a URL or boolean cannot be parsed,
    /// `VOXNEXUS_CONFIG` points at a missing file, the file cannot be read, or (on Unix)
    /// the file is more permissive than mode 0600.
    pub fn load() -> Result<Self, ConfigError> {
        let file = discover_config_file()?;
        let env: Vec<(String, String)> = env::vars().collect();
        Self::from_sources(file.as_deref(), &env)
    }

    /// Load from an explicit file (optional) and an explicit environment map.
    ///
    /// Used by tests and by [`Self::load`]. `env` entries overlay the file.
    ///
    /// # Errors
    ///
    /// Returns an error when a required key is missing, a value is invalid, or the file
    /// cannot be used (missing, unreadable, unsupported extension, or insecure permissions).
    pub fn from_sources(
        file: Option<&Path>,
        env: &[(String, String)],
    ) -> Result<Self, ConfigError> {
        let mut figment = Figment::new();
        if let Some(path) = file {
            assert_secret_file_mode(path)?;
            figment = merge_config_file(figment, path)?;
        }
        figment = merge_env_overlay(figment, env);

        let raw: RawConfig = figment
            .extract()
            .map_err(|error| ConfigError::from_figment(&error))?;
        Self::from_raw(raw)
    }

    fn from_raw(raw: RawConfig) -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: require_url("DATABASE_URL", raw.database_url)?,
            redis_url: require_url("REDIS_URL", raw.redis_url)?,
            s3_endpoint: require_url("S3_ENDPOINT", raw.s3_endpoint)?,
            s3_access_key: Secret::new(require_nonempty("S3_ACCESS_KEY", raw.s3_access_key)?),
            s3_secret_key: Secret::new(require_nonempty("S3_SECRET_KEY", raw.s3_secret_key)?),
            s3_bucket: require_nonempty("S3_BUCKET", raw.s3_bucket)?,
            typesense_url: require_url("TYPESENSE_URL", raw.typesense_url)?,
            typesense_api_key: Secret::new(require_nonempty(
                "TYPESENSE_API_KEY",
                raw.typesense_api_key,
            )?),
            public_url: require_url("PUBLIC_URL", raw.public_url)?,
            cookie_secure: parse_bool(
                "COOKIE_SECURE",
                raw.cookie_secure.map(FlexString::into_string),
                false,
            )?,
            log_level: parse_log_level(raw.log_level)?,
            log_format: parse_log_format(raw.log_format)?,
            listen_addr: parse_listen_addr(raw.listen_addr)?,
            metrics_enabled: parse_bool(
                "METRICS_ENABLED",
                raw.metrics_enabled.map(FlexString::into_string),
                false,
            )?,
            gateway_allow_unauth: parse_bool(
                "GATEWAY_ALLOW_UNAUTH",
                raw.gateway_allow_unauth.map(FlexString::into_string),
                false,
            )?,
            livekit_url: optional_url("LIVEKIT_URL", raw.livekit_url)?,
            livekit_api_key: optional_secret(raw.livekit_api_key),
            livekit_api_secret: optional_secret(raw.livekit_api_secret),
            oidc_issuer: optional_url("OIDC_ISSUER", raw.oidc_issuer)?,
            oidc_client_id: optional_nonempty(raw.oidc_client_id),
            oidc_client_secret: optional_secret(raw.oidc_client_secret),
            smtp_url: optional_url("SMTP_URL", raw.smtp_url)?,
            smtp_from: optional_nonempty(raw.smtp_from),
        })
    }
}

fn discover_config_file() -> Result<Option<PathBuf>, ConfigError> {
    if let Some(path) = env::var_os(CONFIG_FILE_ENV) {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(Some(path));
        }
        return Err(ConfigError::FileNotFound { path });
    }

    for name in ["config.toml", "config.yaml", "config.yml"] {
        let path = PathBuf::from(name);
        if path.is_file() {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

fn merge_config_file(figment: Figment, path: &Path) -> Result<Figment, ConfigError> {
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "toml" => Ok(figment.merge(Toml::file(path))),
        "yaml" | "yml" => Ok(figment.merge(Yaml::file(path))),
        _ => Err(ConfigError::UnsupportedFile {
            path: path.to_path_buf(),
        }),
    }
}

fn merge_env_overlay(mut figment: Figment, env: &[(String, String)]) -> Figment {
    for (key, value) in env {
        if ENV_KEYS.contains(&key.as_str()) {
            figment = figment.merge(Serialized::default(key.as_str(), value));
        }
    }
    figment
}

fn require_nonempty(key: &'static str, value: Option<String>) -> Result<String, ConfigError> {
    match value {
        Some(value) if !value.trim().is_empty() => Ok(value.trim().to_string()),
        _ => Err(ConfigError::MissingKey { key }),
    }
}

fn require_url(key: &'static str, value: Option<String>) -> Result<Url, ConfigError> {
    let value = require_nonempty(key, value)?;
    Url::parse(&value).map_err(|source| ConfigError::InvalidValue {
        key,
        message: source.to_string(),
    })
}

fn optional_nonempty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn optional_secret(value: Option<String>) -> Option<Secret> {
    optional_nonempty(value).map(Secret::new)
}

fn optional_url(key: &'static str, value: Option<String>) -> Result<Option<Url>, ConfigError> {
    match optional_nonempty(value) {
        Some(value) => Url::parse(&value)
            .map(Some)
            .map_err(|source| ConfigError::InvalidValue {
                key,
                message: source.to_string(),
            }),
        None => Ok(None),
    }
}

fn parse_bool(
    key: &'static str,
    value: Option<String>,
    default: bool,
) -> Result<bool, ConfigError> {
    let Some(value) = optional_nonempty(value) else {
        return Ok(default);
    };
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(ConfigError::InvalidValue {
            key,
            message: format!("expected a boolean, got {value}"),
        }),
    }
}

fn parse_log_level(value: Option<String>) -> Result<LogLevel, ConfigError> {
    match optional_nonempty(value) {
        None => Ok(LogLevel::Info),
        Some(value) => value.parse().map_err(|message| ConfigError::InvalidValue {
            key: "LOG_LEVEL",
            message,
        }),
    }
}

fn parse_log_format(value: Option<String>) -> Result<LogFormat, ConfigError> {
    match optional_nonempty(value) {
        None => Ok(LogFormat::Auto),
        Some(value) => value.parse().map_err(|message| ConfigError::InvalidValue {
            key: "LOG_FORMAT",
            message,
        }),
    }
}

fn parse_listen_addr(value: Option<String>) -> Result<SocketAddr, ConfigError> {
    match optional_nonempty(value) {
        None => Ok(DEFAULT_LISTEN_ADDR),
        Some(value) => value
            .parse::<SocketAddr>()
            .map_err(|source| ConfigError::InvalidValue {
                key: "LISTEN_ADDR",
                message: source.to_string(),
            }),
    }
}

fn assert_secret_file_mode(path: &Path) -> Result<(), ConfigError> {
    let metadata = fs::metadata(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode() & 0o777;
        if mode & 0o077 != 0 {
            return Err(ConfigError::InsecureFile {
                path: path.to_path_buf(),
                mode,
            });
        }
    }

    #[cfg(not(unix))]
    {
        let _ = metadata;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    fn valid_pairs() -> Vec<(String, String)> {
        REQUIRED_KEYS
            .iter()
            .map(|key| {
                let value = match *key {
                    "DATABASE_URL" => "postgres://voxnexus:voxnexus@127.0.0.1:5432/voxnexus",
                    "REDIS_URL" => "redis://127.0.0.1:6379",
                    "S3_ENDPOINT" => "http://127.0.0.1:8333",
                    "S3_ACCESS_KEY" => "any",
                    "S3_SECRET_KEY" => "super-secret",
                    "S3_BUCKET" => "voxnexus",
                    "TYPESENSE_URL" => "http://127.0.0.1:8108",
                    "TYPESENSE_API_KEY" => "ts",
                    "PUBLIC_URL" => "http://127.0.0.1:8080",
                    _ => unreachable!(),
                };
                ((*key).to_string(), value.to_string())
            })
            .collect()
    }

    fn without_key(key: &str) -> Vec<(String, String)> {
        valid_pairs()
            .into_iter()
            .filter(|(candidate, _)| candidate != key)
            .collect()
    }

    #[test]
    fn missing_database_url_names_the_key() {
        let error = Config::from_sources(None, &without_key("DATABASE_URL")).unwrap_err();
        assert!(matches!(
            error,
            ConfigError::MissingKey {
                key: "DATABASE_URL"
            }
        ));
        assert!(error.to_string().contains("DATABASE_URL"));
    }

    #[test]
    fn invalid_url_fails_parse() {
        let mut env = without_key("DATABASE_URL");
        env.push(("DATABASE_URL".to_string(), "not-a-url".to_string()));
        let error = Config::from_sources(None, &env).unwrap_err();
        match error {
            ConfigError::InvalidValue { key, message } => {
                assert_eq!(key, "DATABASE_URL");
                assert!(!message.is_empty());
            }
            other => panic!("expected invalid value, got {other}"),
        }
    }

    #[test]
    fn env_overrides_file() {
        let mut file = NamedTempFile::with_suffix(".toml").expect("temp file");
        writeln!(
            file,
            r#"
DATABASE_URL = "postgres://file.example:5432/voxnexus"
REDIS_URL = "redis://127.0.0.1:6379"
S3_ENDPOINT = "http://127.0.0.1:8333"
S3_ACCESS_KEY = "file-key"
S3_SECRET_KEY = "file-secret"
S3_BUCKET = "file-bucket"
TYPESENSE_URL = "http://127.0.0.1:8108"
TYPESENSE_API_KEY = "file-ts"
PUBLIC_URL = "http://file.example:8080"
"#
        )
        .expect("write toml");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(file.path()).unwrap().permissions();
            perms.set_mode(0o600);
            fs::set_permissions(file.path(), perms).unwrap();
        }

        let env = vec![
            (
                "DATABASE_URL".to_string(),
                "postgres://env.example:5432/voxnexus".to_string(),
            ),
            ("S3_ACCESS_KEY".to_string(), "env-key".to_string()),
        ];

        let config = Config::from_sources(Some(file.path()), &env).expect("load");
        assert_eq!(config.database_url.host_str(), Some("env.example"));
        assert_eq!(config.s3_access_key.expose(), "env-key");
        assert_eq!(config.s3_bucket, "file-bucket");
    }

    #[test]
    fn livekit_oidc_smtp_are_optional() {
        let config = Config::from_sources(None, &valid_pairs()).expect("load");
        assert!(config.livekit_url.is_none());
        assert!(config.oidc_issuer.is_none());
        assert!(config.smtp_url.is_none());
        assert!(!config.cookie_secure);
        assert_eq!(config.log_level, LogLevel::Info);
        assert_eq!(config.log_format, LogFormat::Auto);
        assert_eq!(config.listen_addr, DEFAULT_LISTEN_ADDR);
        assert!(!config.metrics_enabled);
    }

    #[test]
    fn invalid_listen_addr_fails_parse() {
        let mut env = valid_pairs();
        env.push(("LISTEN_ADDR".to_string(), "not-an-address".to_string()));
        let error = Config::from_sources(None, &env).unwrap_err();
        match error {
            ConfigError::InvalidValue { key, .. } => assert_eq!(key, "LISTEN_ADDR"),
            other => panic!("expected invalid LISTEN_ADDR, got {other}"),
        }
    }

    #[test]
    fn secrets_are_redacted_in_debug() {
        let config = Config::from_sources(None, &valid_pairs()).expect("load");
        assert_eq!(format!("{:?}", config.s3_secret_key), "***");
        assert!(!format!("{config:?}").contains("super-secret"));
    }

    #[cfg(unix)]
    #[test]
    fn world_readable_config_file_is_rejected() {
        use std::os::unix::fs::PermissionsExt;

        let mut file = NamedTempFile::with_suffix(".toml").expect("temp file");
        writeln!(file, "DATABASE_URL = \"postgres://127.0.0.1/db\"").unwrap();
        let mut perms = fs::metadata(file.path()).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(file.path(), perms).unwrap();

        let error = Config::from_sources(Some(file.path()), &[]).unwrap_err();
        assert!(matches!(error, ConfigError::InsecureFile { .. }));
        assert!(error.to_string().contains("0600"));
    }
}
