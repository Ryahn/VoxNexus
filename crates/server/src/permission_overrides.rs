//! Channel permission override HTTP API (F030).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use uuid::Uuid;
use voxnexus_auth::{
    delete_override as persist_delete, get_category, get_channel, get_membership, get_role,
    list_category_overrides as persist_list_category, list_channel_overrides as persist_list_channel,
    upsert_category_member_override, upsert_category_role_override, upsert_channel_member_override,
    upsert_channel_role_override, PermissionOverride,
};
use voxnexus_protocol::error_codes;
use voxnexus_protocol::{
    PermissionOverrideListResponse, PermissionOverrideResponse, UpsertPermissionOverrideRequest,
};

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};
use crate::permissions::invalidate_community;

/// List permission overrides for a channel.
#[utoipa::path(
    get,
    path = "/api/v1/channels/{channel_id}/permission-overrides",
    operation_id = "listChannelPermissionOverrides",
    tag = "channels",
    params(("channel_id" = Uuid, Path, description = "Channel id")),
    responses(
        (status = 200, description = "Override list", body = PermissionOverrideListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn list_channel_permission_overrides(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<PermissionOverrideListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let channel = load_channel(&state, channel_id, &request_id).await?;
    require_manage_channels(
        &state,
        channel.community_id,
        user.account_id,
        &request_id,
    )
    .await?;
    let rows = persist_list_channel(&state.pool, channel.id, channel.category_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    Ok(Json(PermissionOverrideListResponse {
        overrides: rows.into_iter().map(to_response).collect(),
    }))
}

/// Upsert a role override on a channel.
#[utoipa::path(
    put,
    path = "/api/v1/channels/{channel_id}/permission-overrides/roles/{role_id}",
    operation_id = "upsertChannelRolePermissionOverride",
    tag = "channels",
    params(
        ("channel_id" = Uuid, Path, description = "Channel id"),
        ("role_id" = Uuid, Path, description = "Role id")
    ),
    request_body = UpsertPermissionOverrideRequest,
    responses(
        (status = 200, description = "Override saved", body = PermissionOverrideResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn upsert_channel_role_permission_override(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path((channel_id, role_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(body): ValidatedJson<UpsertPermissionOverrideRequest>,
) -> Result<Json<PermissionOverrideResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let channel = load_channel(&state, channel_id, &request_id).await?;
    require_manage_channels(
        &state,
        channel.community_id,
        user.account_id,
        &request_id,
    )
    .await?;
    let role = get_role(&state.pool, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    if role.community_id != channel.community_id {
        return Err(not_found(request_id));
    }
    let row = upsert_channel_role_override(
        &state.pool,
        channel.community_id,
        channel.id,
        role_id,
        body.permissions,
    )
    .await
    .map_err(|error| map_auth(&error, request_id.clone()))?;
    invalidate_community(&state, channel.community_id);
    Ok(Json(to_response(row)))
}

/// Upsert a member override on a channel.
#[utoipa::path(
    put,
    path = "/api/v1/channels/{channel_id}/permission-overrides/members/{account_id}",
    operation_id = "upsertChannelMemberPermissionOverride",
    tag = "channels",
    params(
        ("channel_id" = Uuid, Path, description = "Channel id"),
        ("account_id" = Uuid, Path, description = "Member account id")
    ),
    request_body = UpsertPermissionOverrideRequest,
    responses(
        (status = 200, description = "Override saved", body = PermissionOverrideResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn upsert_channel_member_permission_override(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path((channel_id, account_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(body): ValidatedJson<UpsertPermissionOverrideRequest>,
) -> Result<Json<PermissionOverrideResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let channel = load_channel(&state, channel_id, &request_id).await?;
    require_manage_channels(
        &state,
        channel.community_id,
        user.account_id,
        &request_id,
    )
    .await?;
    let membership = get_membership(&state.pool, channel.community_id, account_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    let _ = membership;
    let row = upsert_channel_member_override(
        &state.pool,
        channel.community_id,
        channel.id,
        account_id,
        body.permissions,
    )
    .await
    .map_err(|error| map_auth(&error, request_id.clone()))?;
    invalidate_community(&state, channel.community_id);
    Ok(Json(to_response(row)))
}

/// Delete a permission override.
#[utoipa::path(
    delete,
    path = "/api/v1/communities/{community_id}/permission-overrides/{override_id}",
    operation_id = "deletePermissionOverride",
    tag = "channels",
    params(
        ("community_id" = Uuid, Path, description = "Community id"),
        ("override_id" = Uuid, Path, description = "Override id")
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn delete_permission_override(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path((community_id, override_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_manage_channels(&state, community_id, user.account_id, &request_id).await?;
    let deleted = persist_delete(&state.pool, community_id, override_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    if !deleted {
        return Err(not_found(request_id));
    }
    invalidate_community(&state, community_id);
    Ok(StatusCode::NO_CONTENT)
}

/// List permission overrides for a category.
#[utoipa::path(
    get,
    path = "/api/v1/categories/{category_id}/permission-overrides",
    operation_id = "listCategoryPermissionOverrides",
    tag = "categories",
    params(("category_id" = Uuid, Path, description = "Category id")),
    responses(
        (status = 200, description = "Override list", body = PermissionOverrideListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn list_category_permission_overrides(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(category_id): Path<Uuid>,
) -> Result<Json<PermissionOverrideListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let category = load_category(&state, category_id, &request_id).await?;
    require_manage_channels(
        &state,
        category.community_id,
        user.account_id,
        &request_id,
    )
    .await?;
    let rows = persist_list_category(&state.pool, category.id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    Ok(Json(PermissionOverrideListResponse {
        overrides: rows.into_iter().map(to_response).collect(),
    }))
}

/// Upsert a role override on a category.
#[utoipa::path(
    put,
    path = "/api/v1/categories/{category_id}/permission-overrides/roles/{role_id}",
    operation_id = "upsertCategoryRolePermissionOverride",
    tag = "categories",
    params(
        ("category_id" = Uuid, Path, description = "Category id"),
        ("role_id" = Uuid, Path, description = "Role id")
    ),
    request_body = UpsertPermissionOverrideRequest,
    responses(
        (status = 200, description = "Override saved", body = PermissionOverrideResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn upsert_category_role_permission_override(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path((category_id, role_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(body): ValidatedJson<UpsertPermissionOverrideRequest>,
) -> Result<Json<PermissionOverrideResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let category = load_category(&state, category_id, &request_id).await?;
    require_manage_channels(
        &state,
        category.community_id,
        user.account_id,
        &request_id,
    )
    .await?;
    let role = get_role(&state.pool, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    if role.community_id != category.community_id {
        return Err(not_found(request_id));
    }
    let row = upsert_category_role_override(
        &state.pool,
        category.community_id,
        category.id,
        role_id,
        body.permissions,
    )
    .await
    .map_err(|error| map_auth(&error, request_id.clone()))?;
    invalidate_community(&state, category.community_id);
    Ok(Json(to_response(row)))
}

/// Upsert a member override on a category.
#[utoipa::path(
    put,
    path = "/api/v1/categories/{category_id}/permission-overrides/members/{account_id}",
    operation_id = "upsertCategoryMemberPermissionOverride",
    tag = "categories",
    params(
        ("category_id" = Uuid, Path, description = "Category id"),
        ("account_id" = Uuid, Path, description = "Member account id")
    ),
    request_body = UpsertPermissionOverrideRequest,
    responses(
        (status = 200, description = "Override saved", body = PermissionOverrideResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn upsert_category_member_permission_override(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path((category_id, account_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(body): ValidatedJson<UpsertPermissionOverrideRequest>,
) -> Result<Json<PermissionOverrideResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let category = load_category(&state, category_id, &request_id).await?;
    require_manage_channels(
        &state,
        category.community_id,
        user.account_id,
        &request_id,
    )
    .await?;
    let membership = get_membership(&state.pool, category.community_id, account_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    let _ = membership;
    let row = upsert_category_member_override(
        &state.pool,
        category.community_id,
        category.id,
        account_id,
        body.permissions,
    )
    .await
    .map_err(|error| map_auth(&error, request_id.clone()))?;
    invalidate_community(&state, category.community_id);
    Ok(Json(to_response(row)))
}

async fn load_channel(
    state: &AppState,
    channel_id: Uuid,
    request_id: &str,
) -> Result<voxnexus_domain::Channel, ApiError> {
    get_channel(&state.pool, channel_id)
        .await
        .map_err(|error| map_auth(&error, request_id.to_owned()))?
        .ok_or_else(|| not_found(request_id.to_owned()))
}

async fn load_category(
    state: &AppState,
    category_id: Uuid,
    request_id: &str,
) -> Result<voxnexus_domain::ChannelCategory, ApiError> {
    get_category(&state.pool, category_id)
        .await
        .map_err(|error| map_auth(&error, request_id.to_owned()))?
        .ok_or_else(|| not_found(request_id.to_owned()))
}

fn to_response(row: PermissionOverride) -> PermissionOverrideResponse {
    PermissionOverrideResponse {
        id: row.id,
        community_id: row.community_id,
        channel_id: row.channel_id,
        category_id: row.category_id,
        role_id: row.role_id,
        account_id: row.account_id,
        permissions: row.permissions,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn not_found(request_id: String) -> ApiError {
    ApiError::not_found(request_id)
}

fn map_auth(error: &voxnexus_auth::AuthError, request_id: String) -> ApiError {
    tracing::error!(error = %error, "permission override auth error");
    internal(request_id)
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

async fn require_manage_channels(
    state: &AppState,
    community_id: Uuid,
    account_id: Uuid,
    request_id: &str,
) -> Result<(), ApiError> {
    crate::permissions::require_manage_channels(
        state,
        community_id,
        account_id,
        request_id.to_owned(),
    )
    .await
}
