//! VoxNexus composition root: HTTP API, tracing, and process lifecycle.

pub mod auth;
pub mod auth_middleware;
pub mod categories;
pub mod channels;
pub mod communities;
pub mod csrf;
pub mod error;
pub mod extract;
pub mod extract_auth;
pub mod explain;
pub mod gateway;
pub mod http;
pub mod instance;
pub mod invites;
pub mod oidc;
pub mod openapi;
pub mod permission_overrides;
pub mod permissions;
pub mod presence;
pub mod profile;
pub mod roles;
pub mod spaces;
pub mod telemetry;

/// Product name.
#[must_use]
pub fn hello() -> &'static str {
    "VoxNexus"
}

#[cfg(test)]
mod tests {
    use super::hello;

    #[test]
    fn hello_returns_product_name() {
        assert_eq!(hello(), "VoxNexus");
    }
}
