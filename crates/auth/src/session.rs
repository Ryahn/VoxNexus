//! Session tokens and cookie helpers.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Duration;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Sliding session lifetime (30 days) until instance settings (F017).
pub const SESSION_TTL: Duration = Duration::days(30);

const DEV_COOKIE_NAME: &str = "vn_session";
const SECURE_COOKIE_NAME: &str = "__Host-vn_session";

/// Cookie flag bundle from process config.
#[derive(Debug, Clone, Copy)]
pub struct SessionCookieOptions {
    pub secure: bool,
}

/// Cookie name: `vn_session` in cleartext HTTP, `__Host-vn_session` when Secure.
#[must_use]
pub fn session_cookie_name(secure: bool) -> &'static str {
    if secure {
        SECURE_COOKIE_NAME
    } else {
        DEV_COOKIE_NAME
    }
}

/// 32 random bytes, URL-safe base64 (no pad).
#[must_use]
pub fn new_session_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// SHA-256 of the raw token (stored at rest).
#[must_use]
pub fn hash_session_token(raw_token: &str) -> [u8; 32] {
    let digest = Sha256::digest(raw_token.as_bytes());
    let mut out = [0_u8; 32];
    out.copy_from_slice(&digest);
    out
}

/// `Set-Cookie` value for a new session.
#[must_use]
pub fn session_cookie(raw_token: &str, options: SessionCookieOptions) -> String {
    let name = session_cookie_name(options.secure);
    let max_age = SESSION_TTL.num_seconds().max(0);
    let mut parts = vec![
        format!("{name}={raw_token}"),
        "Path=/".to_owned(),
        "HttpOnly".to_owned(),
        "SameSite=Lax".to_owned(),
        format!("Max-Age={max_age}"),
    ];
    if options.secure {
        parts.push("Secure".to_owned());
    }
    parts.join("; ")
}

/// `Set-Cookie` that clears the session cookie.
#[must_use]
pub fn clear_session_cookie(options: SessionCookieOptions) -> String {
    let name = session_cookie_name(options.secure);
    let mut parts = vec![
        format!("{name}="),
        "Path=/".to_owned(),
        "HttpOnly".to_owned(),
        "SameSite=Lax".to_owned(),
        "Max-Age=0".to_owned(),
    ];
    if options.secure {
        parts.push("Secure".to_owned());
    }
    parts.join("; ")
}
