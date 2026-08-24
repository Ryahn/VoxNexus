//! Load actor contexts and enforce permissions (F029).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use uuid::Uuid;
use voxnexus_auth::{
    get_account, get_community, get_membership, get_space, is_space_member,
    member_roles_for_grants, override_bundle_for_channel,
};
use voxnexus_domain::{Channel, SpaceVisibility};
use voxnexus_permissions::{
    apply_override_layers, collapse_roles_by_weight, parse_role_permissions, ActorContext,
    PermissionCode, resolve, GrantSet,
};

use crate::error::ApiError;
use crate::http::AppState;

/// Load (or cache) permission context for an actor in a community scope.
pub async fn actor_context(
    state: &AppState,
    community_id: Uuid,
    account_id: Uuid,
    space_id: Option<Uuid>,
) -> Result<ActorContext, ApiError> {
    if let Some(cached) = state.permission_cache.get(community_id, account_id) {
        if cached.space_id == space_id {
            return Ok(cached);
        }
    }

    let community = get_community(&state.pool, community_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "community lookup failed");
            internal("permission community lookup")
        })?
        .ok_or_else(|| not_found("permission community"))?;

    let membership = get_membership(&state.pool, community_id, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "membership lookup failed");
            internal("permission membership lookup")
        })?;

    let account = get_account(&state.pool, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "account lookup failed");
            internal("permission account lookup")
        })?
        .ok_or_else(|| not_found("permission account"))?;

    let grants = if membership.is_some() {
        let roles = member_roles_for_grants(&state.pool, community_id, account_id)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "role grant load failed");
                internal("permission role load")
            })?;
        merge_role_grants(&roles)
    } else {
        GrantSet::new()
    };

    let (space_restricted, is_space_member_flag) = if let Some(space_id) = space_id {
        let space = get_space(&state.pool, space_id)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "space lookup failed");
                internal("permission space lookup")
            })?
            .ok_or_else(|| not_found("permission space"))?;
        let restricted = space.visibility == SpaceVisibility::Restricted;
        let member = is_space_member(&state.pool, space_id, account_id)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "space membership lookup failed");
                internal("permission space membership")
            })?;
        (restricted, member)
    } else {
        (false, false)
    };

    let context = ActorContext {
        community_id,
        account_id,
        is_community_member: membership.is_some(),
        is_community_owner: community.owner_account_id == account_id,
        is_instance_admin: account.is_instance_admin,
        grants,
        space_id,
        space_restricted,
        is_space_member: is_space_member_flag,
        timeout_until: None,
    };
    state.permission_cache.put(context.clone());
    Ok(context)
}

/// Whether the actor may exercise `permission` (after space gate).
pub async fn allowed(
    state: &AppState,
    community_id: Uuid,
    account_id: Uuid,
    space_id: Option<Uuid>,
    permission: PermissionCode,
) -> Result<bool, ApiError> {
    let ctx = actor_context(state, community_id, account_id, space_id).await?;
    Ok(resolve(&ctx, permission).is_allow())
}

/// Whether the actor may exercise `permission` on a specific channel (includes overrides).
pub async fn allowed_for_channel(
    state: &AppState,
    channel: &Channel,
    account_id: Uuid,
    permission: PermissionCode,
) -> Result<bool, ApiError> {
    let mut ctx =
        actor_context(state, channel.community_id, account_id, channel.space_id).await?;
    ctx.grants = effective_grants_for_channel(state, channel, account_id, ctx.grants).await?;
    Ok(resolve(&ctx, permission).is_allow())
}

async fn effective_grants_for_channel(
    state: &AppState,
    channel: &Channel,
    account_id: Uuid,
    base: GrantSet,
) -> Result<GrantSet, ApiError> {
    let roles = member_roles_for_grants(&state.pool, channel.community_id, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "role grant load failed");
            internal("permission role load")
        })?;
    let role_ids: Vec<Uuid> = roles.iter().map(|role| role.id).collect();
    let bundle = override_bundle_for_channel(
        &state.pool,
        channel.id,
        channel.category_id,
        account_id,
    )
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "override load failed");
        internal("permission override load")
    })?;
    Ok(apply_override_layers(base, &bundle, &role_ids))
}

/// Require `permission` or return 403 (404 when the community does not exist).
pub async fn require_permission(
    state: &AppState,
    community_id: Uuid,
    account_id: Uuid,
    space_id: Option<Uuid>,
    permission: PermissionCode,
    request_id: String,
    message: impl Into<String>,
) -> Result<(), ApiError> {
    let membership = get_membership(&state.pool, community_id, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "membership lookup failed");
            internal("permission membership lookup")
        })?;
    if membership.is_none()
        && get_community(&state.pool, community_id)
            .await
            .ok()
            .flatten()
            .is_none()
    {
        return Err(not_found(request_id));
    }
    if allowed(state, community_id, account_id, space_id, permission).await? {
        Ok(())
    } else {
        Err(ApiError::permission_denied(request_id, message))
    }
}

/// Require `community.manage_channels`.
pub async fn require_manage_channels(
    state: &AppState,
    community_id: Uuid,
    account_id: Uuid,
    request_id: String,
) -> Result<(), ApiError> {
    require_permission(
        state,
        community_id,
        account_id,
        None,
        PermissionCode::COMMUNITY_MANAGE_CHANNELS,
        request_id,
        "You do not have permission to manage channels.",
    )
    .await
}

/// Filter channels to those the actor may view (`text.view`).
pub async fn visible_channels(
    state: &AppState,
    _community_id: Uuid,
    account_id: Uuid,
    channels: Vec<Channel>,
) -> Result<Vec<Channel>, ApiError> {
    let mut visible = Vec::new();
    for channel in channels {
        if allowed_for_channel(
            state,
            &channel,
            account_id,
            PermissionCode::TEXT_VIEW,
        )
        .await?
        {
            visible.push(channel);
        }
    }
    Ok(visible)
}

/// Require `text.view` for a channel (hidden channels return 404).
pub async fn require_channel_view(
    state: &AppState,
    channel: &Channel,
    account_id: Uuid,
    request_id: String,
) -> Result<(), ApiError> {
    let membership = get_membership(&state.pool, channel.community_id, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "membership lookup failed");
            internal("permission membership lookup")
        })?;
    if membership.is_none() {
        if get_community(&state.pool, channel.community_id)
            .await
            .ok()
            .flatten()
            .is_none()
        {
            return Err(not_found(request_id));
        }
        return Err(ApiError::permission_denied(
            request_id,
            "You must be a community member to view channels.",
        ));
    }
    if allowed_for_channel(state, channel, account_id, PermissionCode::TEXT_VIEW).await? {
        Ok(())
    } else {
        Err(not_found(request_id))
    }
}

pub fn invalidate_community(state: &AppState, community_id: Uuid) {
    state.permission_cache.invalidate_community(community_id);
}

fn merge_role_grants(roles: &[voxnexus_domain::CommunityRole]) -> GrantSet {
    merge_role_grants_public(roles)
}

/// Collapse assigned roles into effective grants (for explain / debug).
pub fn merge_role_grants_public(roles: &[voxnexus_domain::CommunityRole]) -> GrantSet {
    let pairs: Vec<(i32, voxnexus_permissions::RolePermissionSet)> = roles
        .iter()
        .map(|role| (role.weight, parse_role_permissions(&role.permissions)))
        .collect();
    collapse_roles_by_weight(&pairs)
}

fn internal(message: &'static str) -> ApiError {
    ApiError::new(
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        voxnexus_protocol::error_codes::INTERNAL,
        message,
        None,
        "permission-internal",
    )
}

fn not_found(message: impl Into<String>) -> ApiError {
    ApiError::not_found(message)
}
