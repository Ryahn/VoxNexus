//! View As channel list simulation (F032).
//!
//! Simulation only: builds a synthetic [`ActorContext`] and filters channels with
//! the same `resolve` + override path as live checks. Mutating APIs still use the
//! real session actor.

#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_many_lines
)]

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use uuid::Uuid;
use voxnexus_auth::{
    get_account, get_community, get_everyone_role, get_membership, get_profile, get_role,
    get_space, list_channels as persist_list, member_roles_for_grants,
};
use voxnexus_domain::{Channel, CommunityRole, SpaceVisibility};
use voxnexus_permissions::{roles_context, visitor_context, ActorContext, PermissionCode};
use voxnexus_protocol::error_codes;
use voxnexus_protocol::{
    ChannelResponse, ViewAsChannelsRequest, ViewAsChannelsResponse, ViewAsMode,
};

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};
use crate::permissions::{
    actor_context, allowed_for_channel_ctx, merge_role_grants_public, require_manage_channels,
};

/// Simulate which channels a member / role set / visitor can see.
#[utoipa::path(
    post,
    path = "/api/v1/permissions/view-as/channels",
    operation_id = "viewAsChannels",
    tag = "permissions",
    request_body = ViewAsChannelsRequest,
    responses(
        (status = 200, description = "Simulated channel list", body = ViewAsChannelsResponse),
        (status = 400, description = "Invalid request", body = voxnexus_protocol::ErrorBody),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn view_as_channels(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    ValidatedJson(body): ValidatedJson<ViewAsChannelsRequest>,
) -> Result<Json<ViewAsChannelsResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_manage_channels(
        &state,
        body.community_id,
        user.account_id,
        request_id.clone(),
    )
    .await?;

    get_community(&state.pool, body.community_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "view-as community lookup failed");
            internal(&request_id)
        })?
        .ok_or_else(|| not_found(&request_id))?;

    if let Some(space_id) = body.space_id {
        let space = get_space(&state.pool, space_id)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "view-as space lookup failed");
                internal(&request_id)
            })?
            .ok_or_else(|| not_found(&request_id))?;
        if space.community_id != body.community_id {
            return Err(not_found(&request_id));
        }
    }

    let (label, ctx, role_ids, member_account_id) =
        build_subject(&state, &body, &request_id).await?;

    let channels = persist_list(&state.pool, body.community_id, body.space_id, None, false)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "view-as list channels failed");
            internal(&request_id)
        })?;

    let mut visible = Vec::new();
    for channel in channels {
        let mut subject = ctx.clone();
        if let Some(space_id) = channel.space_id {
            apply_space_gate(
                &state,
                &mut subject,
                space_id,
                member_account_id,
                body.mode,
                &request_id,
            )
            .await?;
        } else {
            subject.space_id = None;
            subject.space_restricted = false;
            subject.is_space_member = false;
        }
        if allowed_for_channel_ctx(
            &state,
            &channel,
            subject,
            &role_ids,
            member_account_id,
            PermissionCode::TEXT_VIEW,
        )
        .await?
        {
            visible.push(channel);
        }
    }

    Ok(Json(ViewAsChannelsResponse {
        mode: body.mode,
        label,
        channels: visible.iter().map(to_response).collect(),
    }))
}

async fn build_subject(
    state: &AppState,
    body: &ViewAsChannelsRequest,
    request_id: &str,
) -> Result<(String, ActorContext, Vec<Uuid>, Option<Uuid>), ApiError> {
    match body.mode {
        ViewAsMode::Visitor => {
            let mut ctx = visitor_context(body.community_id);
            ctx.space_id = body.space_id;
            Ok(("Visitor".to_owned(), ctx, Vec::new(), None))
        }
        ViewAsMode::Member => {
            let account_id = body
                .account_id
                .ok_or_else(|| validation(request_id, "account_id is required for mode=member"))?;
            let membership = get_membership(&state.pool, body.community_id, account_id)
                .await
                .map_err(|error| {
                    tracing::error!(error = %error, "view-as membership lookup failed");
                    internal(request_id)
                })?;
            if membership.is_none() {
                return Err(not_found(request_id));
            }
            get_account(&state.pool, account_id)
                .await
                .map_err(|error| {
                    tracing::error!(error = %error, "view-as account lookup failed");
                    internal(request_id)
                })?
                .ok_or_else(|| not_found(request_id))?;
            let ctx = actor_context(state, body.community_id, account_id, body.space_id).await?;
            let roles = member_roles_for_grants(&state.pool, body.community_id, account_id)
                .await
                .map_err(|error| {
                    tracing::error!(error = %error, "view-as role load failed");
                    internal(request_id)
                })?;
            let role_ids: Vec<Uuid> = roles.iter().map(|role| role.id).collect();
            let label = get_profile(&state.pool, account_id)
                .await
                .ok()
                .flatten()
                .map(|profile| profile.display_name)
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| account_id.to_string());
            Ok((label, ctx, role_ids, Some(account_id)))
        }
        ViewAsMode::Roles => {
            let roles = load_roles_for_view_as(state, body, request_id).await?;
            let role_ids: Vec<Uuid> = roles.iter().map(|role| role.id).collect();
            let grants = merge_role_grants_public(&roles);
            let space_restricted = space_restricted(state, body.space_id, request_id).await?;
            let ctx = roles_context(body.community_id, grants, body.space_id, space_restricted);
            let label = roles
                .iter()
                .filter(|role| !role.is_everyone)
                .map(|role| role.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let label = if label.is_empty() {
                "@everyone".to_owned()
            } else {
                format!("@everyone + {label}")
            };
            Ok((label, ctx, role_ids, None))
        }
    }
}

async fn load_roles_for_view_as(
    state: &AppState,
    body: &ViewAsChannelsRequest,
    request_id: &str,
) -> Result<Vec<CommunityRole>, ApiError> {
    let everyone = get_everyone_role(&state.pool, body.community_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "view-as everyone role failed");
            internal(request_id)
        })?
        .ok_or_else(|| validation(request_id, "Community is missing @everyone."))?;

    let mut roles = vec![everyone];
    for role_id in &body.role_ids {
        let role = get_role(&state.pool, *role_id)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "view-as role lookup failed");
                internal(request_id)
            })?
            .ok_or_else(|| not_found(request_id))?;
        if role.community_id != body.community_id {
            return Err(not_found(request_id));
        }
        if role.is_everyone {
            continue;
        }
        if roles.iter().any(|existing| existing.id == role.id) {
            continue;
        }
        roles.push(role);
    }
    Ok(roles)
}

async fn space_restricted(
    state: &AppState,
    space_id: Option<Uuid>,
    request_id: &str,
) -> Result<bool, ApiError> {
    let Some(space_id) = space_id else {
        return Ok(false);
    };
    let space = get_space(&state.pool, space_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "view-as space flags failed");
            internal(request_id)
        })?
        .ok_or_else(|| not_found(request_id))?;
    Ok(space.visibility == SpaceVisibility::Restricted)
}

async fn apply_space_gate(
    state: &AppState,
    ctx: &mut ActorContext,
    space_id: Uuid,
    member_account_id: Option<Uuid>,
    mode: ViewAsMode,
    request_id: &str,
) -> Result<(), ApiError> {
    let space = get_space(&state.pool, space_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "view-as per-channel space failed");
            internal(request_id)
        })?
        .ok_or_else(|| not_found(request_id))?;
    ctx.space_id = Some(space_id);
    ctx.space_restricted = space.visibility == SpaceVisibility::Restricted;
    ctx.is_space_member = match mode {
        ViewAsMode::Visitor => false,
        ViewAsMode::Roles => true,
        ViewAsMode::Member => {
            let account_id = member_account_id.unwrap_or(Uuid::nil());
            voxnexus_auth::is_space_member(&state.pool, space_id, account_id)
                .await
                .map_err(|error| {
                    tracing::error!(error = %error, "view-as space membership failed");
                    internal(request_id)
                })?
        }
    };
    Ok(())
}

fn to_response(channel: &Channel) -> ChannelResponse {
    ChannelResponse {
        id: channel.id,
        community_id: channel.community_id,
        space_id: channel.space_id,
        category_id: channel.category_id,
        channel_type: channel.channel_type,
        name: channel.name.clone(),
        topic: channel.topic.clone(),
        position: channel.position,
        archived_at: channel.archived_at,
        config: channel.config.clone(),
        created_at: channel.created_at,
        updated_at: channel.updated_at,
    }
}

fn validation(request_id: &str, message: impl Into<String>) -> ApiError {
    ApiError::new(
        StatusCode::BAD_REQUEST,
        error_codes::VALIDATION_ERROR,
        message,
        None,
        request_id.to_owned(),
    )
}

fn not_found(request_id: &str) -> ApiError {
    ApiError::not_found(request_id.to_owned())
}

fn internal(request_id: &str) -> ApiError {
    ApiError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        error_codes::INTERNAL,
        "Unexpected server error.",
        None,
        request_id.to_owned(),
    )
}
