//! Permission families and bit masks (F029).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Permission family — each packs grants into a `u64` bitset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    Community,
    Space,
    Text,
    Voice,
}

impl Family {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Community => "community",
            Self::Space => "space",
            Self::Text => "text",
            Self::Voice => "voice",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "community" => Some(Self::Community),
            "space" => Some(Self::Space),
            "text" => Some(Self::Text),
            "voice" => Some(Self::Voice),
            _ => None,
        }
    }
}

/// Community-family bits (see MASTER_PLAN §5.7).
pub mod community {
    pub const ADMINISTRATOR: u64 = 1 << 0;
    pub const MANAGE_ROLES: u64 = 1 << 2;
    pub const MANAGE_CHANNELS: u64 = 1 << 3;
    pub const VIEW_AUDIT: u64 = 1 << 4;
}

/// Text-channel bits.
pub mod text {
    pub const VIEW: u64 = 1 << 0;
    pub const SEND: u64 = 1 << 1;
}

/// Merged grants across families for one actor.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GrantSet {
    bits: HashMap<Family, u64>,
}

impl GrantSet {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bits: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_family(mut self, family: Family, mask: u64) -> Self {
        self.bits.insert(family, mask);
        self
    }

    /// OR-merge another family's bits into this set.
    pub fn merge_family(&mut self, family: Family, mask: u64) {
        let entry = self.bits.entry(family).or_insert(0);
        *entry |= mask;
    }

    #[must_use]
    pub fn get(&self, family: Family) -> u64 {
        self.bits.get(&family).copied().unwrap_or(0)
    }

    #[must_use]
    pub fn has(&self, family: Family, bit: u64) -> bool {
        self.get(family) & bit != 0
    }

    /// Replace a family's mask (overwrites, does not OR).
    pub fn set_family(&mut self, family: Family, mask: u64) {
        if mask == 0 {
            self.bits.remove(&family);
        } else {
            self.bits.insert(family, mask);
        }
    }
}
