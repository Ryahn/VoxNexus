//! Synthetic `ActorContext` builders for View As (F032).
//!
//! These helpers only construct evaluation context. Callers must still run
//! [`crate::resolve`] / override application — never a parallel permission path.

use uuid::Uuid;

use crate::eval::ActorContext;
use crate::family::GrantSet;

/// Non-member visitor: fails the membership gate for every permission.
#[must_use]
pub fn visitor_context(community_id: Uuid) -> ActorContext {
    ActorContext {
        community_id,
        account_id: Uuid::nil(),
        is_community_member: false,
        is_community_owner: false,
        is_instance_admin: false,
        grants: GrantSet::new(),
        space_id: None,
        space_restricted: false,
        is_space_member: false,
        timeout_until: None,
    }
}

/// Synthetic member holding the given role grants (no owner / admin / timeout).
///
/// `account_id` is [`Uuid::nil`] so channel **member** overrides do not apply.
/// When `space_id` is set, the actor is treated as a Space member so restricted
/// Spaces can still be previewed for the selected roles.
#[must_use]
pub fn roles_context(
    community_id: Uuid,
    grants: GrantSet,
    space_id: Option<Uuid>,
    space_restricted: bool,
) -> ActorContext {
    ActorContext {
        community_id,
        account_id: Uuid::nil(),
        is_community_member: true,
        is_community_owner: false,
        is_instance_admin: false,
        grants,
        space_id,
        space_restricted,
        is_space_member: space_id.is_some(),
        timeout_until: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code::PermissionCode;
    use crate::eval::resolve;
    use crate::family::{text, Family};

    #[test]
    fn visitor_denied_text_view() {
        let ctx = visitor_context(Uuid::now_v7());
        assert!(!resolve(&ctx, PermissionCode::TEXT_VIEW).is_allow());
    }

    #[test]
    fn roles_context_uses_grants() {
        let grants = GrantSet::new().with_family(Family::Text, text::VIEW);
        let ctx = roles_context(Uuid::now_v7(), grants, None, false);
        assert!(resolve(&ctx, PermissionCode::TEXT_VIEW).is_allow());
        assert_eq!(ctx.account_id, Uuid::nil());
    }
}
