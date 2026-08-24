//! Permission evaluation, caching, and stable codes (F029).

mod explain;
mod override_layer;
mod cache;
mod code;
mod eval;
mod family;
mod grants;

pub use explain::{resolve_traced, ExplainStep};
pub use override_layer::{apply_override_layers, OverrideBundle};
pub use cache::PermissionCache;
pub use code::PermissionCode;
pub use eval::{ActorContext, Decision, resolve};
pub use family::{community, text, Family, GrantSet};
pub use grants::{
    collapse_roles_by_weight, default_everyone_grants, default_everyone_permissions_json,
    empty_role_permissions_json, parse_role_permissions, parse_role_permissions_allow_only,
    permissions_with_manage_roles, RolePermissionSet,
};

/// Product crate name.
pub const CRATE_NAME: &str = "voxnexus-permissions";

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::*;

    fn base_ctx() -> ActorContext {
        ActorContext {
            community_id: Uuid::now_v7(),
            account_id: Uuid::now_v7(),
            is_community_member: true,
            is_community_owner: false,
            is_instance_admin: false,
            grants: GrantSet::new(),
            space_id: None,
            space_restricted: false,
            is_space_member: false,
            timeout_until: None,
        }
    }

    #[test]
    fn multi_role_or_grants_view() {
        let mut ctx = base_ctx();
        ctx.grants.merge_family(Family::Text, 0);
        ctx.grants.merge_family(Family::Text, text::VIEW);
        assert!(resolve(&ctx, PermissionCode::TEXT_VIEW).is_allow());
    }

    #[test]
    fn weight_deny_beats_lower_priority_allow() {
        let high = RolePermissionSet::new().with_deny(Family::Text, text::VIEW);
        let low = RolePermissionSet::new().with_allow(Family::Text, text::VIEW);
        let grants = collapse_roles_by_weight(&[(20, high), (100, low)]);
        assert!(!grants.has(Family::Text, text::VIEW));
    }

    #[test]
    fn weight_inherit_skips_to_base_allow() {
        let specialty = RolePermissionSet::new();
        let member = RolePermissionSet::new().with_allow(Family::Text, text::VIEW);
        let grants = collapse_roles_by_weight(&[(50, specialty), (100, member)]);
        assert!(grants.has(Family::Text, text::VIEW));
    }

    #[test]
    fn administrator_bypasses_manage_channels() {
        let mut ctx = base_ctx();
        ctx.grants
            .merge_family(Family::Community, community::ADMINISTRATOR);
        assert!(resolve(&ctx, PermissionCode::COMMUNITY_MANAGE_CHANNELS).is_allow());
    }

    #[test]
    fn restricted_space_blocks_visitor() {
        let mut ctx = base_ctx();
        ctx.grants.merge_family(Family::Text, text::VIEW);
        ctx.space_restricted = true;
        ctx.is_space_member = false;
        assert!(!resolve(&ctx, PermissionCode::TEXT_VIEW).is_allow());
    }

    #[test]
    fn non_member_denied() {
        let ctx = ActorContext {
            community_id: Uuid::now_v7(),
            account_id: Uuid::now_v7(),
            is_community_member: false,
            is_community_owner: false,
            is_instance_admin: false,
            grants: default_everyone_grants(),
            space_id: None,
            space_restricted: false,
            is_space_member: false,
            timeout_until: None,
        };
        assert!(!resolve(&ctx, PermissionCode::TEXT_VIEW).is_allow());
    }

    #[test]
    fn timeout_strips_send() {
        let mut ctx = base_ctx();
        ctx.grants.merge_family(Family::Text, text::VIEW | text::SEND);
        ctx.timeout_until = Some(Utc::now() + chrono::Duration::hours(1));
        assert!(!resolve(&ctx, PermissionCode::TEXT_SEND).is_allow());
    }

    #[test]
    fn owner_always_allowed() {
        let ctx = ActorContext {
            community_id: Uuid::now_v7(),
            account_id: Uuid::now_v7(),
            is_community_member: true,
            is_community_owner: true,
            is_instance_admin: false,
            grants: GrantSet::new(),
            space_id: None,
            space_restricted: true,
            is_space_member: false,
            timeout_until: None,
        };
        assert!(resolve(&ctx, PermissionCode::TEXT_VIEW).is_allow());
    }
}
