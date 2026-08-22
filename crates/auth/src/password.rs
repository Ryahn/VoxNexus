//! Argon2id password hashing.

use std::sync::OnceLock;

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use password_hash::rand_core::OsRng;

/// Password hashing failures.
#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("password hashing failed")]
    Hash,
    #[error("password verification failed")]
    Verify,
}

fn argon2() -> Result<Argon2<'static>, PasswordError> {
    // Interactive-ish defaults: ~19 MiB, 2 iterations, 1 lane.
    let params = Params::new(19_456, 2, 1, None).map_err(|_| PasswordError::Hash)?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

fn dummy_password_hash() -> &'static str {
    static HASH: OnceLock<String> = OnceLock::new();
    HASH.get_or_init(|| hash_password("voxnexus-timing-dummy").expect("dummy hash"))
        .as_str()
}

/// Hash a password with Argon2id.
///
/// # Errors
///
/// Returns [`PasswordError::Hash`] if parameters or hashing fail.
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon = argon2()?;
    argon
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| PasswordError::Hash)
}

/// Verify `password` against an optional stored PHC hash.
///
/// When `stored` is `None`, verifies against a dummy hash so the cost stays similar.
///
/// # Errors
///
/// Returns [`PasswordError::Verify`] if the stored hash cannot be parsed.
pub fn verify_password(password: &str, stored: Option<&str>) -> Result<bool, PasswordError> {
    let hash_str = stored.unwrap_or_else(|| dummy_password_hash());
    let parsed = PasswordHash::new(hash_str).map_err(|_| PasswordError::Verify)?;
    let argon = argon2()?;
    match argon.verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(stored.is_some()),
        Err(password_hash::Error::Password) => Ok(false),
        Err(_) => Err(PasswordError::Verify),
    }
}

#[cfg(test)]
mod tests {
    use super::{hash_password, verify_password};

    #[test]
    fn round_trip_hash() {
        let hash = hash_password("correct horse battery").expect("hash");
        assert!(verify_password("correct horse battery", Some(&hash)).expect("verify"));
        assert!(!verify_password("wrong", Some(&hash)).expect("verify"));
    }

    #[test]
    fn missing_hash_is_false_but_runs() {
        assert!(!verify_password("anything", None).expect("verify"));
    }
}
