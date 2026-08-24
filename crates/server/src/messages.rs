//! Channel messages API (F034 / F036 / F037).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use chrono::{Duration, Utc};
use uuid::Uuid;
use voxnexus_auth::{
    create_message as persist_create, get_channel, get_message, list_member_account_ids,
    list_messages as persist_list, soft_delete_message as persist_delete,
    update_message as persist_update, AuthError, MessageWithAuthor, MESSAGE_EDIT_WINDOW_SECS,
};
use voxnexus_domain::{Channel, ChannelType};
use voxnexus_permissions::PermissionCode;
use voxnexus_protocol::error_codes;
use voxnexus_protocol::{
    CreateMessageRequest, ListMessagesQuery, MessageCreatePayload, MessageDeletePayload,
    MessageListResponse, MessageResponse, MessageUpdatePayload, UpdateMessageRequest,
};
use voxnexus_realtime::PresenceHubMessage;

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};
use crate::permissions::{allowed_for_channel, require_channel_send, require_channel_view};

/// Send a message in a text channel. Requires `text.view` + `text.send`.
///
/// Optional body `nonce` or header `Idempotency-Key` for idempotent create.
#[utoipa::path(
    post,
    path = "/api/v1/channels/{channel_id}/messages",
    operation_id = "createMessage",
    tag = "messages",
    params(("channel_id" = Uuid, Path, description = "Channel id")),
    request_body = CreateMessageRequest,
    responses(
        (status = 201, description = "Message created", body = MessageResponse),
        (status = 200, description = "Idempotent replay", body = MessageResponse),
        (status = 400, description = "Invalid request", body = voxnexus_protocol::ErrorBody),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn create_message(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(channel_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<CreateMessageRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    let channel = load_text_channel(&state, channel_id, request_id.clone()).await?;
    require_channel_send(&state, &channel, user.account_id, request_id.clone()).await?;
    require_not_archived(&channel, request_id.clone())?;

    let header_nonce = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let nonce = body.nonce.as_deref().or(header_nonce.as_deref());

    let (row, created) = persist_create(
        &state.pool,
        channel.id,
        channel.community_id,
        user.account_id,
        &body.content,
        nonce,
        body.referenced_message_id,
    )
    .await
    .map_err(|error| map_message_auth(error, request_id.clone()))?;

    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    if created {
        broadcast_message_event(
            &state,
            &channel,
            PresenceHubMessage::MessageCreate(to_create_payload(&row)),
        )
        .await;
    }
    Ok((status, Json(to_response(&row))))
}

/// List messages in a channel (newest first). Requires `text.view` (404 if hidden).
#[utoipa::path(
    get,
    path = "/api/v1/channels/{channel_id}/messages",
    operation_id = "listMessages",
    tag = "messages",
    params(
        ("channel_id" = Uuid, Path, description = "Channel id"),
        ListMessagesQuery
    ),
    responses(
        (status = 200, description = "Message page", body = MessageListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn list_messages(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<MessageListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let channel = load_text_channel(&state, channel_id, request_id.clone()).await?;
    require_channel_view(&state, &channel, user.account_id, request_id.clone()).await?;

    if query.before.is_some() && query.after.is_some() {
        return Err(bad_request(
            request_id,
            "Specify at most one of before or after.",
        ));
    }

    let page = persist_list(
        &state.pool,
        channel.id,
        query.before,
        query.after,
        query.resolved_limit(),
    )
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "list messages failed");
        internal(request_id)
    })?;

    Ok(Json(MessageListResponse {
        items: page.items.iter().map(to_response).collect(),
        has_more: page.has_more,
    }))
}

/// Edit a message. Author only. Requires channel view.
#[utoipa::path(
    patch,
    path = "/api/v1/channels/{channel_id}/messages/{message_id}",
    operation_id = "updateMessage",
    tag = "messages",
    params(
        ("channel_id" = Uuid, Path, description = "Channel id"),
        ("message_id" = Uuid, Path, description = "Message id")
    ),
    request_body = UpdateMessageRequest,
    responses(
        (status = 200, description = "Message updated", body = MessageResponse),
        (status = 400, description = "Invalid request", body = voxnexus_protocol::ErrorBody),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn update_message(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path((channel_id, message_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(body): ValidatedJson<UpdateMessageRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let channel = load_text_channel(&state, channel_id, request_id.clone()).await?;
    require_channel_view(&state, &channel, user.account_id, request_id.clone()).await?;
    require_not_archived(&channel, request_id.clone())?;

    let existing = get_message(&state.pool, message_id, Some(channel.id))
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "get message failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| ApiError::not_found(request_id.clone()))?;

    if existing.message.author_id != user.account_id {
        return Err(ApiError::permission_denied(
            request_id,
            "You can only edit your own messages.",
        ));
    }
    if let Some(window_secs) = MESSAGE_EDIT_WINDOW_SECS {
        let age = Utc::now().signed_duration_since(existing.message.created_at);
        if age > Duration::seconds(window_secs) {
            return Err(ApiError::permission_denied(
                request_id,
                "The edit window for this message has expired.",
            ));
        }
    }

    let row = persist_update(&state.pool, message_id, &body.content)
        .await
        .map_err(|error| map_message_auth(error, request_id))?;

    broadcast_message_event(
        &state,
        &channel,
        PresenceHubMessage::MessageUpdate(to_update_payload(&row)),
    )
    .await;

    Ok(Json(to_response(&row)))
}

/// Soft-delete a message. Author or `text.manage_messages`.
#[utoipa::path(
    delete,
    path = "/api/v1/channels/{channel_id}/messages/{message_id}",
    operation_id = "deleteMessage",
    tag = "messages",
    params(
        ("channel_id" = Uuid, Path, description = "Channel id"),
        ("message_id" = Uuid, Path, description = "Message id")
    ),
    responses(
        (status = 204, description = "Message deleted"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn delete_message(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path((channel_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let channel = load_text_channel(&state, channel_id, request_id.clone()).await?;
    require_channel_view(&state, &channel, user.account_id, request_id.clone()).await?;

    let existing = get_message(&state.pool, message_id, Some(channel.id))
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "get message failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| ApiError::not_found(request_id.clone()))?;

    let is_author = existing.message.author_id == user.account_id;
    let can_manage = allowed_for_channel(
        &state,
        &channel,
        user.account_id,
        PermissionCode::TEXT_MANAGE_MESSAGES,
    )
    .await?;
    if !is_author && !can_manage {
        return Err(ApiError::permission_denied(
            request_id,
            "You do not have permission to delete this message.",
        ));
    }

    let deleted = persist_delete(&state.pool, message_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "delete message failed");
            internal(request_id.clone())
        })?;
    if !deleted {
        return Err(ApiError::not_found(request_id));
    }

    broadcast_message_event(
        &state,
        &channel,
        PresenceHubMessage::MessageDelete(MessageDeletePayload {
            id: existing.message.id,
            channel_id: existing.message.channel_id,
            community_id: existing.message.community_id,
        }),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
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
            "Messages are only available in text channels.",
        ));
    }
    Ok(channel)
}

fn require_not_archived(channel: &Channel, request_id: String) -> Result<(), ApiError> {
    if channel.archived_at.is_some() {
        return Err(ApiError::permission_denied(
            request_id,
            "Cannot modify messages in an archived channel.",
        ));
    }
    Ok(())
}

fn to_response(row: &MessageWithAuthor) -> MessageResponse {
    MessageResponse {
        id: row.message.id,
        channel_id: row.message.channel_id,
        community_id: row.message.community_id,
        author_id: row.message.author_id,
        author_display_name: row.author_display_name.clone(),
        content: row.message.content.clone(),
        nonce: row.message.nonce.clone(),
        referenced_message_id: row.message.referenced_message_id,
        reply_to: row.reply.as_ref().map(to_reply_preview),
        created_at: row.message.created_at,
        edited_at: row.message.edited_at,
    }
}

fn to_create_payload(row: &MessageWithAuthor) -> MessageCreatePayload {
    MessageCreatePayload {
        id: row.message.id,
        channel_id: row.message.channel_id,
        community_id: row.message.community_id,
        author_id: row.message.author_id,
        author_display_name: row.author_display_name.clone(),
        content: row.message.content.clone(),
        nonce: row.message.nonce.clone(),
        referenced_message_id: row.message.referenced_message_id,
        reply_to: row.reply.as_ref().map(to_reply_preview),
        created_at: row.message.created_at,
        edited_at: row.message.edited_at,
    }
}

fn to_update_payload(row: &MessageWithAuthor) -> MessageUpdatePayload {
    MessageUpdatePayload {
        id: row.message.id,
        channel_id: row.message.channel_id,
        community_id: row.message.community_id,
        author_id: row.message.author_id,
        author_display_name: row.author_display_name.clone(),
        content: row.message.content.clone(),
        nonce: row.message.nonce.clone(),
        referenced_message_id: row.message.referenced_message_id,
        reply_to: row.reply.as_ref().map(to_reply_preview),
        created_at: row.message.created_at,
        edited_at: row.message.edited_at,
    }
}

fn to_reply_preview(
    reply: &voxnexus_auth::MessageReplyPreview,
) -> voxnexus_protocol::MessageReplyPreview {
    voxnexus_protocol::MessageReplyPreview {
        message_id: reply.message_id,
        author_id: reply.author_id,
        author_display_name: reply.author_display_name.clone(),
        excerpt: reply.excerpt.clone(),
        deleted: reply.deleted,
    }
}

async fn broadcast_message_event(state: &AppState, channel: &Channel, message: PresenceHubMessage) {
    let members = match list_member_account_ids(&state.pool, channel.community_id).await {
        Ok(ids) => ids,
        Err(error) => {
            tracing::error!(error = %error, "list members for message fanout failed");
            return;
        }
    };
    let mut recipients = Vec::new();
    for account_id in members {
        if let Ok(true) =
            allowed_for_channel(state, channel, account_id, PermissionCode::TEXT_VIEW).await
        {
            recipients.push(account_id);
        }
    }
    state
        .presence_hub
        .broadcast_to_accounts(&recipients, message)
        .await;
}

fn map_message_auth(error: AuthError, request_id: String) -> ApiError {
    match error {
        AuthError::InvalidMessage => bad_request(request_id, "Invalid message."),
        AuthError::InvalidReplyTarget => bad_request(
            request_id,
            "Reply target must be a message in this channel.",
        ),
        other => {
            tracing::error!(error = %other, "message auth error");
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
