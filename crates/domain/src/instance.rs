//! Singleton instance policy and OIDC placeholder settings.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Who may register new local accounts on this instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationMode {
    Open,
    Invite,
    Closed,
}

impl RegistrationMode {
    /// Database / API string for this mode.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Invite => "invite",
            Self::Closed => "closed",
        }
    }

    /// Parse a persisted or configured mode string.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "open" => Some(Self::Open),
            "invite" => Some(Self::Invite),
            "closed" => Some(Self::Closed),
            _ => None,
        }
    }

    /// Whether `POST /api/v1/auth/register` is allowed.
    #[must_use]
    pub fn allows_registration(self) -> bool {
        matches!(self, Self::Open)
    }
}

/// Who may create communities on this instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CommunityCreationMode {
    Open,
    AdminOnly,
    Single,
}

impl CommunityCreationMode {
    /// Database / API string for this mode.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::AdminOnly => "admin_only",
            Self::Single => "single",
        }
    }

    /// Parse a persisted or configured mode string.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "open" => Some(Self::Open),
            "admin_only" => Some(Self::AdminOnly),
            "single" => Some(Self::Single),
            _ => None,
        }
    }

    /// Whether an authenticated user may call user-driven community creation APIs.
    ///
    /// `single` always denies user-driven creates; the bootstrap path seeds the one community.
    #[must_use]
    pub fn user_can_create_community(self, is_instance_admin: bool) -> bool {
        match self {
            Self::Open => true,
            Self::AdminOnly => is_instance_admin,
            Self::Single => false,
        }
    }

    /// Whether startup should seed the singleton community (`single` + empty).
    #[must_use]
    pub fn needs_bootstrap_community(self, existing_count: i64) -> bool {
        matches!(self, Self::Single) && existing_count == 0
    }
}

/// Persisted singleton instance row (F017).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instance {
    pub id: Uuid,
    pub name: String,
    pub public_url: String,
    pub registration_mode: RegistrationMode,
    pub community_creation_mode: CommunityCreationMode,
    pub oidc_enabled: bool,
    pub oidc_issuer: Option<String>,
    pub oidc_client_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
