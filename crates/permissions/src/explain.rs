//! Permission explanation trace (F031).

use crate::code::PermissionCode;
use crate::eval::{ActorContext, Decision, resolve};

/// One step in a permission explanation chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainStep {
    pub stage: &'static str,
    pub outcome: &'static str,
    pub detail: String,
}

/// Resolve with an audit trail matching [`resolve`].
#[must_use]
pub fn resolve_traced(ctx: &ActorContext, permission: PermissionCode) -> (Decision, Vec<ExplainStep>) {
    let mut steps = Vec::new();
    if !ctx.is_community_member && !ctx.is_instance_admin {
        steps.push(step(
            "membership",
            "deny",
            "Actor is not a community member.",
        ));
        return (Decision::Deny, steps);
    }
    steps.push(step(
        "membership",
        "continue",
        if ctx.is_instance_admin {
            "Instance administrator (member gate skipped)."
        } else {
            "Actor is a community member."
        },
    ));

    if ctx.is_community_owner {
        steps.push(step("owner", "allow", "Community owner bypass."));
        return (Decision::Allow, steps);
    }
    steps.push(step("owner", "continue", "Actor is not the community owner."));

    if ctx.space_restricted && !ctx.is_space_member {
        steps.push(step(
            "space",
            "deny",
            "Space is restricted and actor is not a space member.",
        ));
        return (Decision::Deny, steps);
    }
    if ctx.space_restricted {
        steps.push(step("space", "continue", "Space is restricted; actor is a space member."));
    } else {
        steps.push(step("space", "continue", "No restricted-space gate."));
    }

    if ctx.has_administrator() && !permission.owner_only {
        steps.push(step(
            "administrator",
            "allow",
            "Administrator bypass for this permission.",
        ));
        return (Decision::Allow, steps);
    }
    if ctx.has_administrator() {
        steps.push(step(
            "administrator",
            "continue",
            "Administrator does not bypass owner-only permissions.",
        ));
    } else {
        steps.push(step("administrator", "continue", "No administrator bypass."));
    }

    if ctx.is_timed_out() && permission_stripped_by_timeout(permission) {
        steps.push(step(
            "timeout",
            "deny",
            "Timed-out members cannot send messages.",
        ));
        return (Decision::Deny, steps);
    }
    if ctx.is_timed_out() {
        steps.push(step("timeout", "continue", "Timeout does not affect this permission."));
    } else {
        steps.push(step("timeout", "continue", "Actor is not timed out."));
    }

    if ctx.grants.has(permission.family, permission.bit) {
        steps.push(step(
            "grants",
            "allow",
            format!(
                "Effective grants include {}.",
                permission.as_str()
            ),
        ));
        return (Decision::Allow, steps);
    }

    steps.push(step(
        "grants",
        "deny",
        format!(
            "Effective grants do not include {}.",
            permission.as_str()
        ),
    ));
    let decision = resolve(ctx, permission);
    debug_assert_eq!(decision, Decision::Deny);
    (Decision::Deny, steps)
}

fn step(stage: &'static str, outcome: &'static str, detail: impl Into<String>) -> ExplainStep {
    ExplainStep {
        stage,
        outcome,
        detail: detail.into(),
    }
}

fn permission_stripped_by_timeout(permission: PermissionCode) -> bool {
    matches!(permission, PermissionCode::TEXT_SEND)
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::family::{text, Family, GrantSet};

    fn base_ctx() -> ActorContext {
        ActorContext {
            community_id: Uuid::now_v7(),
            account_id: Uuid::now_v7(),
            is_community_member: true,
            is_community_owner: false,
            is_instance_admin: false,
            grants: GrantSet::new().with_family(Family::Text, text::VIEW),
            space_id: None,
            space_restricted: false,
            is_space_member: false,
            timeout_until: None,
        }
    }

    #[test]
    fn traced_matches_resolve_allow() {
        let ctx = base_ctx();
        let (decision, _) = resolve_traced(&ctx, PermissionCode::TEXT_VIEW);
        assert_eq!(decision, resolve(&ctx, PermissionCode::TEXT_VIEW));
        assert!(decision.is_allow());
    }

    #[test]
    fn traced_matches_resolve_deny() {
        let ctx = base_ctx();
        let (decision, steps) = resolve_traced(&ctx, PermissionCode::TEXT_SEND);
        assert_eq!(decision, resolve(&ctx, PermissionCode::TEXT_SEND));
        assert!(!decision.is_allow());
        assert!(steps.iter().any(|s| s.stage == "grants"));
    }
}
