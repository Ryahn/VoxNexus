//! Community create, list, and settings (F019).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use voxnexus_auth::{
    create_community as persist_community, delete_community as persist_delete, delete_object_meta,
    get_community, get_instance, get_membership, get_object, get_profile, insert_object,
    join_community as persist_join, leave_community as persist_leave, list_communities_for_account,
    list_member_account_ids, list_members, set_community_banner, set_community_icon,
    set_community_invite_splash, set_community_tag_badge, set_nickname,
    slugify, transfer_community as persist_transfer, unique_slug, update_community, CommunityPatch,
    CreateCommunityInput, MemberListItem,
};
use voxnexus_domain::{Community, CommunityMemberRole, JoinMode};
use voxnexus_media::{sniff_image, AVATAR_MAX_BYTES, BANNER_MAX_BYTES};
use voxnexus_protocol::error_codes;
use voxnexus_protocol::{
    CommunityListResponse, CommunityMemberListResponse, CommunityMemberResponse, CommunityResponse,
    CreateCommunityRequest, CursorQuery, DeleteCommunityRequest, MemberJoinPayload,
    MemberLeavePayload, TransferCommunityRequest, UpdateCommunityRequest, UpdateNicknameRequest,
};
use voxnexus_realtime::PresenceHubMessage;
use voxnexus_storage::ObjectKey;

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};

/// Create a community when instance policy allows.
#[utoipa::path(
    post,
    path = "/api/v1/communities",
    operation_id = "createCommunity",
    tag = "communities",
    request_body = CreateCommunityRequest,
    responses(
        (status = 201, description = "Community created", body = CommunityResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Creation disallowed by instance policy", body = voxnexus_protocol::ErrorBody),
        (status = 409, description = "Slug conflict", body = voxnexus_protocol::ErrorBody),
        (status = 422, description = "Validation error", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn create_community(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    ValidatedJson(body): ValidatedJson<CreateCommunityRequest>,
) -> Result<(StatusCode, Json<CommunityResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    let instance = get_instance(&state.pool).await.map_err(|error| {
        tracing::error!(error = %error, "instance lookup failed");
        internal(request_id.clone())
    })?;
    if !instance
        .community_creation_mode
        .user_can_create_community(user.is_instance_admin)
    {
        return Err(ApiError::permission_denied(
            request_id,
            "Community creation is not allowed on this instance.",
        ));
    }

    let name = body.name.trim().to_owned();
    if name.is_empty() {
        return Err(validation(request_id, "Name is required."));
    }
    let base_slug = body
        .slug
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(|| slugify(&name), slugify);
    let slug = unique_slug(&state.pool, &base_slug)
        .await
        .map_err(|error| map_auth(error, request_id.clone()))?;

    let join_mode = body.join_mode.unwrap_or(JoinMode::Open);
    if join_mode == JoinMode::Application {
        return Err(validation(
            request_id,
            "Application join mode is not available yet.",
        ));
    }

    let community = persist_community(
        &state.pool,
        user.account_id,
        CreateCommunityInput {
            name,
            slug,
            description: body.description.unwrap_or_default().trim().to_owned(),
            timezone: body
                .timezone
                .unwrap_or_else(|| "UTC".to_owned())
                .trim()
                .to_owned(),
            join_mode,
            discoverable_on_instance: body.discoverable_on_instance.unwrap_or(true),
        },
    )
    .await
    .map_err(|error| map_auth(error, request_id.clone()))?;

    Ok((StatusCode::CREATED, Json(to_response(&community))))
}

/// List communities the caller belongs to.
#[utoipa::path(
    get,
    path = "/api/v1/communities",
    operation_id = "listCommunities",
    tag = "communities",
    responses(
        (status = 200, description = "Membership communities", body = CommunityListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn list_communities(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
) -> Result<Json<CommunityListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let communities = list_communities_for_account(&state.pool, user.account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list communities failed");
            internal(request_id)
        })?;
    Ok(Json(CommunityListResponse {
        communities: communities.iter().map(to_response).collect(),
    }))
}

/// Get one community (members only).
#[utoipa::path(
    get,
    path = "/api/v1/communities/{community_id}",
    operation_id = "getCommunity",
    tag = "communities",
    params(("community_id" = Uuid, Path, description = "Community id")),
    responses(
        (status = 200, description = "Community", body = CommunityResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not a member", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_community_by_id(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
) -> Result<Json<CommunityResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_member(&state, community_id, user.account_id, &request_id).await?;
    let community = get_community(&state.pool, community_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "get community failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| not_found(request_id))?;
    Ok(Json(to_response(&community)))
}

/// Update community settings (owner only).
#[utoipa::path(
    patch,
    path = "/api/v1/communities/{community_id}",
    operation_id = "updateCommunity",
    tag = "communities",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body = UpdateCommunityRequest,
    responses(
        (status = 200, description = "Updated community", body = CommunityResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not the owner", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody),
        (status = 422, description = "Validation error", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn update_community_settings(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateCommunityRequest>,
) -> Result<Json<CommunityResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_owner(&state, community_id, user.account_id, &request_id).await?;
    if body.join_mode == Some(JoinMode::Application) {
        return Err(validation(
            request_id,
            "Application join mode is not available yet.",
        ));
    }
    let community = update_community(
        &state.pool,
        community_id,
        CommunityPatch {
            name: body.name.map(|value| value.trim().to_owned()),
            description: body.description.map(|value| value.trim().to_owned()),
            timezone: body.timezone.map(|value| value.trim().to_owned()),
            join_mode: body.join_mode,
            discoverable_on_instance: body.discoverable_on_instance,
            tag_name: body.tag_name.map(|value| value.trim().to_owned()),
            tag_color: body.tag_color.map(|value| value.trim().to_owned()),
            invite_path: body.invite_path.as_ref().map(|value| {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_owned())
                }
            }),
        },
    )
    .await
    .map_err(|error| map_auth(error, request_id))?;
    Ok(Json(to_response(&community)))
}

/// Transfer community ownership to an existing member (owner only).
#[utoipa::path(
    post,
    path = "/api/v1/communities/{community_id}/transfer",
    operation_id = "transferCommunity",
    tag = "communities",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body = TransferCommunityRequest,
    responses(
        (status = 200, description = "Ownership transferred", body = CommunityResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not the owner", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn transfer_community(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<TransferCommunityRequest>,
) -> Result<Json<CommunityResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_owner(&state, community_id, user.account_id, &request_id).await?;
    let community = persist_transfer(&state.pool, community_id, user.account_id, body.account_id)
        .await
        .map_err(|error| map_membership_auth(error, request_id))?;
    tracing::info!(
        community_id = %community_id,
        from = %user.account_id,
        to = %body.account_id,
        "community ownership transferred"
    );
    Ok(Json(to_response(&community)))
}

/// Delete a community (owner only, typed name confirm).
#[utoipa::path(
    post,
    path = "/api/v1/communities/{community_id}/delete",
    operation_id = "deleteCommunity",
    tag = "communities",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body = DeleteCommunityRequest,
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not the owner", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody),
        (status = 422, description = "Validation error", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn delete_community(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<DeleteCommunityRequest>,
) -> Result<StatusCode, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_owner(&state, community_id, user.account_id, &request_id).await?;
    let community = get_community(&state.pool, community_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "get community failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| not_found(request_id.clone()))?;
    let confirm = body.confirm_name.trim();
    if confirm != community.name {
        return Err(validation(
            request_id,
            "Type the community name exactly to confirm deletion.",
        ));
    }
    persist_delete(&state.pool, community_id, user.account_id)
        .await
        .map_err(|error| map_membership_auth(error, request_id.clone()))?;
    tracing::info!(
        community_id = %community_id,
        owner_account_id = %user.account_id,
        "community deleted"
    );
    Ok(StatusCode::NO_CONTENT)
}

/// Upload community icon (owner).
#[utoipa::path(
    put,
    path = "/api/v1/communities/{community_id}/icon",
    operation_id = "uploadCommunityIcon",
    tag = "communities",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body(content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "Updated community", body = CommunityResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not the owner", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn upload_community_icon(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    body: Bytes,
) -> Result<Json<CommunityResponse>, ApiError> {
    upload_image(&state, user, headers, community_id, body, ImageSlot::Icon).await
}

/// Upload community banner (owner).
#[utoipa::path(
    put,
    path = "/api/v1/communities/{community_id}/banner",
    operation_id = "uploadCommunityBanner",
    tag = "communities",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body(content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "Updated community", body = CommunityResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not the owner", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn upload_community_banner(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    body: Bytes,
) -> Result<Json<CommunityResponse>, ApiError> {
    upload_image(&state, user, headers, community_id, body, ImageSlot::Banner).await
}

/// Upload community tag badge (owner).
#[utoipa::path(
    put,
    path = "/api/v1/communities/{community_id}/tag-badge",
    operation_id = "uploadCommunityTagBadge",
    tag = "communities",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body(content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "Updated community", body = CommunityResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not the owner", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn upload_community_tag_badge(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    body: Bytes,
) -> Result<Json<CommunityResponse>, ApiError> {
    upload_image(&state, user, headers, community_id, body, ImageSlot::TagBadge).await
}

/// Upload community invite splash (owner).
#[utoipa::path(
    put,
    path = "/api/v1/communities/{community_id}/invite-splash",
    operation_id = "uploadCommunityInviteSplash",
    tag = "communities",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body(content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "Updated community", body = CommunityResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not the owner", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn upload_community_invite_splash(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    body: Bytes,
) -> Result<Json<CommunityResponse>, ApiError> {
    upload_image(&state, user, headers, community_id, body, ImageSlot::InviteSplash).await
}

/// Serve community icon bytes.
#[utoipa::path(
    get,
    path = "/api/v1/communities/{community_id}/icon",
    operation_id = "getCommunityIcon",
    tag = "communities",
    params(("community_id" = Uuid, Path, description = "Community id")),
    responses(
        (status = 200, description = "Image bytes", content_type = "application/octet-stream"),
        (status = 404, description = "No icon", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_community_icon(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    serve_image(&state, headers, community_id, ImageSlot::Icon).await
}

/// Serve community banner bytes.
#[utoipa::path(
    get,
    path = "/api/v1/communities/{community_id}/banner",
    operation_id = "getCommunityBanner",
    tag = "communities",
    params(("community_id" = Uuid, Path, description = "Community id")),
    responses(
        (status = 200, description = "Image bytes", content_type = "application/octet-stream"),
        (status = 404, description = "No banner", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_community_banner(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    serve_image(&state, headers, community_id, ImageSlot::Banner).await
}

/// Serve community tag badge bytes.
#[utoipa::path(
    get,
    path = "/api/v1/communities/{community_id}/tag-badge",
    operation_id = "getCommunityTagBadge",
    tag = "communities",
    params(("community_id" = Uuid, Path, description = "Community id")),
    responses(
        (status = 200, description = "Image bytes", content_type = "application/octet-stream"),
        (status = 404, description = "No badge", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_community_tag_badge(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    serve_image(&state, headers, community_id, ImageSlot::TagBadge).await
}

/// Serve community invite splash bytes.
#[utoipa::path(
    get,
    path = "/api/v1/communities/{community_id}/invite-splash",
    operation_id = "getCommunityInviteSplash",
    tag = "communities",
    params(("community_id" = Uuid, Path, description = "Community id")),
    responses(
        (status = 200, description = "Image bytes", content_type = "application/octet-stream"),
        (status = 404, description = "No splash", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_community_invite_splash(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    serve_image(&state, headers, community_id, ImageSlot::InviteSplash).await
}

/// Join an open community.
#[utoipa::path(
    post,
    path = "/api/v1/communities/{community_id}/join",
    operation_id = "joinCommunity",
    tag = "communities",
    params(("community_id" = Uuid, Path, description = "Community id")),
    responses(
        (status = 201, description = "Joined", body = CommunityMemberResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Join not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody),
        (status = 409, description = "Already a member", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn join_community(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
) -> Result<(StatusCode, Json<CommunityMemberResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    let member = persist_join(&state.pool, community_id, user.account_id)
        .await
        .map_err(|error| map_membership_auth(error, request_id.clone()))?;
    let profile = get_profile(&state.pool, user.account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "profile lookup failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| internal(request_id.clone()))?;

    let response = member_response(&MemberListItem {
        member: member.clone(),
        display_name: profile.display_name.clone(),
        has_avatar: profile.avatar_object_id.is_some(),
    });

    let mut recipients = list_member_account_ids(&state.pool, community_id)
        .await
        .unwrap_or_default();
    recipients.push(user.account_id);
    recipients.sort_unstable();
    recipients.dedup();
    state
        .presence_hub
        .broadcast_to_accounts(
            &recipients,
            PresenceHubMessage::MemberJoin(MemberJoinPayload {
                community_id,
                account_id: user.account_id,
                role: member.role.as_str().to_owned(),
                nickname: member.nickname.clone(),
                display_name: profile.display_name,
            }),
        )
        .await;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Leave a community (owners cannot leave until transfer exists).
#[utoipa::path(
    post,
    path = "/api/v1/communities/{community_id}/leave",
    operation_id = "leaveCommunity",
    tag = "communities",
    params(("community_id" = Uuid, Path, description = "Community id")),
    responses(
        (status = 204, description = "Left"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Owner cannot leave", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not a member / not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn leave_community(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let recipients = list_member_account_ids(&state.pool, community_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list members for leave fanout failed");
            internal(request_id.clone())
        })?;
    persist_leave(&state.pool, community_id, user.account_id)
        .await
        .map_err(|error| map_membership_auth(error, request_id))?;

    state
        .presence_hub
        .broadcast_to_accounts(
            &recipients,
            PresenceHubMessage::MemberLeave(MemberLeavePayload {
                community_id,
                account_id: user.account_id,
            }),
        )
        .await;

    Ok(StatusCode::NO_CONTENT)
}

/// List community members (members only).
#[utoipa::path(
    get,
    path = "/api/v1/communities/{community_id}/members",
    operation_id = "listCommunityMembers",
    tag = "communities",
    params(
        ("community_id" = Uuid, Path, description = "Community id"),
        ("after" = Option<Uuid>, Query, description = "Cursor: members after this account id"),
        ("before" = Option<Uuid>, Query, description = "Cursor: members before this account id"),
        ("limit" = Option<u16>, Query, description = "Page size (1-100)")
    ),
    responses(
        (status = 200, description = "Members page", body = CommunityMemberListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not a member", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn list_community_members(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    Query(query): Query<CursorQuery>,
) -> Result<Json<CommunityMemberListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_member(&state, community_id, user.account_id, &request_id).await?;
    let page = list_members(
        &state.pool,
        community_id,
        query.after,
        query.before,
        query.resolved_limit(),
    )
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "list members failed");
        internal(request_id)
    })?;
    Ok(Json(CommunityMemberListResponse {
        items: page.items.iter().map(member_response).collect(),
        has_more: page.has_more,
    }))
}

/// Update the caller's nickname in a community.
#[utoipa::path(
    patch,
    path = "/api/v1/communities/{community_id}/members/me",
    operation_id = "updateMyCommunityNickname",
    tag = "communities",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body = UpdateNicknameRequest,
    responses(
        (status = 200, description = "Updated membership", body = CommunityMemberResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not a member", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody),
        (status = 422, description = "Validation error", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn update_my_nickname(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateNicknameRequest>,
) -> Result<Json<CommunityMemberResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_member(&state, community_id, user.account_id, &request_id).await?;
    let nickname = body.nickname.trim().to_owned();
    let member = set_nickname(&state.pool, community_id, user.account_id, &nickname)
        .await
        .map_err(|error| map_membership_auth(error, request_id.clone()))?;
    let profile = get_profile(&state.pool, user.account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "profile lookup failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| internal(request_id))?;
    Ok(Json(member_response(&MemberListItem {
        member,
        display_name: profile.display_name,
        has_avatar: profile.avatar_object_id.is_some(),
    })))
}

#[derive(Clone, Copy)]
enum ImageSlot {
    Icon,
    Banner,
    TagBadge,
    InviteSplash,
}

impl ImageSlot {
    fn max_bytes(self) -> usize {
        match self {
            Self::Icon | Self::TagBadge => AVATAR_MAX_BYTES,
            Self::Banner | Self::InviteSplash => BANNER_MAX_BYTES,
        }
    }

    fn folder(self) -> &'static str {
        match self {
            Self::Icon => "community-icons",
            Self::Banner => "community-banners",
            Self::TagBadge => "community-tag-badges",
            Self::InviteSplash => "community-invite-splashes",
        }
    }
}

async fn upload_image(
    state: &AppState,
    user: AuthUser,
    headers: HeaderMap,
    community_id: Uuid,
    body: Bytes,
    slot: ImageSlot,
) -> Result<Json<CommunityResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_owner(state, community_id, user.account_id, &request_id).await?;
    if body.len() > slot.max_bytes() {
        return Err(validation(
            request_id,
            format!("Image exceeds maximum size of {} bytes.", slot.max_bytes()),
        ));
    }
    let Some(kind) = sniff_image(&body) else {
        return Err(validation(
            request_id,
            "Image must be JPEG, PNG, GIF, or WebP.",
        ));
    };
    let object_id = Uuid::now_v7();
    let key_str = format!(
        "vn/{}/{community_id}/{object_id}.{}",
        slot.folder(),
        kind.extension()
    );
    let key = ObjectKey::parse(&key_str).map_err(|_| internal(request_id.clone()))?;
    let digest = Sha256::digest(&body);
    state
        .storage
        .put(key, body.clone(), kind.mime())
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "community image put failed");
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::INTERNAL,
                "Failed to store image.",
                None,
                request_id.clone(),
            )
        })?;
    insert_object(
        &state.pool,
        object_id,
        &key_str,
        digest.as_slice(),
        kind.mime(),
        i64::try_from(body.len()).unwrap_or(i64::MAX),
        user.account_id,
    )
    .await
    .map_err(|error| map_auth(error, request_id.clone()))?;

    let previous = match slot {
        ImageSlot::Icon => set_community_icon(&state.pool, community_id, object_id).await,
        ImageSlot::Banner => set_community_banner(&state.pool, community_id, object_id).await,
        ImageSlot::TagBadge => set_community_tag_badge(&state.pool, community_id, object_id).await,
        ImageSlot::InviteSplash => {
            set_community_invite_splash(&state.pool, community_id, object_id).await
        }
    }
    .map_err(|error| map_auth(error, request_id.clone()))?;

    if let Some(old_id) = previous {
        if let Ok(Some(old)) = get_object(&state.pool, old_id).await {
            if let Ok(old_key) = ObjectKey::parse(&old.storage_key) {
                let _ = state.storage.delete(&old_key).await;
            }
            let _ = delete_object_meta(&state.pool, old_id).await;
        }
    }

    let community = get_community(&state.pool, community_id)
        .await
        .map_err(|error| map_auth(error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id))?;
    Ok(Json(to_response(&community)))
}

async fn serve_image(
    state: &AppState,
    headers: HeaderMap,
    community_id: Uuid,
    slot: ImageSlot,
) -> Result<impl IntoResponse, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let community = get_community(&state.pool, community_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "get community for image failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| not_found(request_id.clone()))?;
    let object_id = match slot {
        ImageSlot::Icon => community.icon_object_id,
        ImageSlot::Banner => community.banner_object_id,
        ImageSlot::TagBadge => community.tag_badge_object_id,
        ImageSlot::InviteSplash => community.invite_splash_object_id,
    }
    .ok_or_else(|| not_found(request_id.clone()))?;
    let meta = get_object(&state.pool, object_id)
        .await
        .map_err(|error| map_auth(error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    let key = ObjectKey::parse(&meta.storage_key).map_err(|_| internal(request_id.clone()))?;
    let bytes = state.storage.get(&key).await.map_err(|error| {
        tracing::error!(error = %error, "community image get failed");
        internal(request_id)
    })?;
    Ok((
        [
            (header::CONTENT_TYPE, meta.mime),
            (header::CACHE_CONTROL, "public, max-age=3600".to_owned()),
        ],
        bytes,
    ))
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
    if membership.is_none() {
        // Distinguish missing community vs not a member.
        if get_community(&state.pool, community_id)
            .await
            .ok()
            .flatten()
            .is_none()
        {
            return Err(not_found(request_id.to_owned()));
        }
        return Err(ApiError::permission_denied(
            request_id.to_owned(),
            "You are not a member of this community.",
        ));
    }
    Ok(())
}

async fn require_owner(
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
            "Only the community owner can change these settings.",
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
                    "Only the community owner can change these settings.",
                ))
            }
        }
    }
}

fn to_response(community: &Community) -> CommunityResponse {
    CommunityResponse {
        id: community.id,
        name: community.name.clone(),
        slug: community.slug.clone(),
        description: community.description.clone(),
        timezone: community.timezone.clone(),
        join_mode: community.join_mode,
        owner_account_id: community.owner_account_id,
        icon_url: community
            .icon_object_id
            .map(|_| format!("/api/v1/communities/{}/icon", community.id)),
        banner_url: community
            .banner_object_id
            .map(|_| format!("/api/v1/communities/{}/banner", community.id)),
        tag_name: community.tag_name.clone(),
        tag_color: community.tag_color.clone(),
        tag_badge_url: community
            .tag_badge_object_id
            .map(|_| format!("/api/v1/communities/{}/tag-badge", community.id)),
        invite_splash_url: community
            .invite_splash_object_id
            .map(|_| format!("/api/v1/communities/{}/invite-splash", community.id)),
        invite_path: community.invite_path.clone(),
        discoverable_on_instance: community.discoverable_on_instance,
        created_at: community.created_at,
        updated_at: community.updated_at,
    }
}

fn member_response(item: &MemberListItem) -> CommunityMemberResponse {
    CommunityMemberResponse {
        community_id: item.member.community_id,
        account_id: item.member.account_id,
        role: item.member.role,
        nickname: item.member.nickname.clone(),
        display_name: item.display_name.clone(),
        has_avatar: item.has_avatar,
        avatar_url: item
            .has_avatar
            .then(|| format!("/api/v1/profiles/{}/avatar", item.member.account_id)),
        joined_at: item.member.joined_at,
    }
}

fn map_auth(error: voxnexus_auth::AuthError, request_id: String) -> ApiError {
    match error {
        voxnexus_auth::AuthError::SlugTaken => ApiError::new(
            StatusCode::CONFLICT,
            error_codes::VALIDATION_ERROR,
            "That slug or invite path is already taken.",
            None,
            request_id,
        ),
        other => map_membership_auth(other, request_id),
    }
}

fn map_membership_auth(error: voxnexus_auth::AuthError, request_id: String) -> ApiError {
    match error {
        voxnexus_auth::AuthError::AlreadyMember => {
            ApiError::conflict(request_id, "You are already a member of this community.")
        }
        voxnexus_auth::AuthError::JoinNotAllowed => ApiError::permission_denied(
            request_id,
            "This community is invite-only or requires an application.",
        ),
        voxnexus_auth::AuthError::OwnerCannotLeave => ApiError::permission_denied(
            request_id,
            "Community owners cannot leave without transferring ownership.",
        ),
        voxnexus_auth::AuthError::NotMember => ApiError::new(
            StatusCode::NOT_FOUND,
            error_codes::NOT_FOUND,
            "You are not a member of this community.",
            None,
            request_id,
        ),
        voxnexus_auth::AuthError::NotCommunityOwner => ApiError::permission_denied(
            request_id,
            "Only the community owner can perform this action.",
        ),
        other => {
            let message = other.to_string();
            if message.contains("no rows returned") || message.contains("RowNotFound") {
                return not_found(request_id);
            }
            tracing::error!(error = %other, "community auth error");
            internal(request_id)
        }
    }
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

fn not_found(request_id: String) -> ApiError {
    ApiError::new(
        StatusCode::NOT_FOUND,
        error_codes::NOT_FOUND,
        "Community not found.",
        None,
        request_id,
    )
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
