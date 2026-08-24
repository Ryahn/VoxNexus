//! Permission explain API (F031).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use uuid::Uuid;
use voxnexus_auth::{get_channel, get_membership, member_roles_for_grants};
use voxnexus_permissions::{
    apply_override_layers, resolve_traced, ExplainStep, OverrideBundle, PermissionCode,
};
use voxnexus_protocol::error_codes;
use voxnexus_protocol::{
    ExplainPermissionRequest, ExplainPermissionResponse, PermissionExplainStep,
};

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};
use crate::permissions::{actor_context, merge_role_grants_public};

/// Explain why an actor has or lacks a permission on a community or channel resource.
#[utoipa::path(
    post,
    path = "/api/v1/permissions/explain",
    operation_id = "explainPermission",
    tag = "permissions",
    request_body = ExplainPermissionRequest,
    responses(
        (status = 200, description = "Explanation", body = ExplainPermissionResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn explain_permission(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    ValidatedJson(body): ValidatedJson<ExplainPermissionRequest>,
) -> Result<Json<ExplainPermissionResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let permission = PermissionCode::parse(&body.permission).ok_or_else(|| {
        validation(
            request_id.clone(),
            "Unknown permission code. Use text.view, text.send, community.manage_channels, etc.",
        )
    })?;

    if body.account_id != user.account_id {
        crate::permissions::require_manage_channels(
            &state,
            body.community_id,
            user.account_id,
            request_id.clone(),
        )
        .await?;
    }

    let subject_membership = get_membership(&state.pool, body.community_id, body.account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "explain membership lookup failed");
            internal(request_id.clone())
        })?;
    if subject_membership.is_none() {
        return Err(not_found(request_id));
    }

    let channel = if let Some(channel_id) = body.channel_id {
        let channel = get_channel(&state.pool, channel_id)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "explain channel lookup failed");
                internal(request_id.clone())
            })?
            .ok_or_else(|| not_found(request_id.clone()))?;
        if channel.community_id != body.community_id {
            return Err(not_found(request_id));
        }
        Some(channel)
    } else {
        None
    };

    let space_id = channel.as_ref().and_then(|ch| ch.space_id);
    let mut steps = Vec::new();
    let mut ctx = actor_context(&state, body.community_id, body.account_id, space_id).await?;

    let roles = member_roles_for_grants(&state.pool, body.community_id, body.account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "explain role load failed");
            internal(request_id.clone())
        })?;
    let role_ids: Vec<Uuid> = roles.iter().map(|role| role.id).collect();
    let base = merge_role_grants_public(&roles);
    steps.push(map_step(ExplainStep {
        stage: "roles",
        outcome: "continue",
        detail: if roles.is_empty() {
            "No roles assigned.".to_owned()
        } else {
            roles
                .iter()
                .map(|role| {
                    if role.is_everyone {
                        format!("@everyone (w{})", role.weight)
                    } else {
                        format!("{} (w{})", role.name, role.weight)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        },
    }));
    steps.push(grant_step("role_grants", &base, permission));

    if let Some(channel) = &channel {
        let bundle = voxnexus_auth::override_bundle_for_channel(
            &state.pool,
            channel.id,
            channel.category_id,
            body.account_id,
        )
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "explain override load failed");
            internal(request_id.clone())
        })?;

        let category_only = OverrideBundle {
            category_roles: bundle.category_roles.clone(),
            category_member: bundle.category_member.clone(),
            ..OverrideBundle::default()
        };
        let after_category = apply_override_layers(base, &category_only, &role_ids);
        if category_only != OverrideBundle::default() {
            steps.push(grant_step("category_override", &after_category, permission));
        }

        let channel_only = OverrideBundle {
            channel_roles: bundle.channel_roles.clone(),
            channel_member: bundle.channel_member.clone(),
            ..OverrideBundle::default()
        };
        let after_channel = apply_override_layers(after_category, &channel_only, &role_ids);
        if channel_only != OverrideBundle::default() {
            steps.push(grant_step("channel_override", &after_channel, permission));
        }
        ctx.grants = after_channel;
    } else {
        ctx.grants = base;
    }

    let (decision, resolve_steps) = resolve_traced(&ctx, permission);
    steps.extend(resolve_steps.into_iter().map(map_step));

    let allowed = decision.is_allow();

    Ok(Json(ExplainPermissionResponse {
        allowed,
        permission: permission.as_str().to_owned(),
        account_id: body.account_id,
        channel_id: body.channel_id,
        steps,
    }))
}

fn grant_step(
    stage: &'static str,
    grants: &voxnexus_permissions::GrantSet,
    permission: PermissionCode,
) -> PermissionExplainStep {
    let has = grants.has(permission.family, permission.bit);
    PermissionExplainStep {
        stage: stage.to_owned(),
        outcome: if has { "allow" } else { "deny" }.to_owned(),
        detail: format!(
            "Effective {} bit for {} is {}.",
            permission.as_str(),
            stage.replace('_', " "),
            if has { "set" } else { "clear" }
        ),
    }
}

fn map_step(step: ExplainStep) -> PermissionExplainStep {
    PermissionExplainStep {
        stage: step.stage.to_owned(),
        outcome: step.outcome.to_owned(),
        detail: step.detail,
    }
}

fn validation(request_id: String, message: impl Into<String>) -> ApiError {
    ApiError::new(
        StatusCode::BAD_REQUEST,
        error_codes::VALIDATION_ERROR,
        message,
        None,
        request_id,
    )
}

fn not_found(request_id: String) -> ApiError {
    ApiError::not_found(request_id)
}

fn internal(request_id: String) -> ApiError {
    ApiError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        error_codes::INTERNAL,
        "Unexpected server error.",
        None,
        request_id,
    )
}
