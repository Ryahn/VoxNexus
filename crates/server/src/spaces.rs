//! Space CRUD (F022).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use uuid::Uuid;
use voxnexus_auth::{
    create_space as persist_create, delete_space as persist_delete, get_community, get_membership,
    get_space, list_spaces as persist_list, update_space as persist_update, CreateSpaceInput,
    SpacePatch,
};
use voxnexus_domain::{CommunityMemberRole, Space, SpaceVisibility};
use voxnexus_protocol::error_codes;
use voxnexus_protocol::{
    CreateSpaceRequest, SpaceListResponse, SpaceResponse, UpdateSpaceRequest,
};

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};

/// Create a Space in a community (owner / manage_spaces — owner until F029).
#[utoipa::path(
    post,
    path = "/api/v1/communities/{community_id}/spaces",
    operation_id = "createSpace",
    tag = "spaces",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body = CreateSpaceRequest,
    responses(
        (status = 201, description = "Space created", body = SpaceResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Community not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn create_space(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<CreateSpaceRequest>,
) -> Result<(StatusCode, Json<SpaceResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_manage_spaces(&state, community_id, user.account_id, &request_id).await?;
    let name = body.name.trim().to_owned();
    if name.is_empty() {
        return Err(validation(request_id, "Name is required."));
    }
    let space = persist_create(
        &state.pool,
        community_id,
        CreateSpaceInput {
            name,
            description: body.description.unwrap_or_default().trim().to_owned(),
            topic: body.topic.unwrap_or_default().trim().to_owned(),
            game: body.game.unwrap_or_default().trim().to_owned(),
            visibility: body.visibility.unwrap_or(SpaceVisibility::Open),
        },
    )
    .await
    .map_err(|error| map_auth(error, request_id))?;
    Ok((StatusCode::CREATED, Json(to_response(&space))))
}

/// List Spaces in a community (members).
#[utoipa::path(
    get,
    path = "/api/v1/communities/{community_id}/spaces",
    operation_id = "listSpaces",
    tag = "spaces",
    params(("community_id" = Uuid, Path, description = "Community id")),
    responses(
        (status = 200, description = "Space list", body = SpaceListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not a member", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Community not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn list_spaces(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
) -> Result<Json<SpaceListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_member(&state, community_id, user.account_id, &request_id).await?;
    let spaces = persist_list(&state.pool, community_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list spaces failed");
            internal(request_id)
        })?;
    Ok(Json(SpaceListResponse {
        spaces: spaces.iter().map(to_response).collect(),
    }))
}

/// Get one Space (community members).
#[utoipa::path(
    get,
    path = "/api/v1/spaces/{space_id}",
    operation_id = "getSpace",
    tag = "spaces",
    params(("space_id" = Uuid, Path, description = "Space id")),
    responses(
        (status = 200, description = "Space", body = SpaceResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not a member", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_space_by_id(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(space_id): Path<Uuid>,
) -> Result<Json<SpaceResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let space = get_space(&state.pool, space_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "get space failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| not_found(request_id.clone()))?;
    require_member(&state, space.community_id, user.account_id, &request_id).await?;
    Ok(Json(to_response(&space)))
}

/// Update a Space (owner until F029).
#[utoipa::path(
    patch,
    path = "/api/v1/spaces/{space_id}",
    operation_id = "updateSpace",
    tag = "spaces",
    params(("space_id" = Uuid, Path, description = "Space id")),
    request_body = UpdateSpaceRequest,
    responses(
        (status = 200, description = "Updated space", body = SpaceResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn update_space(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(space_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateSpaceRequest>,
) -> Result<Json<SpaceResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = get_space(&state.pool, space_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "get space failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| not_found(request_id.clone()))?;
    require_manage_spaces(&state, current.community_id, user.account_id, &request_id).await?;
    if body.name.is_none()
        && body.description.is_none()
        && body.topic.is_none()
        && body.game.is_none()
        && body.visibility.is_none()
        && body.position.is_none()
    {
        return Err(validation(request_id, "Provide at least one field to update."));
    }
    let space = persist_update(
        &state.pool,
        space_id,
        SpacePatch {
            name: body.name.map(|value| value.trim().to_owned()),
            description: body.description.map(|value| value.trim().to_owned()),
            topic: body.topic.map(|value| value.trim().to_owned()),
            game: body.game.map(|value| value.trim().to_owned()),
            visibility: body.visibility,
            position: body.position,
        },
    )
    .await
    .map_err(|error| map_auth(error, request_id))?;
    Ok(Json(to_response(&space)))
}

/// Delete a Space (owner until F029).
#[utoipa::path(
    delete,
    path = "/api/v1/spaces/{space_id}",
    operation_id = "deleteSpace",
    tag = "spaces",
    params(("space_id" = Uuid, Path, description = "Space id")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn delete_space(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(space_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = get_space(&state.pool, space_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "get space failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| not_found(request_id.clone()))?;
    require_manage_spaces(&state, current.community_id, user.account_id, &request_id).await?;
    let deleted = persist_delete(&state.pool, space_id)
        .await
        .map_err(|error| map_auth(error, request_id.clone()))?;
    if !deleted {
        return Err(not_found(request_id));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn require_member(
    state: &AppState,
    community_id: Uuid,
    account_id: Uuid,
    request_id: &str,
) -> Result<(), ApiError> {
    let membership = get_membership(&state.pool, community_id, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "membership lookup failed");
            internal(request_id.to_owned())
        })?;
    if membership.is_some() {
        return Ok(());
    }
    if get_community(&state.pool, community_id)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        Err(not_found(request_id.to_owned()))
    } else {
        Err(ApiError::permission_denied(
            request_id.to_owned(),
            "You must be a community member to view spaces.",
        ))
    }
}

async fn require_manage_spaces(
    state: &AppState,
    community_id: Uuid,
    account_id: Uuid,
    request_id: &str,
) -> Result<(), ApiError> {
    // Until F029 role permissions, owner stands in for `community.manage_spaces`.
    let membership = get_membership(&state.pool, community_id, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "membership lookup failed");
            internal(request_id.to_owned())
        })?;
    match membership {
        Some(member) if member.role == CommunityMemberRole::Owner => Ok(()),
        Some(_) => Err(ApiError::permission_denied(
            request_id.to_owned(),
            "Only the community owner can manage spaces.",
        )),
        None => {
            if get_community(&state.pool, community_id)
                .await
                .ok()
                .flatten()
                .is_none()
            {
                Err(not_found(request_id.to_owned()))
            } else {
                Err(ApiError::permission_denied(
                    request_id.to_owned(),
                    "Only the community owner can manage spaces.",
                ))
            }
        }
    }
}

fn to_response(space: &Space) -> SpaceResponse {
    SpaceResponse {
        id: space.id,
        community_id: space.community_id,
        name: space.name.clone(),
        description: space.description.clone(),
        topic: space.topic.clone(),
        game: space.game.clone(),
        visibility: space.visibility,
        icon_url: None,
        position: space.position,
        created_at: space.created_at,
        updated_at: space.updated_at,
    }
}

fn map_auth(error: voxnexus_auth::AuthError, request_id: String) -> ApiError {
    tracing::error!(error = %error, "space auth error");
    internal(request_id)
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
