//! Shared domain types with no I/O.

mod community;
mod instance;
mod presence;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use community::{Community, CommunityInvite, CommunityMember, CommunityMemberRole, JoinMode};
pub use instance::{CommunityCreationMode, Instance, RegistrationMode};
pub use presence::{PresenceStatus, PublicPresenceStatus};

/// Product crate name (workspace placeholder until modules grow).
pub const CRATE_NAME: &str = "voxnexus-domain";

/// Singleton instance id (matches the seeded `instances` row).
pub const DEFAULT_INSTANCE_ID: Uuid = Uuid::from_u128(0x0190_0000_0000_7000_8000_0000_0000_0001);

/// Human or bot account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub instance_id: Uuid,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub is_bot: bool,
    pub is_instance_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Account {
    /// Local password accounts must store a hash; bots/OIDC-only may omit it.
    #[must_use]
    pub fn is_local_password_account(&self) -> bool {
        !self.is_bot && self.password_hash.is_some()
    }
}

/// OIDC (or other issuer) subject linked to an account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthIdentity {
    pub id: Uuid,
    pub account_id: Uuid,
    pub issuer: String,
    pub subject: String,
    pub created_at: DateTime<Utc>,
}

/// Server-side session row. Cookie holds the unhashed secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub account_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

/// Per-account profile (display fields + optional image object ids).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub account_id: Uuid,
    pub display_name: String,
    pub bio: String,
    pub presence_status: PresenceStatus,
    pub custom_status: String,
    pub avatar_object_id: Option<Uuid>,
    pub banner_object_id: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

/// Metadata for a byte object in SeaweedFS / S3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectMeta {
    pub id: Uuid,
    pub storage_key: String,
    pub mime: String,
    pub byte_size: i64,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}
