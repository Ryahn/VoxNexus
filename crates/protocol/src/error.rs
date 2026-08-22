use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Stable `code` strings in [`ErrorBody`].
pub mod error_codes {
    pub const NOT_FOUND: &str = "not_found";
    pub const INVALID_JSON: &str = "invalid_json";
    pub const VALIDATION_ERROR: &str = "validation_error";
    pub const INTERNAL: &str = "internal";
    pub const UNAUTHENTICATED: &str = "unauthenticated";
    pub const PERMISSION_DENIED: &str = "permission_denied";
    pub const CONFLICT: &str = "conflict";
    pub const RATE_LIMITED: &str = "rate_limited";
    pub const GATEWAY_UNAVAILABLE: &str = "gateway_unavailable";
}

/// JSON error envelope for `/api/v1` and unknown routes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object, nullable)]
    pub details: Option<serde_json::Value>,
    pub request_id: String,
}
