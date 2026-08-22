//! Validated object keys (no path traversal).

use crate::StorageError;

/// Opaque object key after validation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectKey(String);

impl ObjectKey {
    /// Parse and validate a storage key.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::InvalidKey`] when the key is empty, absolute, contains `..`,
    /// backslashes, null bytes, or empty path segments.
    pub fn parse(raw: impl AsRef<str>) -> Result<Self, StorageError> {
        let raw = raw.as_ref();
        if raw.is_empty() {
            return Err(StorageError::InvalidKey("key must not be empty".into()));
        }
        if raw.contains('\0') {
            return Err(StorageError::InvalidKey("key must not contain NUL".into()));
        }
        if raw.starts_with('/') || raw.starts_with('\\') {
            return Err(StorageError::InvalidKey("key must not be absolute".into()));
        }
        if raw.contains('\\') {
            return Err(StorageError::InvalidKey(
                "key must not contain backslashes".into(),
            ));
        }
        for segment in raw.split('/') {
            if segment.is_empty() {
                return Err(StorageError::InvalidKey(
                    "key must not contain empty segments".into(),
                ));
            }
            if segment == "." || segment == ".." {
                return Err(StorageError::InvalidKey(
                    "key must not contain '.' or '..' segments".into(),
                ));
            }
        }
        Ok(Self(raw.to_owned()))
    }

    /// Borrow the validated key string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ObjectKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for ObjectKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_normal_keys() {
        assert!(ObjectKey::parse("vn/2026/018f-uuid").is_ok());
        assert!(ObjectKey::parse("a").is_ok());
    }

    #[test]
    fn rejects_traversal_and_absolute() {
        assert!(ObjectKey::parse("../secret").is_err());
        assert!(ObjectKey::parse("a/../b").is_err());
        assert!(ObjectKey::parse("/abs").is_err());
        assert!(ObjectKey::parse("a\\b").is_err());
        assert!(ObjectKey::parse("a//b").is_err());
        assert!(ObjectKey::parse("").is_err());
        assert!(ObjectKey::parse(".").is_err());
    }
}
