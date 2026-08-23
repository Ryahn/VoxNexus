//! Community invites (F021).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use chrono::{Duration, Months, Utc};
use uuid::Uuid;
use voxnexus_auth::{
    accept_invite as persist_accept, create_invite as persist_create, get_community, get_invite,
    get_profile, list_invites as persist_list, list_member_account_ids,
    revoke_invite as persist_revoke, update_invite as persist_update, CreateInviteInput,
    InvitePatch, MemberListItem,
};
use voxnexus_domain::{CommunityInvite, CommunityMemberRole};
use voxnexus_protocol::error_codes;
use voxnexus_protocol::{
    CommunityMemberResponse, CreateInviteRequest, InviteExpireAfter, InviteExpireUnit,
    InviteListResponse, InvitePreviewResponse, InviteResponse, MemberJoinPayload,
    UpdateInviteRequest,
};
use voxnexus_realtime::PresenceHubMessage;

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};

/// Create an invite (owner / manage_invites — owner until F029).
#[utoipa::path(
    post,
    path = "/api/v1/communities/{community_id}/invites",
    operation_id = "createCommunityInvite",
    tag = "invites",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body = CreateInviteRequest,
    responses(
        (status = 201, description = "Invite created", body = InviteResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Community not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn create_community_invite(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<CreateInviteRequest>,
) -> Result<(StatusCode, Json<InviteResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_manage_invites(&state, community_id, user.account_id, &request_id).await?;
    let expires_at = resolve_expire_after(body.expire_after.as_ref(), &request_id)?;
    let invite = persist_create(
        &state.pool,
        community_id,
        user.account_id,
        CreateInviteInput {
            max_uses: body.max_uses,
            expires_at,
        },
    )
    .await
    .map_err(|error| map_invite_auth(error, request_id))?;
    Ok((StatusCode::CREATED, Json(to_invite_response(&invite))))
}

/// List active (non-revoked) invites for a community.
#[utoipa::path(
    get,
    path = "/api/v1/communities/{community_id}/invites",
    operation_id = "listCommunityInvites",
    tag = "invites",
    params(("community_id" = Uuid, Path, description = "Community id")),
    responses(
        (status = 200, description = "Invites", body = InviteListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Community not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn list_community_invites(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
) -> Result<Json<InviteListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_manage_invites(&state, community_id, user.account_id, &request_id).await?;
    let invites = persist_list(&state.pool, community_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list invites failed");
            internal(request_id)
        })?;
    Ok(Json(InviteListResponse {
        invites: invites.iter().map(to_invite_response).collect(),
    }))
}

/// Revoke an invite.
#[utoipa::path(
    delete,
    path = "/api/v1/communities/{community_id}/invites/{invite_id}",
    operation_id = "revokeCommunityInvite",
    tag = "invites",
    params(
        ("community_id" = Uuid, Path, description = "Community id"),
        ("invite_id" = Uuid, Path, description = "Invite id")
    ),
    responses(
        (status = 204, description = "Revoked"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn revoke_community_invite(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path((community_id, invite_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_manage_invites(&state, community_id, user.account_id, &request_id).await?;
    let invite = get_invite(&state.pool, invite_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "invite lookup failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| not_found(request_id.clone()))?;
    if invite.community_id != community_id {
        return Err(not_found(request_id));
    }
    persist_revoke(&state.pool, invite_id)
        .await
        .map_err(|error| map_invite_auth(error, request_id))?;
    Ok(StatusCode::NO_CONTENT)
}

/// Pause or unpause an invite.
#[utoipa::path(
    patch,
    path = "/api/v1/communities/{community_id}/invites/{invite_id}",
    operation_id = "updateCommunityInvite",
    tag = "invites",
    params(
        ("community_id" = Uuid, Path, description = "Community id"),
        ("invite_id" = Uuid, Path, description = "Invite id")
    ),
    request_body = UpdateInviteRequest,
    responses(
        (status = 200, description = "Updated", body = InviteResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn update_community_invite(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path((community_id, invite_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(body): ValidatedJson<UpdateInviteRequest>,
) -> Result<Json<InviteResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_manage_invites(&state, community_id, user.account_id, &request_id).await?;
    let invite = get_invite(&state.pool, invite_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "invite lookup failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| not_found(request_id.clone()))?;
    if invite.community_id != community_id {
        return Err(not_found(request_id));
    }
    let updated = persist_update(
        &state.pool,
        invite_id,
        InvitePatch {
            paused: body.paused,
        },
    )
    .await
    .map_err(|error| map_invite_auth(error, request_id))?;
    Ok(Json(to_invite_response(&updated)))
}

/// Preview an invite by code (authenticated).
#[utoipa::path(
    get,
    path = "/api/v1/invites/{code}",
    operation_id = "getInvitePreview",
    tag = "invites",
    params(("code" = String, Path, description = "Invite code")),
    responses(
        (status = 200, description = "Invite preview", body = InvitePreviewResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_invite_preview(
    State(state): State<AppState>,
    _user: AuthUser,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<InvitePreviewResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let invite = voxnexus_auth::get_invite_by_code(&state.pool, code.trim())
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "invite preview lookup failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| not_found(request_id.clone()))?;
    if invite.revoked_at.is_some() {
        return Err(not_found(request_id));
    }
    let community = get_community(&state.pool, invite.community_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "community lookup failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| not_found(request_id))?;
    let now = Utc::now();
    Ok(Json(InvitePreviewResponse {
        code: invite.code,
        community_id: community.id,
        community_name: community.name,
        community_slug: community.slug,
        paused: invite.paused,
        expired: invite.expires_at.is_some_and(|expires| expires <= now),
        exhausted: invite.max_uses.is_some_and(|max| invite.uses >= max),
    }))
}

/// Accept an invite by code.
#[utoipa::path(
    post,
    path = "/api/v1/invites/{code}/accept",
    operation_id = "acceptInvite",
    tag = "invites",
    params(("code" = String, Path, description = "Invite code")),
    responses(
        (status = 201, description = "Joined", body = CommunityMemberResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Invite not usable", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody),
        (status = 409, description = "Already a member", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn accept_invite(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<(StatusCode, Json<CommunityMemberResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    let (_invite, member, item) = persist_accept(&state.pool, code.trim(), user.account_id)
        .await
        .map_err(|error| map_invite_auth(error, request_id.clone()))?;

    let mut recipients = list_member_account_ids(&state.pool, member.community_id)
        .await
        .unwrap_or_default();
    recipients.push(user.account_id);
    recipients.sort_unstable();
    recipients.dedup();
    let profile = get_profile(&state.pool, user.account_id)
        .await
        .ok()
        .flatten();
    state
        .presence_hub
        .broadcast_to_accounts(
            &recipients,
            PresenceHubMessage::MemberJoin(MemberJoinPayload {
                community_id: member.community_id,
                account_id: user.account_id,
                role: member.role.as_str().to_owned(),
                nickname: member.nickname.clone(),
                display_name: profile.map_or_else(|| item.display_name.clone(), |p| p.display_name),
            }),
        )
        .await;

    Ok((StatusCode::CREATED, Json(member_response(&item))))
}

async fn require_manage_invites(
    state: &AppState,
    community_id: Uuid,
    account_id: Uuid,
    request_id: &str,
) -> Result<(), ApiError> {
    // Until F029 role permissions, owner stands in for `community.manage_invites`.
    let membership = voxnexus_auth::get_membership(&state.pool, community_id, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "membership lookup failed");
            internal(request_id.to_owned())
        })?;
    match membership {
        Some(member) if member.role == CommunityMemberRole::Owner => Ok(()),
        Some(_) => Err(ApiError::permission_denied(
            request_id.to_owned(),
            "Only the community owner can manage invites.",
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
                    "Only the community owner can manage invites.",
                ))
            }
        }
    }
}

fn resolve_expire_after(
    expire_after: Option<&InviteExpireAfter>,
    request_id: &str,
) -> Result<Option<chrono::DateTime<Utc>>, ApiError> {
    let Some(spec) = expire_after else {
        return Ok(None);
    };
    if spec.value == 0 {
        return Err(validation(
            request_id.to_owned(),
            "Expire value must be at least 1.",
        ));
    }
    let now = Utc::now();
    let expires_at = match spec.unit {
        InviteExpireUnit::Hours => {
            if spec.value > 24 {
                return Err(validation(
                    request_id.to_owned(),
                    "Hourly expiry cannot exceed 24 hours.",
                ));
            }
            now + Duration::hours(i64::from(spec.value))
        }
        InviteExpireUnit::Days => {
            if spec.value > 14 {
                return Err(validation(
                    request_id.to_owned(),
                    "Daily expiry cannot exceed 14 days.",
                ));
            }
            now + Duration::days(i64::from(spec.value))
        }
        InviteExpireUnit::Months => {
            if spec.value > 3 {
                return Err(validation(
                    request_id.to_owned(),
                    "Monthly expiry cannot exceed 3 months.",
                ));
            }
            now.checked_add_months(Months::new(spec.value))
                .ok_or_else(|| validation(request_id.to_owned(), "Invalid expiry duration."))?
        }
    };
    Ok(Some(expires_at))
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

fn to_invite_response(invite: &CommunityInvite) -> InviteResponse {
    InviteResponse {
        id: invite.id,
        community_id: invite.community_id,
        code: invite.code.clone(),
        created_by: invite.created_by,
        max_uses: invite.max_uses,
        uses: invite.uses,
        expires_at: invite.expires_at,
        paused: invite.paused,
        revoked_at: invite.revoked_at,
        created_at: invite.created_at,
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

fn map_invite_auth(error: voxnexus_auth::AuthError, request_id: String) -> ApiError {
    match error {
        voxnexus_auth::AuthError::InviteNotFound => not_found(request_id),
        voxnexus_auth::AuthError::InviteExpired => {
            ApiError::permission_denied(request_id, "This invite has expired.")
        }
        voxnexus_auth::AuthError::InviteExhausted => {
            ApiError::permission_denied(request_id, "This invite has reached its maximum uses.")
        }
        voxnexus_auth::AuthError::InvitePaused => {
            ApiError::permission_denied(request_id, "This invite is paused.")
        }
        voxnexus_auth::AuthError::AlreadyMember => {
            ApiError::conflict(request_id, "You are already a member of this community.")
        }
        voxnexus_auth::AuthError::JoinNotAllowed => {
            ApiError::permission_denied(request_id, "This community cannot be joined via invite.")
        }
        other => {
            let message = other.to_string();
            if message.contains("no rows returned") || message.contains("RowNotFound") {
                return not_found(request_id);
            }
            tracing::error!(error = %other, "invite auth error");
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
        "Invite not found.",
        None,
        request_id,
    )
}
