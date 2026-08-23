//! Community create, list, and settings (F019).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use voxnexus_auth::{
    create_community as persist_community, delete_object_meta, get_community, get_instance,
    get_membership, get_object, insert_object, list_communities_for_account, set_community_banner,
    set_community_icon, slugify, unique_slug, update_community, CommunityPatch,
    CreateCommunityInput,
};
use voxnexus_domain::{Community, CommunityMemberRole, JoinMode};
use voxnexus_media::{sniff_image, AVATAR_MAX_BYTES, BANNER_MAX_BYTES};
use voxnexus_protocol::error_codes;
use voxnexus_protocol::{
    CommunityListResponse, CommunityResponse, CreateCommunityRequest, UpdateCommunityRequest,
};
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
            join_mode: body.join_mode.unwrap_or(JoinMode::Open),
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
    let community = update_community(
        &state.pool,
        community_id,
        CommunityPatch {
            name: body.name.map(|value| value.trim().to_owned()),
            description: body.description.map(|value| value.trim().to_owned()),
            timezone: body.timezone.map(|value| value.trim().to_owned()),
            join_mode: body.join_mode,
            discoverable_on_instance: body.discoverable_on_instance,
        },
    )
    .await
    .map_err(|error| map_auth(error, request_id))?;
    Ok(Json(to_response(&community)))
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

#[derive(Clone, Copy)]
enum ImageSlot {
    Icon,
    Banner,
}

impl ImageSlot {
    fn max_bytes(self) -> usize {
        match self {
            Self::Icon => AVATAR_MAX_BYTES,
            Self::Banner => BANNER_MAX_BYTES,
        }
    }

    fn folder(self) -> &'static str {
        match self {
            Self::Icon => "community-icons",
            Self::Banner => "community-banners",
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
        discoverable_on_instance: community.discoverable_on_instance,
        created_at: community.created_at,
        updated_at: community.updated_at,
    }
}

fn map_auth(error: voxnexus_auth::AuthError, request_id: String) -> ApiError {
    match error {
        voxnexus_auth::AuthError::SlugTaken => ApiError::new(
            StatusCode::CONFLICT,
            error_codes::VALIDATION_ERROR,
            "Community slug is already taken.",
            None,
            request_id,
        ),
        other => {
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
