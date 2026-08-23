//! User presence status (F018).

use serde::{Deserialize, Serialize};

/// Stored preference and gateway-reported status while connected.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PresenceStatus {
    Online,
    Idle,
    Dnd,
    Invisible,
}

impl PresenceStatus {
    /// Database / API string for this status.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Idle => "idle",
            Self::Dnd => "dnd",
            Self::Invisible => "invisible",
        }
    }

    /// Parse a persisted or configured status string.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "online" => Some(Self::Online),
            "idle" => Some(Self::Idle),
            "dnd" => Some(Self::Dnd),
            "invisible" => Some(Self::Invisible),
            _ => None,
        }
    }
}

/// Presence exposed to clients (offline when disconnected or hidden from viewers).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PublicPresenceStatus {
    Online,
    Idle,
    Dnd,
    Invisible,
    Offline,
}

impl PublicPresenceStatus {
    /// Map a stored status for a connected account to the public view.
    #[must_use]
    pub fn from_connected(stored: PresenceStatus, self_view: bool) -> Self {
        match stored {
            PresenceStatus::Online => Self::Online,
            PresenceStatus::Idle => Self::Idle,
            PresenceStatus::Dnd => Self::Dnd,
            PresenceStatus::Invisible if self_view => Self::Invisible,
            PresenceStatus::Invisible => Self::Offline,
        }
    }

    /// Offline when the account has no live gateway connection.
    #[must_use]
    pub fn offline() -> Self {
        Self::Offline
    }
}
