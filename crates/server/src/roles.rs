//! Community role CRUD and assignments (F028).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use voxnexus_auth::{
    assign_role as persist_assign, can_manage_role_weight, clone_role as persist_clone,
    create_role as persist_create, delete_object_meta, delete_role as persist_delete, get_membership,
    get_object, get_role, insert_object, list_member_account_ids,
    list_member_roles as persist_list_member_roles, list_roles as persist_list,
    permissions_with_manage_roles, remove_role_assignment, role_actor, set_role_icon,
    update_role as persist_update, CreateRoleInput, RoleActor, RolePatch,
};
use voxnexus_domain::CommunityRole;
use voxnexus_media::{sniff_image, AVATAR_MAX_BYTES};
use voxnexus_permissions::PermissionCode;
use voxnexus_protocol::error_codes;
use voxnexus_protocol::{
    AssignRoleRequest, BulkAssignRoleGroupRequest, CommunityRolePayload, CreateRoleGroupRequest,
    CreateRoleRequest, MemberRoleUpdatePayload, ReorderRolesRequest, RoleDeletePayload,
    RoleGroupListResponse, RoleGroupResponse, RoleListResponse, RoleResponse,
    UpdateRoleGroupRequest, UpdateRoleRequest,
};
use voxnexus_realtime::PresenceHubMessage;
use voxnexus_storage::ObjectKey;

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};
use crate::permissions::{invalidate_community, require_permission};

/// Create a custom role.
#[utoipa::path(
    post,
    path = "/api/v1/communities/{community_id}/roles",
    operation_id = "createRole",
    tag = "roles",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created", body = RoleResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody),
        (status = 409, description = "Name taken", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn create_role(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<CreateRoleRequest>,
) -> Result<(StatusCode, Json<RoleResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    let _actor = require_role_manager(&state, community_id, user.account_id, &request_id).await?;
    let name = body.name.trim().to_owned();
    if name.is_empty() {
        return Err(validation(request_id, "Name is required."));
    }
    let role = persist_create(
        &state.pool,
        community_id,
        CreateRoleInput {
            name,
            color: body
                .color
                .unwrap_or_else(|| "141 152 173".to_owned())
                .trim()
                .to_owned(),
            hoist: body.hoist.unwrap_or(false),
            mentionable: body.mentionable.unwrap_or(false),
            permissions: body
                .permissions
                .unwrap_or_else(|| permissions_with_manage_roles(body.manage_roles.unwrap_or(false))),
            weight: body.weight,
            group_id: body.group_id,
            short_tag: body.short_tag.unwrap_or_default().trim().to_owned(),
            icon_emoji: body.icon_emoji,
            gradient: body.gradient,
            role_card: body.role_card.unwrap_or_else(|| serde_json::json!({})),
        },
    )
    .await
    .map_err(|error| map_auth(&error, request_id))?;
    let response = to_response(&role);
    invalidate_community(&state, community_id);
    broadcast_role_create(&state, &response).await;
    Ok((StatusCode::CREATED, Json(response)))
}

/// List roles in a community.
#[utoipa::path(
    get,
    path = "/api/v1/communities/{community_id}/roles",
    operation_id = "listRoles",
    tag = "roles",
    params(("community_id" = Uuid, Path, description = "Community id")),
    responses(
        (status = 200, description = "Role list", body = RoleListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not a member", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn list_roles(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
) -> Result<Json<RoleListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_member(&state, community_id, user.account_id, &request_id).await?;
    let roles = persist_list(&state.pool, community_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list roles failed");
            internal(request_id)
        })?;
    Ok(Json(RoleListResponse {
        roles: roles.iter().map(to_response).collect(),
    }))
}

/// Reorder custom roles (owner / role managers).
#[utoipa::path(
    post,
    path = "/api/v1/communities/{community_id}/roles/reorder",
    operation_id = "reorderRoles",
    tag = "roles",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body = ReorderRolesRequest,
    responses(
        (status = 200, description = "Reordered", body = RoleListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn reorder_roles(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<ReorderRolesRequest>,
) -> Result<Json<RoleListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let actor = require_role_manager(&state, community_id, user.account_id, &request_id).await?;
    for (index, role_id) in body.role_ids.iter().enumerate() {
        let current = get_role(&state.pool, *role_id)
            .await
            .map_err(|error| map_auth(&error, request_id.clone()))?
            .ok_or_else(|| not_found(request_id.clone()))?;
        if current.community_id != community_id {
            return Err(not_found(request_id));
        }
        if current.is_everyone {
            continue;
        }
        if !can_manage_role_weight(actor, current.weight) {
            return Err(hierarchy_denied(request_id.clone()));
        }
        let position = i32::try_from(index)
            .map_err(|_| validation(request_id.clone(), "Role order index is out of range."))?;
        let updated = persist_update(
            &state.pool,
            *role_id,
            RolePatch {
                position: Some(position),
                ..RolePatch::default()
            },
        )
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
        broadcast_role_update(&state, &to_response(&updated)).await;
    }
    let roles = persist_list(&state.pool, community_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list roles failed");
            internal(request_id)
        })?;
    invalidate_community(&state, community_id);
    Ok(Json(RoleListResponse {
        roles: roles.iter().map(to_response).collect(),
    }))
}

/// Get one role.
#[utoipa::path(
    get,
    path = "/api/v1/roles/{role_id}",
    operation_id = "getRole",
    tag = "roles",
    params(("role_id" = Uuid, Path, description = "Role id")),
    responses(
        (status = 200, description = "Role", body = RoleResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not a member", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_role_by_id(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(role_id): Path<Uuid>,
) -> Result<Json<RoleResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let role = get_role(&state.pool, role_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "get role failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| not_found(request_id.clone()))?;
    require_member(&state, role.community_id, user.account_id, &request_id).await?;
    Ok(Json(to_response(&role)))
}

/// Update a role.
#[utoipa::path(
    patch,
    path = "/api/v1/roles/{role_id}",
    operation_id = "updateRole",
    tag = "roles",
    params(("role_id" = Uuid, Path, description = "Role id")),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Updated role", body = RoleResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn update_role(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(role_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateRoleRequest>,
) -> Result<Json<RoleResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = get_role(&state.pool, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    let actor =
        require_role_manager(&state, current.community_id, user.account_id, &request_id).await?;
    if !can_manage_role_weight(actor, current.weight) {
        return Err(hierarchy_denied(request_id.clone()));
    }
    if body.name.is_none()
        && body.position.is_none()
        && body.weight.is_none()
        && body.group_id.is_none()
        && body.clear_group.is_none()
        && body.color.is_none()
        && body.hoist.is_none()
        && body.mentionable.is_none()
        && body.manage_roles.is_none()
        && body.permissions.is_none()
        && body.short_tag.is_none()
        && body.icon_emoji.is_none()
        && body.clear_icon_emoji.is_none()
        && body.gradient.is_none()
        && body.clear_gradient.is_none()
        && body.role_card.is_none()
    {
        return Err(validation(
            request_id,
            "Provide at least one field to update.",
        ));
    }
    let permissions = body
        .permissions
        .or_else(|| body.manage_roles.map(permissions_with_manage_roles));
    let name = if current.is_everyone {
        None
    } else {
        body.name.map(|value| value.trim().to_owned())
    };
    let group_id = if body.clear_group == Some(true) {
        Some(None)
    } else {
        body.group_id.map(Some)
    };
    let icon_emoji = if body.clear_icon_emoji == Some(true) {
        Some(None)
    } else {
        body.icon_emoji.map(Some)
    };
    let gradient = if body.clear_gradient == Some(true) {
        Some(None)
    } else {
        body.gradient.map(Some)
    };
    let role = persist_update(
        &state.pool,
        role_id,
        RolePatch {
            name,
            position: if current.is_everyone {
                None
            } else {
                body.position
            },
            weight: if current.is_everyone {
                None
            } else {
                body.weight
            },
            group_id: if current.is_everyone { None } else { group_id },
            color: body.color.map(|value| value.trim().to_owned()),
            hoist: body.hoist,
            mentionable: body.mentionable,
            permissions,
            short_tag: body.short_tag.map(|value| value.trim().to_owned()),
            icon_emoji,
            icon_object_key: None,
            gradient,
            role_card: body.role_card,
        },
    )
    .await
    .map_err(|error| map_auth(&error, request_id))?;
    let response = to_response(&role);
    invalidate_community(&state, current.community_id);
    broadcast_role_update(&state, &response).await;
    Ok(Json(response))
}

/// Delete a custom role.
#[utoipa::path(
    delete,
    path = "/api/v1/roles/{role_id}",
    operation_id = "deleteRole",
    tag = "roles",
    params(("role_id" = Uuid, Path, description = "Role id")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn delete_role(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(role_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = get_role(&state.pool, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    let actor =
        require_role_manager(&state, current.community_id, user.account_id, &request_id).await?;
    if !can_manage_role_weight(actor, current.weight) {
        return Err(hierarchy_denied(request_id.clone()));
    }
    let deleted = persist_delete(&state.pool, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    if !deleted {
        return Err(not_found(request_id));
    }
    invalidate_community(&state, current.community_id);
    broadcast_role_delete(&state, current.community_id, role_id).await;
    Ok(StatusCode::NO_CONTENT)
}

/// Clone a role shell.
#[utoipa::path(
    post,
    path = "/api/v1/roles/{role_id}/clone",
    operation_id = "cloneRole",
    tag = "roles",
    params(("role_id" = Uuid, Path, description = "Role id")),
    responses(
        (status = 201, description = "Cloned role", body = RoleResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn clone_role(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(role_id): Path<Uuid>,
) -> Result<(StatusCode, Json<RoleResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = get_role(&state.pool, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    let actor =
        require_role_manager(&state, current.community_id, user.account_id, &request_id).await?;
    if !can_manage_role_weight(actor, current.weight) {
        return Err(hierarchy_denied(request_id));
    }
    let role = persist_clone(&state.pool, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id))?;
    let response = to_response(&role);
    invalidate_community(&state, current.community_id);
    broadcast_role_create(&state, &response).await;
    Ok((StatusCode::CREATED, Json(response)))
}

/// List roles assigned to a member.
#[utoipa::path(
    get,
    path = "/api/v1/communities/{community_id}/members/{account_id}/roles",
    operation_id = "listMemberRoles",
    tag = "roles",
    params(
        ("community_id" = Uuid, Path, description = "Community id"),
        ("account_id" = Uuid, Path, description = "Account id")
    ),
    responses(
        (status = 200, description = "Member roles", body = RoleListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not a member", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn list_member_roles(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path((community_id, account_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<RoleListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_member(&state, community_id, user.account_id, &request_id).await?;
    let roles = persist_list_member_roles(&state.pool, community_id, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list member roles failed");
            internal(request_id)
        })?;
    Ok(Json(RoleListResponse {
        roles: roles.iter().map(to_response).collect(),
    }))
}

/// Assign a role to a member.
#[utoipa::path(
    post,
    path = "/api/v1/communities/{community_id}/members/{account_id}/roles",
    operation_id = "assignMemberRole",
    tag = "roles",
    params(
        ("community_id" = Uuid, Path, description = "Community id"),
        ("account_id" = Uuid, Path, description = "Account id")
    ),
    request_body = AssignRoleRequest,
    responses(
        (status = 204, description = "Assigned"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn assign_member_role(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path((community_id, account_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(body): ValidatedJson<AssignRoleRequest>,
) -> Result<StatusCode, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let actor = require_role_manager(&state, community_id, user.account_id, &request_id).await?;
    let target_role = get_role(&state.pool, body.role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    if target_role.community_id != community_id {
        return Err(not_found(request_id));
    }
    if !can_manage_role_weight(actor, target_role.weight) {
        return Err(hierarchy_denied(request_id.clone()));
    }
    persist_assign(&state.pool, community_id, account_id, body.role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    invalidate_community(&state, community_id);
    broadcast_member_roles(&state, community_id, account_id).await;
    Ok(StatusCode::NO_CONTENT)
}

/// Remove a role from a member.
#[utoipa::path(
    delete,
    path = "/api/v1/communities/{community_id}/members/{account_id}/roles/{role_id}",
    operation_id = "removeMemberRole",
    tag = "roles",
    params(
        ("community_id" = Uuid, Path, description = "Community id"),
        ("account_id" = Uuid, Path, description = "Account id"),
        ("role_id" = Uuid, Path, description = "Role id")
    ),
    responses(
        (status = 204, description = "Removed"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn remove_member_role(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path((community_id, account_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let actor = require_role_manager(&state, community_id, user.account_id, &request_id).await?;
    let target_role = get_role(&state.pool, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    if target_role.community_id != community_id {
        return Err(not_found(request_id));
    }
    if !can_manage_role_weight(actor, target_role.weight) {
        return Err(hierarchy_denied(request_id.clone()));
    }
    let removed = remove_role_assignment(&state.pool, community_id, account_id, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    if !removed {
        return Err(not_found(request_id));
    }
    invalidate_community(&state, community_id);
    broadcast_member_roles(&state, community_id, account_id).await;
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
    Err(ApiError::permission_denied(
        request_id.to_owned(),
        "You must be a community member to view roles.",
    ))
}

async fn require_role_manager(
    state: &AppState,
    community_id: Uuid,
    account_id: Uuid,
    request_id: &str,
) -> Result<RoleActor, ApiError> {
    require_member(state, community_id, account_id, request_id).await?;
    require_permission(
        state,
        community_id,
        account_id,
        None,
        PermissionCode::COMMUNITY_MANAGE_ROLES,
        request_id.to_owned(),
        "You do not have permission to manage roles.",
    )
    .await?;
    let actor = role_actor(&state.pool, community_id, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "role actor lookup failed");
            internal(request_id.to_owned())
        })?;
    if actor.is_owner {
        return Ok(actor);
    }
    if crate::permissions::allowed(
        state,
        community_id,
        account_id,
        None,
        PermissionCode::COMMUNITY_ADMINISTRATOR,
    )
    .await?
    {
        return Ok(RoleActor {
            is_owner: false,
            can_manage_roles: true,
            min_weight: 0,
        });
    }
    Ok(actor)
}

fn to_response(role: &CommunityRole) -> RoleResponse {
    RoleResponse {
        id: role.id,
        community_id: role.community_id,
        name: role.name.clone(),
        position: role.position,
        weight: role.weight,
        group_id: role.group_id,
        color: role.color.clone(),
        hoist: role.hoist,
        mentionable: role.mentionable,
        permissions: role.permissions.clone(),
        is_everyone: role.is_everyone,
        short_tag: role.short_tag.clone(),
        icon_emoji: role.icon_emoji.clone(),
        icon_object_key: role.icon_object_key.clone(),
        gradient: role.gradient.clone(),
        role_card: role.role_card.clone(),
        created_at: role.created_at,
        updated_at: role.updated_at,
    }
}

fn to_payload(role: &RoleResponse) -> CommunityRolePayload {
    CommunityRolePayload {
        id: role.id,
        community_id: role.community_id,
        name: role.name.clone(),
        position: role.position,
        weight: role.weight,
        group_id: role.group_id,
        color: role.color.clone(),
        hoist: role.hoist,
        mentionable: role.mentionable,
        permissions: role.permissions.clone(),
        is_everyone: role.is_everyone,
        short_tag: role.short_tag.clone(),
        icon_emoji: role.icon_emoji.clone(),
        icon_object_key: role.icon_object_key.clone(),
        gradient: role.gradient.clone(),
        role_card: role.role_card.clone(),
    }
}

async fn broadcast_role_create(state: &AppState, role: &RoleResponse) {
    let community_id = role.community_id;
    let payload = to_payload(role);
    let recipients = list_member_account_ids(&state.pool, community_id)
        .await
        .unwrap_or_default();
    state
        .presence_hub
        .broadcast_to_accounts(&recipients, PresenceHubMessage::RoleCreate(payload))
        .await;
}

async fn broadcast_role_update(state: &AppState, role: &RoleResponse) {
    let community_id = role.community_id;
    let payload = to_payload(role);
    let recipients = list_member_account_ids(&state.pool, community_id)
        .await
        .unwrap_or_default();
    state
        .presence_hub
        .broadcast_to_accounts(&recipients, PresenceHubMessage::RoleUpdate(payload))
        .await;
}

async fn broadcast_role_delete(state: &AppState, community_id: Uuid, role_id: Uuid) {
    let payload = RoleDeletePayload {
        community_id,
        role_id,
    };
    let recipients = list_member_account_ids(&state.pool, community_id)
        .await
        .unwrap_or_default();
    state
        .presence_hub
        .broadcast_to_accounts(&recipients, PresenceHubMessage::RoleDelete(payload))
        .await;
}

async fn broadcast_member_roles(state: &AppState, community_id: Uuid, account_id: Uuid) {
    let roles = persist_list_member_roles(&state.pool, community_id, account_id)
        .await
        .unwrap_or_default();
    let role_ids: Vec<Uuid> = roles.iter().map(|role| role.id).collect();
    let payload = MemberRoleUpdatePayload {
        community_id,
        account_id,
        role_ids,
    };
    let recipients = list_member_account_ids(&state.pool, community_id)
        .await
        .unwrap_or_default();
    state
        .presence_hub
        .broadcast_to_accounts(&recipients, PresenceHubMessage::MemberRoleUpdate(payload))
        .await;
}

/// Create a role group.
#[utoipa::path(
    post,
    path = "/api/v1/communities/{community_id}/role-groups",
    operation_id = "createRoleGroup",
    tag = "roles",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body = CreateRoleGroupRequest,
    responses(
        (status = 201, description = "Group created", body = RoleGroupResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 409, description = "Name taken", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn create_role_group(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<CreateRoleGroupRequest>,
) -> Result<(StatusCode, Json<RoleGroupResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    let _actor = require_role_manager(&state, community_id, user.account_id, &request_id).await?;
    let name = body.name.trim().to_owned();
    if name.is_empty() {
        return Err(validation(request_id, "Name is required."));
    }
    let group = voxnexus_auth::create_role_group(&state.pool, community_id, name)
        .await
        .map_err(|error| map_auth(&error, request_id))?;
    Ok((StatusCode::CREATED, Json(group_to_response(&group))))
}

/// List role groups.
#[utoipa::path(
    get,
    path = "/api/v1/communities/{community_id}/role-groups",
    operation_id = "listRoleGroups",
    tag = "roles",
    params(("community_id" = Uuid, Path, description = "Community id")),
    responses(
        (status = 200, description = "Group list", body = RoleGroupListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not a member", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn list_role_groups(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
) -> Result<Json<RoleGroupListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_member(&state, community_id, user.account_id, &request_id).await?;
    let groups = voxnexus_auth::list_role_groups(&state.pool, community_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list role groups failed");
            internal(request_id)
        })?;
    Ok(Json(RoleGroupListResponse {
        groups: groups.iter().map(group_to_response).collect(),
    }))
}

/// Update a role group.
#[utoipa::path(
    patch,
    path = "/api/v1/role-groups/{group_id}",
    operation_id = "updateRoleGroup",
    tag = "roles",
    params(("group_id" = Uuid, Path, description = "Group id")),
    request_body = UpdateRoleGroupRequest,
    responses(
        (status = 200, description = "Updated group", body = RoleGroupResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn update_role_group(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(group_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateRoleGroupRequest>,
) -> Result<Json<RoleGroupResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = voxnexus_auth::get_role_group(&state.pool, group_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    let _actor =
        require_role_manager(&state, current.community_id, user.account_id, &request_id).await?;
    let group = voxnexus_auth::update_role_group(
        &state.pool,
        group_id,
        body.name.map(|value| value.trim().to_owned()),
        body.display_order,
    )
    .await
    .map_err(|error| map_auth(&error, request_id))?;
    Ok(Json(group_to_response(&group)))
}

/// Delete a role group (roles become ungrouped).
#[utoipa::path(
    delete,
    path = "/api/v1/role-groups/{group_id}",
    operation_id = "deleteRoleGroup",
    tag = "roles",
    params(("group_id" = Uuid, Path, description = "Group id")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn delete_role_group(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(group_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = voxnexus_auth::get_role_group(&state.pool, group_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    let _actor =
        require_role_manager(&state, current.community_id, user.account_id, &request_id).await?;
    let deleted = voxnexus_auth::delete_role_group(&state.pool, group_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    if !deleted {
        return Err(not_found(request_id));
    }
    invalidate_community(&state, current.community_id);
    Ok(StatusCode::NO_CONTENT)
}

/// Bulk move roles into a group (or ungroup).
#[utoipa::path(
    post,
    path = "/api/v1/communities/{community_id}/role-groups/bulk-assign",
    operation_id = "bulkAssignRoleGroup",
    tag = "roles",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body = BulkAssignRoleGroupRequest,
    responses(
        (status = 200, description = "Updated roles", body = RoleListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn bulk_assign_role_group(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<BulkAssignRoleGroupRequest>,
) -> Result<Json<RoleListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let actor = require_role_manager(&state, community_id, user.account_id, &request_id).await?;
    if let Some(group_id) = body.group_id {
        let group = voxnexus_auth::get_role_group(&state.pool, group_id)
            .await
            .map_err(|error| map_auth(&error, request_id.clone()))?
            .ok_or_else(|| not_found(request_id.clone()))?;
        if group.community_id != community_id {
            return Err(not_found(request_id));
        }
    }
    for role_id in &body.role_ids {
        let current = get_role(&state.pool, *role_id)
            .await
            .map_err(|error| map_auth(&error, request_id.clone()))?
            .ok_or_else(|| not_found(request_id.clone()))?;
        if current.community_id != community_id || current.is_everyone {
            return Err(validation(
                request_id.clone(),
                "Cannot assign @everyone or foreign roles to a group.",
            ));
        }
        if !can_manage_role_weight(actor, current.weight) {
            return Err(hierarchy_denied(request_id.clone()));
        }
        let updated = persist_update(
            &state.pool,
            *role_id,
            RolePatch {
                group_id: Some(body.group_id),
                ..RolePatch::default()
            },
        )
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
        broadcast_role_update(&state, &to_response(&updated)).await;
    }
    invalidate_community(&state, community_id);
    let roles = persist_list(&state.pool, community_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list roles failed");
            internal(request_id)
        })?;
    Ok(Json(RoleListResponse {
        roles: roles.iter().map(to_response).collect(),
    }))
}

/// Upload a custom role icon (JPEG/PNG/GIF/WebP).
#[utoipa::path(
    post,
    path = "/api/v1/roles/{role_id}/icon",
    operation_id = "uploadRoleIcon",
    tag = "roles",
    params(("role_id" = Uuid, Path, description = "Role id")),
    request_body(content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "Updated role", body = RoleResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody),
        (status = 422, description = "Validation error", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn upload_role_icon(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(role_id): Path<Uuid>,
    body: Bytes,
) -> Result<Json<RoleResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = get_role(&state.pool, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    let actor =
        require_role_manager(&state, current.community_id, user.account_id, &request_id).await?;
    if !can_manage_role_weight(actor, current.weight) {
        return Err(hierarchy_denied(request_id.clone()));
    }
    if body.len() > AVATAR_MAX_BYTES {
        return Err(validation(
            request_id,
            format!("Image exceeds maximum size of {AVATAR_MAX_BYTES} bytes."),
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
        "vn/role-icons/{}/{object_id}.{}",
        current.community_id,
        kind.extension()
    );
    let key = ObjectKey::parse(&key_str).map_err(|_| internal(request_id.clone()))?;
    let digest = Sha256::digest(&body);
    state
        .storage
        .put(key, body.clone(), kind.mime())
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "role icon put failed");
            internal(request_id.clone())
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
    .map_err(|error| map_auth(&error, request_id.clone()))?;

    let previous = set_role_icon(&state.pool, role_id, Some(object_id))
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    if let Some(old_key) = previous {
        if let Ok(old_id) = Uuid::parse_str(&old_key) {
            if let Ok(Some(old)) = get_object(&state.pool, old_id).await {
                if let Ok(parsed) = ObjectKey::parse(&old.storage_key) {
                    let _ = state.storage.delete(&parsed).await;
                }
                let _ = delete_object_meta(&state.pool, old_id).await;
            }
        }
    }

    let role = get_role(&state.pool, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id))?;
    let response = to_response(&role);
    invalidate_community(&state, current.community_id);
    broadcast_role_update(&state, &response).await;
    Ok(Json(response))
}

/// Serve a role's custom icon.
#[utoipa::path(
    get,
    path = "/api/v1/roles/{role_id}/icon",
    operation_id = "getRoleIcon",
    tag = "roles",
    params(("role_id" = Uuid, Path, description = "Role id")),
    responses(
        (status = 200, description = "Image bytes"),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_role_icon(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(role_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let role = get_role(&state.pool, role_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "get role for icon failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| not_found(request_id.clone()))?;
    let object_key = role
        .icon_object_key
        .ok_or_else(|| not_found(request_id.clone()))?;
    let object_id = Uuid::parse_str(&object_key).map_err(|_| not_found(request_id.clone()))?;
    let meta = get_object(&state.pool, object_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    let key = ObjectKey::parse(&meta.storage_key).map_err(|_| internal(request_id.clone()))?;
    let bytes = state.storage.get(&key).await.map_err(|error| {
        tracing::error!(error = %error, "role icon get failed");
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

/// Clear a role's custom icon.
#[utoipa::path(
    delete,
    path = "/api/v1/roles/{role_id}/icon",
    operation_id = "deleteRoleIcon",
    tag = "roles",
    params(("role_id" = Uuid, Path, description = "Role id")),
    responses(
        (status = 200, description = "Updated role", body = RoleResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn delete_role_icon(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(role_id): Path<Uuid>,
) -> Result<Json<RoleResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = get_role(&state.pool, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    let actor =
        require_role_manager(&state, current.community_id, user.account_id, &request_id).await?;
    if !can_manage_role_weight(actor, current.weight) {
        return Err(hierarchy_denied(request_id.clone()));
    }
    let previous = set_role_icon(&state.pool, role_id, None)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    if let Some(old_key) = previous {
        if let Ok(old_id) = Uuid::parse_str(&old_key) {
            if let Ok(Some(old)) = get_object(&state.pool, old_id).await {
                if let Ok(parsed) = ObjectKey::parse(&old.storage_key) {
                    let _ = state.storage.delete(&parsed).await;
                }
                let _ = delete_object_meta(&state.pool, old_id).await;
            }
        }
    }
    let role = get_role(&state.pool, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id))?;
    let response = to_response(&role);
    invalidate_community(&state, current.community_id);
    broadcast_role_update(&state, &response).await;
    Ok(Json(response))
}

fn group_to_response(group: &voxnexus_domain::CommunityRoleGroup) -> RoleGroupResponse {
    RoleGroupResponse {
        id: group.id,
        community_id: group.community_id,
        name: group.name.clone(),
        display_order: group.display_order,
        created_at: group.created_at,
        updated_at: group.updated_at,
    }
}

fn map_auth(error: &voxnexus_auth::AuthError, request_id: String) -> ApiError {
    match error {
        voxnexus_auth::AuthError::RoleNameTaken => ApiError::new(
            StatusCode::CONFLICT,
            error_codes::CONFLICT,
            "A role with this name already exists.",
            None,
            request_id,
        ),
        voxnexus_auth::AuthError::RoleWeightTaken => ApiError::new(
            StatusCode::CONFLICT,
            error_codes::CONFLICT,
            "A role with this weight already exists.",
            None,
            request_id,
        ),
        voxnexus_auth::AuthError::InvalidRoleWeight => validation(
            request_id,
            "Role weight must be between 1 and 1000.",
        ),
        voxnexus_auth::AuthError::RoleGroupNotFound => not_found(request_id),
        voxnexus_auth::AuthError::RoleGroupNameTaken => ApiError::new(
            StatusCode::CONFLICT,
            error_codes::CONFLICT,
            "A role group with this name already exists.",
            None,
            request_id,
        ),
        voxnexus_auth::AuthError::EveryoneRoleImmutable => {
            ApiError::permission_denied(request_id, "The @everyone role cannot be changed that way.")
        }
        voxnexus_auth::AuthError::RoleHierarchyDenied => hierarchy_denied(request_id),
        voxnexus_auth::AuthError::CannotManageRoles => cannot_manage_roles(request_id),
        voxnexus_auth::AuthError::RoleScopeMismatch => {
            ApiError::permission_denied(request_id, "Role does not belong to this community.")
        }
        other => {
            tracing::error!(error = %other, "role auth error");
            internal(request_id)
        }
    }
}

fn hierarchy_denied(request_id: String) -> ApiError {
    ApiError::permission_denied(
        request_id,
        "You cannot manage roles at or above your highest role.",
    )
}

fn cannot_manage_roles(request_id: String) -> ApiError {
    ApiError::permission_denied(request_id, "You cannot manage roles in this community.")
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
