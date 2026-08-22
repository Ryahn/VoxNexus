//! Auth HTTP DTOs.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Register with email and password.
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email, length(max = 320))]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

/// Login with email and password.
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email, length(max = 320))]
    pub email: String,
    #[validate(length(min = 1, max = 128))]
    pub password: String,
}

/// Public account fields (never includes password hash).
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AccountResponse {
    pub id: Uuid,
    pub email: Option<String>,
    pub is_bot: bool,
    pub is_instance_admin: bool,
}

/// Authenticated session payload.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AuthSessionResponse {
    pub account: AccountResponse,
}
