//! Channel message attachments (F038).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Deserializer};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use voxnexus_auth::{
    create_pending_attachment, get_attachment, get_channel, get_object, insert_object,
    set_attachment_thumbnail, AuthError,
};
use voxnexus_domain::{Channel, ChannelType, MessageAttachment};
use voxnexus_jobs::{enqueue_thumbnail, thumbnail_storage, ThumbnailJob};
use voxnexus_media::{make_jpeg_thumbnail, sniff_image, validate_attachment, ATTACHMENT_MAX_BYTES};
use voxnexus_protocol::error_codes;
use voxnexus_protocol::AttachmentResponse;
use voxnexus_storage::ObjectKey;

use crate::error::ApiError;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};
use crate::permissions::{require_channel_attach, require_channel_view};

#[derive(Debug, Default, Deserialize)]
pub struct DownloadQuery {
    /// Prefer the thumbnail object when present. Accepts `true`/`1`/`yes`.
    #[serde(default, deserialize_with = "deserialize_truthy_flag")]
    pub thumb: bool,
}

fn deserialize_truthy_flag<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(raw) = Option::<String>::deserialize(deserializer)? else {
        return Ok(false);
    };
    Ok(matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    ))
}

/// Upload a pending attachment for a text channel. Requires `text.attach`.
#[utoipa::path(
    post,
    path = "/api/v1/channels/{channel_id}/attachments",
    operation_id = "uploadChannelAttachment",
    tag = "messages",
    params(("channel_id" = Uuid, Path, description = "Channel id")),
    request_body(content_type = "application/octet-stream", description = "Raw file bytes"),
    responses(
        (status = 201, description = "Attachment stored", body = AttachmentResponse),
        (status = 400, description = "Invalid request", body = voxnexus_protocol::ErrorBody),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn upload_channel_attachment(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(channel_id): Path<Uuid>,
    body: Bytes,
) -> Result<(StatusCode, Json<AttachmentResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    let channel = load_text_channel(&state, channel_id, request_id.clone()).await?;
    require_channel_attach(&state, &channel, user.account_id, request_id.clone()).await?;
    if channel.archived_at.is_some() {
        return Err(ApiError::permission_denied(
            request_id,
            "Cannot upload attachments in an archived channel.",
        ));
    }
    if body.len() > ATTACHMENT_MAX_BYTES {
        return Err(bad_request(
            request_id,
            format!("Attachment exceeds maximum size of {ATTACHMENT_MAX_BYTES} bytes."),
        ));
    }

    let filename = filename_from_headers(&headers).unwrap_or_else(|| "file".to_owned());
    let Some(allowed) = validate_attachment(&body, &filename) else {
        return Err(bad_request(
            request_id,
            "Attachment type is not allowed (executables are rejected; use images, PDF, or text).",
        ));
    };

    let object_id = Uuid::now_v7();
    let ext = filename
        .rsplit('.')
        .next()
        .filter(|part| !part.is_empty() && part.len() <= 16)
        .map_or_else(
            || {
                sniff_image(&body)
                    .map(|kind| kind.extension().to_owned())
                    .unwrap_or_else(|| "bin".to_owned())
            },
            str::to_ascii_lowercase,
        );
    let key_str = format!(
        "vn/attachments/{}/{}/{object_id}.{ext}",
        channel.community_id, channel.id
    );
    let key = ObjectKey::parse(&key_str).map_err(|_| internal(request_id.clone()))?;
    let digest = Sha256::digest(&body);
    state
        .storage
        .put(key, body.clone(), &allowed.content_type)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "attachment put failed");
            internal(request_id.clone())
        })?;

    insert_object(
        &state.pool,
        object_id,
        &key_str,
        digest.as_slice(),
        &allowed.content_type,
        i64::try_from(body.len()).unwrap_or(i64::MAX),
        user.account_id,
    )
    .await
    .map_err(|error| map_auth(error, request_id.clone()))?;

    let attachment = create_pending_attachment(
        &state.pool,
        channel.id,
        channel.community_id,
        object_id,
        &filename,
        &allowed.content_type,
        i64::try_from(body.len()).unwrap_or(i64::MAX),
        allowed.width.and_then(|w| i32::try_from(w).ok()),
        allowed.height.and_then(|h| i32::try_from(h).ok()),
        user.account_id,
    )
    .await
    .map_err(|error| map_auth(error, request_id.clone()))?;

    if sniff_image(&body).is_some() {
        if let Err(error) = generate_thumbnail(&state, &attachment, &body).await {
            tracing::warn!(error = %error, attachment_id = %attachment.id, "inline thumbnail failed");
        }
        let mut storage = thumbnail_storage(state.redis.clone());
        if let Err(error) = enqueue_thumbnail(&mut storage, ThumbnailJob::new(attachment.id)).await
        {
            tracing::warn!(error = %error, "enqueue thumbnail failed");
        }
    }

    let refreshed = get_attachment(&state.pool, attachment.id)
        .await
        .map_err(|error| map_auth(error, request_id))?
        .unwrap_or(attachment);

    Ok((StatusCode::CREATED, Json(to_attachment_response(&refreshed))))
}

/// Download an attachment (or thumbnail). Requires `text.view` on its channel.
#[utoipa::path(
    get,
    path = "/api/v1/attachments/{attachment_id}",
    operation_id = "downloadAttachment",
    tag = "messages",
    params(
        ("attachment_id" = Uuid, Path, description = "Attachment id"),
        ("thumb" = Option<String>, Query, description = "Prefer thumbnail when available (`true`/`1`)")
    ),
    responses(
        (status = 200, description = "Attachment bytes"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn download_attachment(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(attachment_id): Path<Uuid>,
    Query(query): Query<DownloadQuery>,
) -> axum::response::Response {
    let request_id = request_id_from_headers(&headers);
    let attachment = match get_attachment(&state.pool, attachment_id).await {
        Ok(Some(row)) => row,
        Ok(None) => return ApiError::not_found(request_id).into_response(),
        Err(error) => {
            tracing::error!(error = %error, "get attachment failed");
            return internal(request_id).into_response();
        }
    };

    let channel = match get_channel(&state.pool, attachment.channel_id).await {
        Ok(Some(channel)) => channel,
        Ok(None) => return ApiError::not_found(request_id).into_response(),
        Err(error) => {
            tracing::error!(error = %error, "get channel for attachment failed");
            return internal(request_id).into_response();
        }
    };

    if require_channel_view(&state, &channel, user.account_id, request_id.clone())
        .await
        .is_err()
    {
        // Do not leak attachment existence to outsiders / hidden-channel members.
        return ApiError::not_found(request_id).into_response();
    }

    let want_thumb = query.thumb;
    let object_id = if want_thumb {
        attachment
            .thumbnail_object_id
            .unwrap_or(attachment.object_id)
    } else {
        attachment.object_id
    };

    let meta = match get_object(&state.pool, object_id).await {
        Ok(Some(meta)) => meta,
        Ok(None) => return ApiError::not_found(request_id).into_response(),
        Err(error) => {
            tracing::error!(error = %error, "get object failed");
            return internal(request_id).into_response();
        }
    };
    let Ok(key) = ObjectKey::parse(&meta.storage_key) else {
        return internal(request_id).into_response();
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
            if let Ok(value) = header::HeaderValue::from_str(&format!(
                "inline; filename=\"{}\"",
                attachment.filename.replace('"', "")
            )) {
                response
                    .headers_mut()
                    .insert(header::CONTENT_DISPOSITION, value);
            }
            response.headers_mut().insert(
                header::CACHE_CONTROL,
                header::HeaderValue::from_static("private, max-age=3600"),
            );
            response
        }
        Err(error) => {
            tracing::warn!(error = %error, %attachment_id, "attachment get failed");
            ApiError::not_found(request_id).into_response()
        }
    }
}

/// Generate a thumbnail for an attachment if missing (used by worker + inline upload).
pub async fn generate_thumbnail_for_attachment(
    state: &AppState,
    attachment_id: Uuid,
) -> Result<(), String> {
    let attachment = get_attachment(&state.pool, attachment_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "attachment not found".to_owned())?;
    if attachment.thumbnail_object_id.is_some() {
        return Ok(());
    }
    let meta = get_object(&state.pool, attachment.object_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "object not found".to_owned())?;
    let key = ObjectKey::parse(&meta.storage_key).map_err(|error| error.to_string())?;
    let bytes = state
        .storage
        .get(&key)
        .await
        .map_err(|error| error.to_string())?;
    generate_thumbnail(state, &attachment, &bytes).await
}

async fn generate_thumbnail(
    state: &AppState,
    attachment: &MessageAttachment,
    bytes: &Bytes,
) -> Result<(), String> {
    if attachment.thumbnail_object_id.is_some() {
        return Ok(());
    }
    if sniff_image(bytes).is_none() {
        return Ok(());
    }
    let thumb_bytes = make_jpeg_thumbnail(bytes)?;
    let thumb_id = Uuid::now_v7();
    let key_str = format!(
        "vn/attachments/{}/{}/{thumb_id}.thumb.jpg",
        attachment.community_id, attachment.channel_id
    );
    let key = ObjectKey::parse(&key_str).map_err(|error| error.to_string())?;
    let digest = Sha256::digest(&thumb_bytes);
    state
        .storage
        .put(key, Bytes::from(thumb_bytes.clone()), "image/jpeg")
        .await
        .map_err(|error| error.to_string())?;
    insert_object(
        &state.pool,
        thumb_id,
        &key_str,
        digest.as_slice(),
        "image/jpeg",
        i64::try_from(thumb_bytes.len()).unwrap_or(i64::MAX),
        attachment.created_by,
    )
    .await
    .map_err(|error| error.to_string())?;
    set_attachment_thumbnail(&state.pool, attachment.id, thumb_id)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[must_use]
pub fn to_attachment_response(attachment: &MessageAttachment) -> AttachmentResponse {
    AttachmentResponse {
        id: attachment.id,
        filename: attachment.filename.clone(),
        content_type: attachment.content_type.clone(),
        byte_size: attachment.byte_size,
        width: attachment.width,
        height: attachment.height,
        url: format!("/api/v1/attachments/{}", attachment.id),
        thumbnail_url: attachment
            .thumbnail_object_id
            .map(|_| format!("/api/v1/attachments/{}?thumb=1", attachment.id)),
    }
}

async fn load_text_channel(
    state: &AppState,
    channel_id: Uuid,
    request_id: String,
) -> Result<Channel, ApiError> {
    let channel = get_channel(&state.pool, channel_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "get channel failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| ApiError::not_found(request_id.clone()))?;
    if channel.channel_type != ChannelType::Text {
        return Err(bad_request(
            request_id,
            "Attachments are only available in text channels.",
        ));
    }
    Ok(channel)
}

fn filename_from_headers(headers: &HeaderMap) -> Option<String> {
    if let Some(value) = headers
        .get("x-filename")
        .and_then(|value| value.to_str().ok())
    {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_owned());
        }
    }
    let disposition = headers
        .get(header::CONTENT_DISPOSITION)
        .and_then(|value| value.to_str().ok())?;
    for part in disposition.split(';') {
        let part = part.trim();
        if let Some(name) = part.strip_prefix("filename=") {
            let name = name.trim().trim_matches('"').trim();
            if !name.is_empty() {
                return Some(name.to_owned());
            }
        }
    }
    None
}

fn map_auth(error: AuthError, request_id: String) -> ApiError {
    match error {
        AuthError::InvalidAttachment => bad_request(request_id, "Invalid attachment."),
        other => {
            tracing::error!(error = %other, "attachment auth error");
            internal(request_id)
        }
    }
}

fn bad_request(request_id: impl Into<String>, message: impl Into<String>) -> ApiError {
    ApiError::new(
        StatusCode::BAD_REQUEST,
        error_codes::VALIDATION_ERROR,
        message,
        None,
        request_id,
    )
}

fn internal(request_id: impl Into<String>) -> ApiError {
    ApiError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        error_codes::INTERNAL,
        "Unexpected server error.",
        None,
        request_id,
    )
}
