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

/// Change password (requires current password).
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, max = 128))]
    pub current_password: String,
    #[validate(length(min = 8, max = 128))]
    pub new_password: String,
    /// When true, delete all other sessions for this account (current session stays).
    pub revoke_other_sessions: Option<bool>,
}

/// Change email (requires current password; applied immediately until F117 adds confirmation mail).
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct ChangeEmailRequest {
    #[validate(email, length(max = 320))]
    pub email: String,
    #[validate(length(min = 1, max = 128))]
    pub current_password: String,
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
