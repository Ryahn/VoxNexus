//! Space CRUD and membership (F022 / F023).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use uuid::Uuid;
use voxnexus_auth::{
    add_space_member as persist_add_member, can_view_space, create_space as persist_create,
    delete_space as persist_delete, get_community, get_membership, get_space, is_space_member,
    join_space as persist_join, leave_space as persist_leave,
    list_space_members as persist_list_members, list_spaces_visible_to,
    remove_space_member as persist_remove_member, update_space as persist_update, CreateSpaceInput,
    SpaceMemberListItem, SpacePatch,
};
use voxnexus_domain::{CommunityMemberRole, Space, SpaceVisibility};
use voxnexus_protocol::error_codes;
use voxnexus_protocol::{
    AddSpaceMemberRequest, CreateSpaceRequest, SpaceListResponse, SpaceMemberListResponse,
    SpaceMemberResponse, SpaceResponse, UpdateSpaceRequest,
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
        user.account_id,
        CreateSpaceInput {
            name,
            description: body.description.unwrap_or_default().trim().to_owned(),
            topic: body.topic.unwrap_or_default().trim().to_owned(),
            game: body.game.unwrap_or_default().trim().to_owned(),
            visibility: body.visibility.unwrap_or(SpaceVisibility::Open),
        },
    )
    .await
    .map_err(|error| map_auth(&error, request_id))?;
    Ok((StatusCode::CREATED, Json(to_response(&space, true))))
}

/// List Spaces visible to the caller in a community.
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
    let spaces = list_spaces_visible_to(&state.pool, community_id, user.account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list spaces failed");
            internal(request_id)
        })?;
    Ok(Json(SpaceListResponse {
        spaces: spaces
            .iter()
            .map(|(space, is_member)| to_response(space, *is_member))
            .collect(),
    }))
}

/// Get one Space (404 if restricted and not a member).
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
    let visible = can_view_space(&state.pool, &space, user.account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "space visibility check failed");
            internal(request_id.clone())
        })?;
    if !visible {
        return Err(not_found(request_id));
    }
    let member = is_space_member(&state.pool, space.id, user.account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "space membership check failed");
            internal(request_id)
        })?;
    Ok(Json(to_response(&space, member)))
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
        return Err(validation(
            request_id,
            "Provide at least one field to update.",
        ));
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
    .map_err(|error| map_auth(&error, request_id.clone()))?;
    let member = is_space_member(&state.pool, space.id, user.account_id)
        .await
        .map_err(|error| map_auth(&error, request_id))?;
    Ok(Json(to_response(&space, member)))
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
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    if !deleted {
        return Err(not_found(request_id));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Join an open space (community members). Restricted spaces require an admin add.
#[utoipa::path(
    post,
    path = "/api/v1/spaces/{space_id}/join",
    operation_id = "joinSpace",
    tag = "spaces",
    params(("space_id" = Uuid, Path, description = "Space id")),
    responses(
        (status = 201, description = "Joined", body = SpaceMemberResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Join not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody),
        (status = 409, description = "Already a member", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn join_space(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(space_id): Path<Uuid>,
) -> Result<(StatusCode, Json<SpaceMemberResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    let space = require_space(&state, space_id, &request_id).await?;
    require_member(&state, space.community_id, user.account_id, &request_id).await?;
    let member = persist_join(&state.pool, space_id, user.account_id)
        .await
        .map_err(|error| map_membership_auth(&error, request_id.clone()))?;
    Ok((
        StatusCode::CREATED,
        Json(
            member_response_simple(
                member.space_id,
                member.account_id,
                member.joined_at,
                &state,
                &request_id,
            )
            .await?,
        ),
    ))
}

/// Leave a space.
#[utoipa::path(
    post,
    path = "/api/v1/spaces/{space_id}/leave",
    operation_id = "leaveSpace",
    tag = "spaces",
    params(("space_id" = Uuid, Path, description = "Space id")),
    responses(
        (status = 204, description = "Left"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not a member or not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn leave_space(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(space_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let _space = require_space(&state, space_id, &request_id).await?;
    persist_leave(&state.pool, space_id, user.account_id)
        .await
        .map_err(|error| map_membership_auth(&error, request_id))?;
    Ok(StatusCode::NO_CONTENT)
}

/// List members of a space (space members or community owner).
#[utoipa::path(
    get,
    path = "/api/v1/spaces/{space_id}/members",
    operation_id = "listSpaceMembers",
    tag = "spaces",
    params(("space_id" = Uuid, Path, description = "Space id")),
    responses(
        (status = 200, description = "Members", body = SpaceMemberListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn list_space_members(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(space_id): Path<Uuid>,
) -> Result<Json<SpaceMemberListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let space = require_space(&state, space_id, &request_id).await?;
    require_member(&state, space.community_id, user.account_id, &request_id).await?;
    let visible = can_view_space(&state.pool, &space, user.account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "space visibility check failed");
            internal(request_id.clone())
        })?;
    if !visible {
        return Err(not_found(request_id));
    }
    let members = persist_list_members(&state.pool, space_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list space members failed");
            internal(request_id)
        })?;
    Ok(Json(SpaceMemberListResponse {
        members: members.iter().map(member_response).collect(),
    }))
}

/// Add a community member to a space (owner until F029).
#[utoipa::path(
    post,
    path = "/api/v1/spaces/{space_id}/members",
    operation_id = "addSpaceMember",
    tag = "spaces",
    params(("space_id" = Uuid, Path, description = "Space id")),
    request_body = AddSpaceMemberRequest,
    responses(
        (status = 201, description = "Member added", body = SpaceMemberResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody),
        (status = 409, description = "Already a member", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn add_space_member(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(space_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<AddSpaceMemberRequest>,
) -> Result<(StatusCode, Json<SpaceMemberResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    let space = require_space(&state, space_id, &request_id).await?;
    require_manage_spaces(&state, space.community_id, user.account_id, &request_id).await?;
    let member = persist_add_member(&state.pool, space_id, body.account_id)
        .await
        .map_err(|error| map_membership_auth(&error, request_id.clone()))?;
    Ok((
        StatusCode::CREATED,
        Json(
            member_response_simple(
                member.space_id,
                member.account_id,
                member.joined_at,
                &state,
                &request_id,
            )
            .await?,
        ),
    ))
}

/// Remove a member from a space (owner until F029).
#[utoipa::path(
    delete,
    path = "/api/v1/spaces/{space_id}/members/{account_id}",
    operation_id = "removeSpaceMember",
    tag = "spaces",
    params(
        ("space_id" = Uuid, Path, description = "Space id"),
        ("account_id" = Uuid, Path, description = "Account id")
    ),
    responses(
        (status = 204, description = "Removed"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn remove_space_member(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path((space_id, account_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let space = require_space(&state, space_id, &request_id).await?;
    require_manage_spaces(&state, space.community_id, user.account_id, &request_id).await?;
    persist_remove_member(&state.pool, space_id, account_id)
        .await
        .map_err(|error| map_membership_auth(&error, request_id))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn require_space(
    state: &AppState,
    space_id: Uuid,
    request_id: &str,
) -> Result<Space, ApiError> {
    get_space(&state.pool, space_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "get space failed");
            internal(request_id.to_owned())
        })?
        .ok_or_else(|| not_found(request_id.to_owned()))
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

fn to_response(space: &Space, is_member: bool) -> SpaceResponse {
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
        is_member,
        created_at: space.created_at,
        updated_at: space.updated_at,
    }
}

fn member_response(item: &SpaceMemberListItem) -> SpaceMemberResponse {
    SpaceMemberResponse {
        space_id: item.member.space_id,
        account_id: item.member.account_id,
        display_name: item.display_name.clone(),
        has_avatar: item.has_avatar,
        avatar_url: item
            .has_avatar
            .then(|| format!("/api/v1/profiles/{}/avatar", item.member.account_id)),
        joined_at: item.member.joined_at,
    }
}

async fn member_response_simple(
    space_id: Uuid,
    account_id: Uuid,
    joined_at: chrono::DateTime<chrono::Utc>,
    state: &AppState,
    request_id: &str,
) -> Result<SpaceMemberResponse, ApiError> {
    let profile = voxnexus_auth::get_profile(&state.pool, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "profile lookup failed");
            internal(request_id.to_owned())
        })?;
    let display_name = profile
        .as_ref()
        .map(|p| p.display_name.clone())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| account_id.to_string());
    let has_avatar = profile
        .as_ref()
        .is_some_and(|p| p.avatar_object_id.is_some());
    Ok(SpaceMemberResponse {
        space_id,
        account_id,
        display_name,
        has_avatar,
        avatar_url: has_avatar.then(|| format!("/api/v1/profiles/{account_id}/avatar")),
        joined_at,
    })
}

fn map_auth(error: &voxnexus_auth::AuthError, request_id: String) -> ApiError {
    tracing::error!(error = %error, "space auth error");
    internal(request_id)
}

fn map_membership_auth(error: &voxnexus_auth::AuthError, request_id: String) -> ApiError {
    match error {
        voxnexus_auth::AuthError::AlreadySpaceMember => {
            ApiError::conflict(request_id, "You are already a member of this space.")
        }
        voxnexus_auth::AuthError::SpaceJoinNotAllowed => ApiError::permission_denied(
            request_id,
            "This space is restricted. Ask an owner to add you.",
        ),
        voxnexus_auth::AuthError::NotSpaceMember => ApiError::new(
            StatusCode::NOT_FOUND,
            error_codes::NOT_FOUND,
            "You are not a member of this space.",
            None,
            request_id,
        ),
        voxnexus_auth::AuthError::NotMember => ApiError::new(
            StatusCode::NOT_FOUND,
            error_codes::NOT_FOUND,
            "That account is not a member of this community.",
            None,
            request_id,
        ),
        other => {
            let message = other.to_string();
            if message.contains("no rows returned") || message.contains("RowNotFound") {
                return not_found(request_id);
            }
            tracing::error!(error = %other, "space membership auth error");
            internal(request_id)
        }
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
