//! Channel messages API (F034–F038).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use chrono::{Duration, Utc};
use uuid::Uuid;
use voxnexus_auth::{
    create_message as persist_create, get_channel, get_membership, get_message, get_role,
    list_attachments_for_messages, list_member_account_ids, list_mentions_for_messages,
    list_messages as persist_list, replace_message_mentions, soft_delete_message as persist_delete,
    update_message as persist_update, AuthError, MessageWithAuthor, MESSAGE_EDIT_WINDOW_SECS,
};
use voxnexus_domain::{Channel, ChannelType};
use voxnexus_permissions::PermissionCode;
use voxnexus_protocol::error_codes;
use voxnexus_protocol::{
    parse_mentions, AttachmentResponse, CreateMessageRequest, ListMessagesQuery,
    MessageCreatePayload, MessageDeletePayload, MessageListResponse, MessageMentions,
    MessageResponse, MessageUpdatePayload, UpdateMessageRequest,
};
use voxnexus_realtime::PresenceHubMessage;

use crate::attachments::to_attachment_response;
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

    let attachment_ids = body.attachment_ids.clone().unwrap_or_default();
    let pending_mentions = validate_mentions(
        &state,
        &channel,
        user.account_id,
        &body.content,
        &request_id,
    )
    .await?;

    let (row, created) = persist_create(
        &state.pool,
        channel.id,
        channel.community_id,
        user.account_id,
        &body.content,
        nonce,
        body.referenced_message_id,
        &attachment_ids,
    )
    .await
    .map_err(|error| map_message_auth(error, request_id.clone()))?;

    let attachments = load_message_attachments(&state, &[row.message.id], &request_id).await?;
    let mentions = if created {
        replace_message_mentions(&state.pool, row.message.id, &pending_mentions)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "store mentions failed");
                internal(request_id.clone())
            })?;
        MessageMentions::from(pending_mentions)
    } else {
        load_message_mentions(&state, &[row.message.id], &request_id)
            .await?
            .into_iter()
            .next()
            .map(|(_, m)| m)
            .unwrap_or_default()
    };
    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    if created {
        broadcast_message_event(
            &state,
            &channel,
            PresenceHubMessage::MessageCreate(to_create_payload(&row, &attachments, &mentions)),
        )
        .await;
    }
    Ok((status, Json(to_response(&row, &attachments, &mentions))))
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
        internal(request_id.clone())
    })?;

    let ids: Vec<Uuid> = page.items.iter().map(|row| row.message.id).collect();
    let attached = list_attachments_for_messages(&state.pool, &ids)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list attachments failed");
            internal(request_id.clone())
        })?;
    let mention_rows = load_message_mentions(&state, &ids, &request_id).await?;
    Ok(Json(MessageListResponse {
        items: page
            .items
            .iter()
            .map(|row| {
                let attachments = attached
                    .iter()
                    .filter(|att| att.message_id == Some(row.message.id))
                    .map(to_attachment_response)
                    .collect::<Vec<_>>();
                let mentions = mention_rows
                    .iter()
                    .find(|(id, _)| *id == row.message.id)
                    .map(|(_, m)| m.clone())
                    .unwrap_or_default();
                to_response(row, &attachments, &mentions)
            })
            .collect(),
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

    let pending_mentions = validate_mentions(
        &state,
        &channel,
        user.account_id,
        &body.content,
        &request_id,
    )
    .await?;

    let row = persist_update(&state.pool, message_id, &body.content)
        .await
        .map_err(|error| map_message_auth(error, request_id.clone()))?;

    replace_message_mentions(&state.pool, row.message.id, &pending_mentions)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "store mentions failed");
            internal(request_id.clone())
        })?;
    let mentions = MessageMentions::from(pending_mentions);

    let attachments = load_message_attachments(&state, &[row.message.id], &request_id).await?;
    broadcast_message_event(
        &state,
        &channel,
        PresenceHubMessage::MessageUpdate(to_update_payload(&row, &attachments, &mentions)),
    )
    .await;

    Ok(Json(to_response(&row, &attachments, &mentions)))
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

fn to_response(
    row: &MessageWithAuthor,
    attachments: &[AttachmentResponse],
    mentions: &MessageMentions,
) -> MessageResponse {
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
        attachments: attachments.to_vec(),
        mentions: mentions.clone(),
        created_at: row.message.created_at,
        edited_at: row.message.edited_at,
    }
}

fn to_create_payload(
    row: &MessageWithAuthor,
    attachments: &[AttachmentResponse],
    mentions: &MessageMentions,
) -> MessageCreatePayload {
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
        attachments: attachments.to_vec(),
        mentions: mentions.clone(),
        created_at: row.message.created_at,
        edited_at: row.message.edited_at,
    }
}

fn to_update_payload(
    row: &MessageWithAuthor,
    attachments: &[AttachmentResponse],
    mentions: &MessageMentions,
) -> MessageUpdatePayload {
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
        attachments: attachments.to_vec(),
        mentions: mentions.clone(),
        created_at: row.message.created_at,
        edited_at: row.message.edited_at,
    }
}

async fn load_message_attachments(
    state: &AppState,
    message_ids: &[Uuid],
    request_id: &str,
) -> Result<Vec<AttachmentResponse>, ApiError> {
    let rows = list_attachments_for_messages(&state.pool, message_ids)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list attachments failed");
            internal(request_id.to_owned())
        })?;
    Ok(rows.iter().map(to_attachment_response).collect())
}

async fn load_message_mentions(
    state: &AppState,
    message_ids: &[Uuid],
    request_id: &str,
) -> Result<Vec<(Uuid, MessageMentions)>, ApiError> {
    list_mentions_for_messages(&state.pool, message_ids)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list mentions failed");
            internal(request_id.to_owned())
        })
}

async fn validate_mentions(
    state: &AppState,
    channel: &Channel,
    actor_id: Uuid,
    content: &str,
    request_id: &str,
) -> Result<voxnexus_protocol::MentionSet, ApiError> {
    let set = parse_mentions(content);
    if set.everyone || set.here {
        let allowed = allowed_for_channel(
            state,
            channel,
            actor_id,
            PermissionCode::COMMUNITY_MENTION_EVERYONE,
        )
        .await?;
        if !allowed {
            return Err(bad_request(
                request_id.to_owned(),
                "You do not have permission to mention @everyone or @here.",
            ));
        }
    }
    if !set.role_ids.is_empty() {
        let can_mention_roles =
            allowed_for_channel(state, channel, actor_id, PermissionCode::TEXT_MENTION_ROLES)
                .await?;
        if !can_mention_roles {
            return Err(bad_request(
                request_id.to_owned(),
                "You do not have permission to mention roles.",
            ));
        }
        let can_force = allowed_for_channel(
            state,
            channel,
            actor_id,
            PermissionCode::COMMUNITY_MANAGE_ROLES,
        )
        .await?
            || allowed_for_channel(
                state,
                channel,
                actor_id,
                PermissionCode::COMMUNITY_MENTION_EVERYONE,
            )
            .await?;
        for role_id in &set.role_ids {
            let role = get_role(&state.pool, *role_id)
                .await
                .map_err(|error| {
                    tracing::error!(error = %error, "get role for mention failed");
                    internal(request_id.to_owned())
                })?
                .ok_or_else(|| {
                    bad_request(request_id.to_owned(), "Mentioned role was not found.")
                })?;
            if role.community_id != channel.community_id {
                return Err(bad_request(
                    request_id.to_owned(),
                    "Mentioned role is not in this community.",
                ));
            }
            if !role.mentionable && !can_force {
                return Err(bad_request(
                    request_id.to_owned(),
                    "That role is not mentionable.",
                ));
            }
        }
    }
    for account_id in &set.account_ids {
        let member = get_membership(&state.pool, channel.community_id, *account_id)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "membership lookup for mention failed");
                internal(request_id.to_owned())
            })?;
        if member.is_none() {
            return Err(bad_request(
                request_id.to_owned(),
                "Mentioned user is not a community member.",
            ));
        }
    }
    Ok(set)
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
        AuthError::InvalidAttachment => bad_request(
            request_id,
            "One or more attachments are invalid or already used.",
        ),
        AuthError::InvalidMention => bad_request(request_id, "Invalid mention."),
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
