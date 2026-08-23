//! Channel CRUD, archive, and clone (F027).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde_json::json;
use uuid::Uuid;
use voxnexus_auth::{
    archive_channel as persist_archive, can_view_space, clone_channel as persist_clone,
    create_channel as persist_create, delete_channel as persist_delete, get_channel, get_community,
    get_membership, get_space, list_channels as persist_list, restore_channel as persist_restore,
    update_channel as persist_update, ChannelPatch, CreateChannelInput,
};
use voxnexus_domain::{Channel, CommunityMemberRole};
use voxnexus_protocol::error_codes;
use voxnexus_protocol::{
    ChannelListResponse, ChannelResponse, CreateChannelRequest, ListChannelsQuery,
    ReorderChannelsRequest, UpdateChannelRequest,
};

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};

/// Create a channel (owner / manage_channels until F029).
#[utoipa::path(
    post,
    path = "/api/v1/communities/{community_id}/channels",
    operation_id = "createChannel",
    tag = "channels",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body = CreateChannelRequest,
    responses(
        (status = 201, description = "Channel created", body = ChannelResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn create_channel(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<CreateChannelRequest>,
) -> Result<(StatusCode, Json<ChannelResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_manage_channels(&state, community_id, user.account_id, &request_id).await?;
    let name = body.name.trim().to_owned();
    if name.is_empty() {
        return Err(validation(request_id, "Name is required."));
    }
    if let Some(space_id) = body.space_id {
        let space = get_space(&state.pool, space_id)
            .await
            .map_err(|error| map_auth(&error, request_id.clone()))?
            .ok_or_else(|| not_found(request_id.clone()))?;
        require_space_visible(&state, &space, user.account_id, &request_id).await?;
    }
    let topic = body.topic.unwrap_or_default().trim().to_owned();
    let config = body.config.unwrap_or_else(|| json!({}));
    let channel = persist_create(
        &state.pool,
        community_id,
        CreateChannelInput {
            name,
            channel_type: body.channel_type,
            topic,
            space_id: body.space_id,
            category_id: body.category_id,
            config,
        },
    )
    .await
    .map_err(|error| map_auth(&error, request_id))?;
    Ok((StatusCode::CREATED, Json(to_response(&channel))))
}

/// List channels in a community scope.
#[utoipa::path(
    get,
    path = "/api/v1/communities/{community_id}/channels",
    operation_id = "listChannels",
    tag = "channels",
    params(
        ("community_id" = Uuid, Path, description = "Community id"),
        ListChannelsQuery
    ),
    responses(
        (status = 200, description = "Channel list", body = ChannelListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not a member", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn list_channels(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    Query(query): Query<ListChannelsQuery>,
) -> Result<Json<ChannelListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_member(&state, community_id, user.account_id, &request_id).await?;
    if let Some(space_id) = query.space_id {
        let space = get_space(&state.pool, space_id)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "get space failed");
                internal(request_id.clone())
            })?
            .ok_or_else(|| not_found(request_id.clone()))?;
        if space.community_id != community_id {
            return Err(not_found(request_id));
        }
        require_space_visible(&state, &space, user.account_id, &request_id).await?;
    }
    let include_archived = query.include_archived.unwrap_or(false);
    let channels = persist_list(
        &state.pool,
        community_id,
        query.space_id,
        query.category_id,
        include_archived,
    )
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "list channels failed");
        internal(request_id)
    })?;
    Ok(Json(ChannelListResponse {
        channels: channels.iter().map(to_response).collect(),
    }))
}

/// Reorder channels in a scope by id list (owner).
#[utoipa::path(
    post,
    path = "/api/v1/communities/{community_id}/channels/reorder",
    operation_id = "reorderChannels",
    tag = "channels",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body = ReorderChannelsRequest,
    responses(
        (status = 200, description = "Reordered", body = ChannelListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody),
        (status = 422, description = "Validation error", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn reorder_channels(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<ReorderChannelsRequest>,
) -> Result<Json<ChannelListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_manage_channels(&state, community_id, user.account_id, &request_id).await?;
    let first = get_channel(&state.pool, body.channel_ids[0])
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    if first.community_id != community_id {
        return Err(not_found(request_id));
    }
    let scope_space_id = first.space_id;
    let scope_category_id = first.category_id;
    for (index, channel_id) in body.channel_ids.iter().enumerate() {
        let current = get_channel(&state.pool, *channel_id)
            .await
            .map_err(|error| map_auth(&error, request_id.clone()))?
            .ok_or_else(|| not_found(request_id.clone()))?;
        if current.community_id != community_id
            || current.space_id != scope_space_id
            || current.category_id != scope_category_id
        {
            return Err(validation(
                request_id.clone(),
                "All channels must belong to the same scope.",
            ));
        }
        let position = i32::try_from(index)
            .map_err(|_| validation(request_id.clone(), "Channel order index is out of range."))?;
        persist_update(
            &state.pool,
            *channel_id,
            ChannelPatch {
                position: Some(position),
                ..ChannelPatch::default()
            },
        )
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    }
    let channels = persist_list(
        &state.pool,
        community_id,
        scope_space_id,
        scope_category_id,
        false,
    )
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "list channels failed");
        internal(request_id)
    })?;
    Ok(Json(ChannelListResponse {
        channels: channels.iter().map(to_response).collect(),
    }))
}

/// Get one channel.
#[utoipa::path(
    get,
    path = "/api/v1/channels/{channel_id}",
    operation_id = "getChannel",
    tag = "channels",
    params(("channel_id" = Uuid, Path, description = "Channel id")),
    responses(
        (status = 200, description = "Channel", body = ChannelResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not a member", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_channel_by_id(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<ChannelResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let channel = get_channel(&state.pool, channel_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "get channel failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| not_found(request_id.clone()))?;
    require_channel_visible(&state, &channel, user.account_id, &request_id).await?;
    Ok(Json(to_response(&channel)))
}

/// Update a channel (owner until F029).
#[utoipa::path(
    patch,
    path = "/api/v1/channels/{channel_id}",
    operation_id = "updateChannel",
    tag = "channels",
    params(("channel_id" = Uuid, Path, description = "Channel id")),
    request_body = UpdateChannelRequest,
    responses(
        (status = 200, description = "Updated channel", body = ChannelResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn update_channel(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(channel_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateChannelRequest>,
) -> Result<Json<ChannelResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = get_channel(&state.pool, channel_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    require_manage_channels(&state, current.community_id, user.account_id, &request_id).await?;
    if body.name.is_none()
        && body.topic.is_none()
        && body.position.is_none()
        && body.space_id.is_none()
        && body.category_id.is_none()
        && body.config.is_none()
    {
        return Err(validation(
            request_id,
            "Provide at least one field to update.",
        ));
    }
    let channel = persist_update(
        &state.pool,
        channel_id,
        ChannelPatch {
            name: body.name.map(|value| value.trim().to_owned()),
            topic: body.topic.map(|value| value.trim().to_owned()),
            position: body.position,
            space_id: body.space_id,
            category_id: body.category_id,
            config: body.config,
        },
    )
    .await
    .map_err(|error| map_auth(&error, request_id))?;
    Ok(Json(to_response(&channel)))
}

/// Delete a channel (owner until F029).
#[utoipa::path(
    delete,
    path = "/api/v1/channels/{channel_id}",
    operation_id = "deleteChannel",
    tag = "channels",
    params(("channel_id" = Uuid, Path, description = "Channel id")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn delete_channel(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(channel_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = get_channel(&state.pool, channel_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    require_manage_channels(&state, current.community_id, user.account_id, &request_id).await?;
    let deleted = persist_delete(&state.pool, channel_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    if !deleted {
        return Err(not_found(request_id));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Archive a channel (hide from default lists).
#[utoipa::path(
    post,
    path = "/api/v1/channels/{channel_id}/archive",
    operation_id = "archiveChannel",
    tag = "channels",
    params(("channel_id" = Uuid, Path, description = "Channel id")),
    responses(
        (status = 200, description = "Archived channel", body = ChannelResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn archive_channel(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<ChannelResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = get_channel(&state.pool, channel_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    require_manage_channels(&state, current.community_id, user.account_id, &request_id).await?;
    let channel = persist_archive(&state.pool, channel_id)
        .await
        .map_err(|error| map_auth(&error, request_id))?;
    Ok(Json(to_response(&channel)))
}

/// Restore an archived channel.
#[utoipa::path(
    post,
    path = "/api/v1/channels/{channel_id}/restore",
    operation_id = "restoreChannel",
    tag = "channels",
    params(("channel_id" = Uuid, Path, description = "Channel id")),
    responses(
        (status = 200, description = "Restored channel", body = ChannelResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn restore_channel(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<ChannelResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = get_channel(&state.pool, channel_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    require_manage_channels(&state, current.community_id, user.account_id, &request_id).await?;
    let channel = persist_restore(&state.pool, channel_id)
        .await
        .map_err(|error| map_auth(&error, request_id))?;
    Ok(Json(to_response(&channel)))
}

/// Clone channel shell (no messages).
#[utoipa::path(
    post,
    path = "/api/v1/channels/{channel_id}/clone",
    operation_id = "cloneChannel",
    tag = "channels",
    params(("channel_id" = Uuid, Path, description = "Channel id")),
    responses(
        (status = 201, description = "Cloned channel", body = ChannelResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn clone_channel(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(channel_id): Path<Uuid>,
) -> Result<(StatusCode, Json<ChannelResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = get_channel(&state.pool, channel_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    require_manage_channels(&state, current.community_id, user.account_id, &request_id).await?;
    let channel = persist_clone(&state.pool, channel_id)
        .await
        .map_err(|error| map_auth(&error, request_id))?;
    Ok((StatusCode::CREATED, Json(to_response(&channel))))
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
            "You must be a community member to view channels.",
        ))
    }
}

async fn require_manage_channels(
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
    match membership {
        Some(member) if member.role == CommunityMemberRole::Owner => Ok(()),
        Some(_) => Err(ApiError::permission_denied(
            request_id.to_owned(),
            "Only the community owner can manage channels.",
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
                    "Only the community owner can manage channels.",
                ))
            }
        }
    }
}

async fn require_space_visible(
    state: &AppState,
    space: &voxnexus_domain::Space,
    account_id: Uuid,
    request_id: &str,
) -> Result<(), ApiError> {
    let visible = can_view_space(&state.pool, space, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "space visibility check failed");
            internal(request_id.to_owned())
        })?;
    if !visible {
        return Err(not_found(request_id.to_owned()));
    }
    Ok(())
}

async fn require_channel_visible(
    state: &AppState,
    channel: &Channel,
    account_id: Uuid,
    request_id: &str,
) -> Result<(), ApiError> {
    require_member(state, channel.community_id, account_id, request_id).await?;
    if let Some(space_id) = channel.space_id {
        let space = get_space(&state.pool, space_id)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "get space failed");
                internal(request_id.to_owned())
            })?
            .ok_or_else(|| not_found(request_id.to_owned()))?;
        require_space_visible(state, &space, account_id, request_id).await?;
    }
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

fn map_auth(error: &voxnexus_auth::AuthError, request_id: String) -> ApiError {
    match error {
        voxnexus_auth::AuthError::ChannelScopeMismatch => ApiError::permission_denied(
            request_id,
            "Channel cannot be moved to a scope outside this community.",
        ),
        other => {
            tracing::error!(error = %other, "channel auth error");
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
