//! Permission resolution (F029).

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::code::PermissionCode;
use crate::family::{community, Family, GrantSet};

/// Allow / deny outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny,
}

impl Decision {
    #[must_use]
    pub fn is_allow(self) -> bool {
        matches!(self, Self::Allow)
    }
}

/// Actor + resource context for evaluation (pure, no I/O).
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct ActorContext {
    pub community_id: Uuid,
    pub account_id: Uuid,
    pub is_community_member: bool,
    pub is_community_owner: bool,
    pub is_instance_admin: bool,
    pub grants: GrantSet,
    pub space_id: Option<Uuid>,
    pub space_restricted: bool,
    pub is_space_member: bool,
    pub timeout_until: Option<DateTime<Utc>>,
}

/// Resolve a permission for `ctx` (steps 2–12 from MASTER_PLAN §5.3, without overrides).
#[must_use]
pub fn resolve(ctx: &ActorContext, permission: PermissionCode) -> Decision {
    if !ctx.is_community_member && !ctx.is_instance_admin {
        return Decision::Deny;
    }
    if ctx.is_community_owner {
        return Decision::Allow;
    }
    if ctx.space_restricted && !ctx.is_space_member {
        return Decision::Deny;
    }
    if ctx.has_administrator() && !permission.owner_only {
        return Decision::Allow;
    }
    if ctx.is_timed_out() && permission_stripped_by_timeout(permission) {
        return Decision::Deny;
    }
    if ctx.grants.has(permission.family, permission.bit) {
        return Decision::Allow;
    }
    Decision::Deny
}

impl ActorContext {
    #[must_use]
    pub fn has_administrator(&self) -> bool {
        self.grants.has(Family::Community, community::ADMINISTRATOR)
    }

    #[must_use]
    pub fn is_timed_out(&self) -> bool {
        self.timeout_until.is_some_and(|until| until > Utc::now())
    }
}

fn permission_stripped_by_timeout(permission: PermissionCode) -> bool {
    matches!(permission, PermissionCode::TEXT_SEND)
}
