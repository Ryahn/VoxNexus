//! VoxNexus composition root: HTTP API, tracing, and process lifecycle.

pub mod csrf;
pub mod error;
pub mod extract;
pub mod gateway;
pub mod http;
pub mod openapi;
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
