//! Tri-state role permission grants and weight collapse (F030-A).

use serde_json::{json, Value};

use crate::family::{community, text, Family, GrantSet};

/// Per-family allow / deny bitsets for one role (Inherit = neither bit set).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RolePermissionSet {
    pub allow: GrantSet,
    pub deny: GrantSet,
}

impl RolePermissionSet {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_allow(mut self, family: Family, mask: u64) -> Self {
        self.allow.merge_family(family, mask);
        self
    }

    #[must_use]
    pub fn with_deny(mut self, family: Family, mask: u64) -> Self {
        self.deny.merge_family(family, mask);
        self
    }
}

/// Default `@everyone` grants for new communities (view public text channels).
#[must_use]
pub fn default_everyone_grants() -> GrantSet {
    GrantSet::new().with_family(Family::Text, text::VIEW)
}

/// Default `@everyone` permission JSON (allow text.view).
#[must_use]
pub fn default_everyone_permissions_json() -> Value {
    json!({
        "allow": { "text": text::VIEW },
        "deny": {}
    })
}

/// Empty custom-role permissions (all inherit).
#[must_use]
pub fn empty_role_permissions_json() -> Value {
    json!({ "allow": {}, "deny": {} })
}

/// Permissions JSON granting `community.manage_roles` (and optional empty deny).
#[must_use]
pub fn permissions_with_manage_roles(enabled: bool) -> Value {
    if enabled {
        json!({
            "allow": { "community": community::MANAGE_ROLES },
            "deny": {}
        })
    } else {
        empty_role_permissions_json()
    }
}

/// Parse role permissions JSON into allow/deny sets.
///
/// Supports F030-A `{ allow, deny }`, F028 `families` / bare masks (as allow),
/// and `manage_roles: true`.
#[must_use]
pub fn parse_role_permissions(value: &Value) -> RolePermissionSet {
    let mut set = RolePermissionSet::new();
    if let Some(allow) = value.get("allow") {
        merge_family_map(&mut set.allow, allow);
    } else if let Some(families) = value.get("families") {
        merge_family_map(&mut set.allow, families);
    } else {
        if let Some(text_mask) = value.get("text").and_then(parse_mask_opt) {
            set.allow.merge_family(Family::Text, text_mask);
        }
        if let Some(community_mask) = value.get("community").and_then(parse_mask_opt) {
            set.allow.merge_family(Family::Community, community_mask);
        }
    }
    if let Some(deny) = value.get("deny") {
        merge_family_map(&mut set.deny, deny);
    }
    if value
        .get("manage_roles")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        set.allow
            .merge_family(Family::Community, community::MANAGE_ROLES);
    }
    // Clear overlapping bits: explicit deny wins within a single role.
    for family in [Family::Community, Family::Space, Family::Text, Family::Voice] {
        let deny = set.deny.get(family);
        if deny != 0 {
            let allow = set.allow.get(family) & !deny;
            set.allow.set_family(family, allow);
        }
    }
    set
}

/// Collapse roles by ascending weight (lower weight = higher priority).
///
/// For each bit, the first non-Inherit (Allow or Deny) wins. Deny removes the
/// bit from the final grant set; Allow adds it.
#[must_use]
pub fn collapse_roles_by_weight(roles: &[(i32, RolePermissionSet)]) -> GrantSet {
    let mut ordered = roles.to_vec();
    ordered.sort_by_key(|(weight, _)| *weight);

    let mut decided: GrantSet = GrantSet::new(); // bits that have a decision (allow or deny)
    let mut grants = GrantSet::new();

    for (_weight, perms) in ordered {
        for family in [Family::Community, Family::Space, Family::Text, Family::Voice] {
            let undecided = !(decided.get(family));
            let allow = perms.allow.get(family) & undecided;
            let deny = perms.deny.get(family) & undecided;
            if allow != 0 {
                grants.merge_family(family, allow);
                decided.merge_family(family, allow);
            }
            if deny != 0 {
                decided.merge_family(family, deny);
            }
        }
    }
    grants
}

/// Legacy helper: parse as allow-only GrantSet (OR of allow bits). Prefer
/// [`parse_role_permissions`] + [`collapse_roles_by_weight`].
#[must_use]
pub fn parse_role_permissions_allow_only(value: &Value) -> GrantSet {
    parse_role_permissions(value).allow
}

fn merge_family_map(grants: &mut GrantSet, value: &Value) {
    let Some(map) = value.as_object() else {
        return;
    };
    for (key, mask_value) in map {
        if let Some(family) = Family::parse(key) {
            let mask = match mask_value {
                Value::Object(obj) => obj
                    .get("allow")
                    .and_then(parse_mask_opt)
                    .or_else(|| parse_mask_opt(mask_value))
                    .unwrap_or(0),
                other => parse_mask(other),
            };
            grants.merge_family(family, mask);
        }
    }
}

fn parse_mask_opt(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn parse_mask(value: &Value) -> u64 {
    parse_mask_opt(value).unwrap_or(0)
}
