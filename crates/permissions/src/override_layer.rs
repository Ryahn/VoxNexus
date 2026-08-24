//! Category/channel override layers (F030).

use uuid::Uuid;

use crate::family::Family;
use crate::family::GrantSet;
use crate::grants::RolePermissionSet;

/// Overrides loaded for one channel (category + channel scopes).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OverrideBundle {
    pub category_roles: Vec<(Uuid, RolePermissionSet)>,
    pub category_member: Option<RolePermissionSet>,
    pub channel_roles: Vec<(Uuid, RolePermissionSet)>,
    pub channel_member: Option<RolePermissionSet>,
}

/// Apply category then channel override layers onto collapsed role grants.
#[must_use]
pub fn apply_override_layers(
    mut grants: GrantSet,
    bundle: &OverrideBundle,
    actor_role_ids: &[Uuid],
) -> GrantSet {
    let category = collapse_layer(
        &bundle.category_roles,
        actor_role_ids,
        bundle.category_member.as_ref(),
    );
    grants = apply_layer(grants, &category);
    let channel = collapse_layer(
        &bundle.channel_roles,
        actor_role_ids,
        bundle.channel_member.as_ref(),
    );
    grants = apply_layer(grants, &channel);
    grants
}

fn collapse_layer(
    role_overrides: &[(Uuid, RolePermissionSet)],
    actor_role_ids: &[Uuid],
    member: Option<&RolePermissionSet>,
) -> RolePermissionSet {
    let mut allow = GrantSet::new();
    let mut deny = GrantSet::new();
    for (role_id, perms) in role_overrides {
        if actor_role_ids.contains(role_id) {
            merge_role_into_layer(&mut allow, &mut deny, perms);
        }
    }
    let mut layer = RolePermissionSet { allow, deny };
    if let Some(member_perms) = member {
        apply_member_overwrite(&mut layer, member_perms);
    }
    layer
}

fn merge_role_into_layer(
    layer_allow: &mut GrantSet,
    layer_deny: &mut GrantSet,
    perms: &RolePermissionSet,
) {
    for family in [
        Family::Community,
        Family::Space,
        Family::Text,
        Family::Voice,
    ] {
        let allow = perms.allow.get(family);
        let deny = perms.deny.get(family);
        // Within one role overwrite, deny wins over allow on the same bit.
        let effective_allow = allow & !deny;
        // Across roles, OR allows and denies independently; apply_layer lets
        // allow restore bits that another role denied (Discord semantics).
        layer_deny.merge_family(family, deny);
        layer_allow.merge_family(family, effective_allow);
    }
}

fn apply_member_overwrite(layer: &mut RolePermissionSet, member: &RolePermissionSet) {
    for family in [
        Family::Community,
        Family::Space,
        Family::Text,
        Family::Voice,
    ] {
        let deny = member.deny.get(family);
        let allow = member.allow.get(family) & !deny;
        if deny != 0 {
            layer
                .allow
                .set_family(family, layer.allow.get(family) & !deny);
            layer.deny.merge_family(family, deny);
        }
        if allow != 0 {
            // Member allow clears prior role deny on the same bits.
            layer
                .deny
                .set_family(family, layer.deny.get(family) & !allow);
            layer
                .allow
                .set_family(family, layer.allow.get(family) | allow);
        }
    }
}

fn apply_layer(mut grants: GrantSet, layer: &RolePermissionSet) -> GrantSet {
    for family in [
        Family::Community,
        Family::Space,
        Family::Text,
        Family::Voice,
    ] {
        let deny = layer.deny.get(family);
        let allow = layer.allow.get(family);
        let current = grants.get(family);
        // Deny first, then allow — so a role allow can restore after another
        // role's deny in the same layer.
        let next = (current & !deny) | allow;
        grants.set_family(family, next);
    }
    grants
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::family::{text, Family};
    use crate::grants::RolePermissionSet;

    #[test]
    fn channel_deny_view_hides_channel() {
        let grants = GrantSet::new().with_family(Family::Text, text::VIEW);
        let role_id = Uuid::now_v7();
        let bundle = OverrideBundle {
            channel_roles: vec![(
                role_id,
                RolePermissionSet::new().with_deny(Family::Text, text::VIEW),
            )],
            ..Default::default()
        };
        let effective = apply_override_layers(grants, &bundle, &[role_id]);
        assert!(!effective.has(Family::Text, text::VIEW));
    }

    #[test]
    fn channel_allow_restores_after_category_deny() {
        let grants = GrantSet::new().with_family(Family::Text, text::VIEW | text::SEND);
        let role_id = Uuid::now_v7();
        let bundle = OverrideBundle {
            category_roles: vec![(
                role_id,
                RolePermissionSet::new().with_deny(Family::Text, text::SEND),
            )],
            channel_roles: vec![(
                role_id,
                RolePermissionSet::new().with_allow(Family::Text, text::SEND),
            )],
            ..Default::default()
        };
        let effective = apply_override_layers(grants, &bundle, &[role_id]);
        assert!(effective.has(Family::Text, text::SEND));
    }

    #[test]
    fn role_allow_beats_everyone_deny_on_same_channel() {
        let grants = GrantSet::new().with_family(Family::Text, text::VIEW);
        let everyone = Uuid::now_v7();
        let vip = Uuid::now_v7();
        let bundle = OverrideBundle {
            channel_roles: vec![
                (
                    everyone,
                    RolePermissionSet::new().with_deny(Family::Text, text::VIEW),
                ),
                (
                    vip,
                    RolePermissionSet::new().with_allow(Family::Text, text::VIEW),
                ),
            ],
            ..Default::default()
        };
        let without_vip = apply_override_layers(grants, &bundle, &[everyone]);
        assert!(!without_vip.has(Family::Text, text::VIEW));
        let with_vip = apply_override_layers(grants, &bundle, &[everyone, vip]);
        assert!(with_vip.has(Family::Text, text::VIEW));
    }

    #[test]
    fn member_allow_beats_role_deny_on_same_channel() {
        let grants = GrantSet::new().with_family(Family::Text, text::VIEW);
        let everyone = Uuid::now_v7();
        let bundle = OverrideBundle {
            channel_roles: vec![(
                everyone,
                RolePermissionSet::new().with_deny(Family::Text, text::VIEW),
            )],
            channel_member: Some(RolePermissionSet::new().with_allow(Family::Text, text::VIEW)),
            ..Default::default()
        };
        let effective = apply_override_layers(grants, &bundle, &[everyone]);
        assert!(effective.has(Family::Text, text::VIEW));
    }
}
