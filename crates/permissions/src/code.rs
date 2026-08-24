//! Stable permission codes (`text.view`, `community.manage_channels`, …).

use crate::family::{community, text, Family};

/// Parsed permission code mapped to a family bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionCode {
    pub family: Family,
    pub bit: u64,
    pub owner_only: bool,
}

impl PermissionCode {
    pub const COMMUNITY_ADMINISTRATOR: Self = Self {
        family: Family::Community,
        bit: community::ADMINISTRATOR,
        owner_only: false,
    };
    pub const COMMUNITY_MANAGE_CHANNELS: Self = Self {
        family: Family::Community,
        bit: community::MANAGE_CHANNELS,
        owner_only: false,
    };
    pub const COMMUNITY_MANAGE_ROLES: Self = Self {
        family: Family::Community,
        bit: community::MANAGE_ROLES,
        owner_only: false,
    };
    pub const COMMUNITY_VIEW_AUDIT: Self = Self {
        family: Family::Community,
        bit: community::VIEW_AUDIT,
        owner_only: false,
    };
    pub const TEXT_VIEW: Self = Self {
        family: Family::Text,
        bit: text::VIEW,
        owner_only: false,
    };
    pub const TEXT_SEND: Self = Self {
        family: Family::Text,
        bit: text::SEND,
        owner_only: false,
    };
    pub const TEXT_MANAGE_MESSAGES: Self = Self {
        family: Family::Text,
        bit: text::MANAGE_MESSAGES,
        owner_only: false,
    };

    /// Parse a stable API permission string.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "community.administrator" => Some(Self::COMMUNITY_ADMINISTRATOR),
            "community.manage_channels" => Some(Self::COMMUNITY_MANAGE_CHANNELS),
            "community.manage_roles" => Some(Self::COMMUNITY_MANAGE_ROLES),
            "community.view_audit" => Some(Self::COMMUNITY_VIEW_AUDIT),
            "text.view" | "channel.view" => Some(Self::TEXT_VIEW),
            "text.send" | "message.send" => Some(Self::TEXT_SEND),
            "text.manage_messages" | "message.manage" | "manage_messages" => {
                Some(Self::TEXT_MANAGE_MESSAGES)
            }
            _ => None,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::COMMUNITY_ADMINISTRATOR => "community.administrator",
            Self::COMMUNITY_MANAGE_CHANNELS => "community.manage_channels",
            Self::COMMUNITY_MANAGE_ROLES => "community.manage_roles",
            Self::COMMUNITY_VIEW_AUDIT => "community.view_audit",
            Self::TEXT_VIEW => "text.view",
            Self::TEXT_SEND => "text.send",
            Self::TEXT_MANAGE_MESSAGES => "text.manage_messages",
            Self { .. } => "unknown",
        }
    }
}
