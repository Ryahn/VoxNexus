//! Profile read/update and avatar/banner upload/proxy (F014).

#![allow(clippy::missing_errors_doc)]

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use voxnexus_auth::{
    delete_object_meta, ensure_profile, get_object, get_profile, insert_object, set_avatar_object,
    set_banner_object, update_profile, AuthError,
};
use voxnexus_domain::Profile;
use voxnexus_media::{sniff_image, AVATAR_MAX_BYTES, BANNER_MAX_BYTES};
use voxnexus_protocol::error_codes;
use voxnexus_protocol::{ProfileResponse, UpdateProfileRequest};
use voxnexus_storage::ObjectKey;

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};

/// Current user's profile.
#[utoipa::path(
    get,
    path = "/api/v1/me/profile",
    operation_id = "getMyProfile",
    tag = "profiles",
    responses(
        (status = 200, description = "Own profile", body = ProfileResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_my_profile(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
) -> Result<Json<ProfileResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let profile = ensure_profile(&state.pool, user.account_id)
        .await
        .map_err(|error| map_db(&error, request_id))?;
    Ok(Json(to_response(&state, &profile, user.account_id).await))
}

/// Update display name and/or bio.
#[utoipa::path(
    patch,
    path = "/api/v1/me/profile",
    operation_id = "updateMyProfile",
    tag = "profiles",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Updated profile", body = ProfileResponse),
        (status = 400, description = "Validation error", body = voxnexus_protocol::ErrorBody),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn update_my_profile(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    ValidatedJson(body): ValidatedJson<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    if body.display_name.is_none()
        && body.bio.is_none()
        && body.presence_status.is_none()
        && body.custom_status.is_none()
    {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "Provide at least one field to update.",
            None,
            request_id,
        ));
    }
    let profile = update_profile(
        &state.pool,
        user.account_id,
        body.display_name.as_deref(),
        body.bio.as_deref(),
        body.presence_status,
        body.custom_status.as_deref(),
    )
    .await
    .map_err(|error| map_db(&error, request_id.clone()))?;

    if body.presence_status.is_some() || body.custom_status.is_some() {
        state
            .presence_hub
            .update(
                user.account_id,
                body.presence_status,
                body.custom_status.as_deref(),
            )
            .await;
    }

    Ok(Json(to_response(&state, &profile, user.account_id).await))
}

/// Upload avatar image (raw body; Content-Type ignored in favor of magic bytes).
#[utoipa::path(
    post,
    path = "/api/v1/me/profile/avatar",
    operation_id = "uploadMyAvatar",
    tag = "profiles",
    request_body(content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "Updated profile", body = ProfileResponse),
        (status = 400, description = "Invalid or oversized image", body = voxnexus_protocol::ErrorBody),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn upload_my_avatar(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ProfileResponse>, ApiError> {
    upload_image(&state, user.account_id, &headers, body, ImageSlot::Avatar).await
}

/// Upload banner image.
#[utoipa::path(
    post,
    path = "/api/v1/me/profile/banner",
    operation_id = "uploadMyBanner",
    tag = "profiles",
    request_body(content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "Updated profile", body = ProfileResponse),
        (status = 400, description = "Invalid or oversized image", body = voxnexus_protocol::ErrorBody),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn upload_my_banner(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ProfileResponse>, ApiError> {
    upload_image(&state, user.account_id, &headers, body, ImageSlot::Banner).await
}

/// Read another account's profile (authenticated).
#[utoipa::path(
    get,
    path = "/api/v1/profiles/{account_id}",
    operation_id = "getProfile",
    tag = "profiles",
    params(("account_id" = Uuid, Path, description = "Account id")),
    responses(
        (status = 200, description = "Profile", body = ProfileResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_profile_by_id(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(account_id): Path<Uuid>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let profile = get_profile(&state.pool, account_id)
        .await
        .map_err(|error| map_db(&error, request_id.clone()))?
        .ok_or_else(|| ApiError::not_found(request_id))?;
    Ok(Json(to_response(&state, &profile, user.account_id).await))
}

/// Stream an account's avatar (authenticated).
#[utoipa::path(
    get,
    path = "/api/v1/profiles/{account_id}/avatar",
    operation_id = "getProfileAvatar",
    tag = "profiles",
    params(("account_id" = Uuid, Path, description = "Account id")),
    responses(
        (status = 200, description = "Image bytes"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_profile_avatar(
    State(state): State<AppState>,
    _user: AuthUser,
    headers: HeaderMap,
    Path(account_id): Path<Uuid>,
) -> Response {
    serve_slot(&state, account_id, &headers, ImageSlot::Avatar).await
}

/// Stream an account's banner (authenticated).
#[utoipa::path(
    get,
    path = "/api/v1/profiles/{account_id}/banner",
    operation_id = "getProfileBanner",
    tag = "profiles",
    params(("account_id" = Uuid, Path, description = "Account id")),
    responses(
        (status = 200, description = "Image bytes"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_profile_banner(
    State(state): State<AppState>,
    _user: AuthUser,
    headers: HeaderMap,
    Path(account_id): Path<Uuid>,
) -> Response {
    serve_slot(&state, account_id, &headers, ImageSlot::Banner).await
}

#[derive(Clone, Copy)]
enum ImageSlot {
    Avatar,
    Banner,
}

impl ImageSlot {
    fn max_bytes(self) -> usize {
        match self {
            Self::Avatar => AVATAR_MAX_BYTES,
            Self::Banner => BANNER_MAX_BYTES,
        }
    }

    fn folder(self) -> &'static str {
        match self {
            Self::Avatar => "avatars",
            Self::Banner => "banners",
        }
    }
}

async fn upload_image(
    state: &AppState,
    account_id: Uuid,
    headers: &HeaderMap,
    body: Bytes,
    slot: ImageSlot,
) -> Result<Json<ProfileResponse>, ApiError> {
    let request_id = request_id_from_headers(headers);
    if body.len() > slot.max_bytes() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            format!("Image exceeds maximum size of {} bytes.", slot.max_bytes()),
            None,
            request_id,
        ));
    }
    let Some(kind) = sniff_image(&body) else {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "Image must be JPEG, PNG, GIF, or WebP.",
            None,
            request_id,
        ));
    };

    let object_id = Uuid::now_v7();
    let key_str = format!(
        "vn/{}/{account_id}/{object_id}.{}",
        slot.folder(),
        kind.extension()
    );
    let key = ObjectKey::parse(&key_str).map_err(|_| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            error_codes::INTERNAL,
            "Unexpected server error.",
            None,
            request_id.clone(),
        )
    })?;

    let digest = Sha256::digest(&body);
    state
        .storage
        .put(key, body.clone(), kind.mime())
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "profile image put failed");
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
        account_id,
    )
    .await
    .map_err(|error| map_db(&error, request_id.clone()))?;

    let previous = match slot {
        ImageSlot::Avatar => set_avatar_object(&state.pool, account_id, object_id).await,
        ImageSlot::Banner => set_banner_object(&state.pool, account_id, object_id).await,
    }
    .map_err(|error| map_db(&error, request_id.clone()))?;

    if let Some(old_id) = previous {
        if let Ok(Some(old)) = get_object(&state.pool, old_id).await {
            if let Ok(old_key) = ObjectKey::parse(&old.storage_key) {
                let _ = state.storage.delete(&old_key).await;
            }
            let _ = delete_object_meta(&state.pool, old_id).await;
        }
    }

    let profile = get_profile(&state.pool, account_id)
        .await
        .map_err(|error| map_db(&error, request_id.clone()))?
        .ok_or_else(|| ApiError::not_found(request_id))?;
    Ok(Json(to_response(state, &profile, account_id).await))
}

async fn serve_slot(
    state: &AppState,
    account_id: Uuid,
    headers: &HeaderMap,
    slot: ImageSlot,
) -> Response {
    let request_id = request_id_from_headers(headers);
    let profile = match get_profile(&state.pool, account_id).await {
        Ok(Some(profile)) => profile,
        Ok(None) => return ApiError::not_found(request_id).into_response(),
        Err(error) => return map_db(&error, request_id).into_response(),
    };
    let object_id = match slot {
        ImageSlot::Avatar => profile.avatar_object_id,
        ImageSlot::Banner => profile.banner_object_id,
    };
    let Some(object_id) = object_id else {
        return ApiError::not_found(request_id).into_response();
    };
    let meta = match get_object(&state.pool, object_id).await {
        Ok(Some(meta)) => meta,
        Ok(None) => return ApiError::not_found(request_id).into_response(),
        Err(error) => return map_db(&error, request_id).into_response(),
    };
    let Ok(key) = ObjectKey::parse(&meta.storage_key) else {
        return ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            error_codes::INTERNAL,
            "Unexpected server error.",
            None,
            request_id,
        )
        .into_response();
    };
    match state.storage.get(&key).await {
        Ok(bytes) => {
            let mut response = bytes.into_response();
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_str(&meta.mime).unwrap_or_else(|_| {
                    header::HeaderValue::from_static("application/octet-stream")
                }),
            );
            response.headers_mut().insert(
                header::CACHE_CONTROL,
                header::HeaderValue::from_static("private, max-age=3600"),
            );
            response
        }
        Err(error) => {
            tracing::warn!(error = %error, %account_id, "profile image get failed");
            ApiError::not_found(request_id).into_response()
        }
    }
}

async fn to_response(state: &AppState, profile: &Profile, viewer_id: Uuid) -> ProfileResponse {
    let (presence_status, custom_status) = state
        .presence_hub
        .view(profile.account_id, viewer_id, &profile.custom_status)
        .await;
    ProfileResponse {
        account_id: profile.account_id,
        display_name: profile.display_name.clone(),
        bio: profile.bio.clone(),
        presence_status,
        custom_status,
        has_avatar: profile.avatar_object_id.is_some(),
        has_banner: profile.banner_object_id.is_some(),
        avatar_url: profile
            .avatar_object_id
            .map(|_| format!("/api/v1/profiles/{}/avatar", profile.account_id)),
        banner_url: profile
            .banner_object_id
            .map(|_| format!("/api/v1/profiles/{}/banner", profile.account_id)),
    }
}

fn map_db(error: &AuthError, request_id: String) -> ApiError {
    tracing::error!(error = %error, "profile db error");
    ApiError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        error_codes::INTERNAL,
        "Unexpected server error.",
        None,
        request_id,
    )
}
