//! Community role CRUD and assignments (F028).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use uuid::Uuid;
use voxnexus_auth::{
    assign_role as persist_assign, can_manage_role_position, clone_role as persist_clone,
    create_role as persist_create, delete_role as persist_delete, get_membership, get_role,
    list_member_account_ids, list_member_roles as persist_list_member_roles,
    list_roles as persist_list, permissions_with_manage_roles, remove_role_assignment, role_actor,
    update_role as persist_update, CreateRoleInput, RoleActor, RolePatch,
};
use voxnexus_domain::CommunityRole;
use voxnexus_protocol::error_codes;
use voxnexus_protocol::{
    AssignRoleRequest, CommunityRolePayload, CreateRoleRequest, MemberRoleUpdatePayload,
    ReorderRolesRequest, RoleDeletePayload, RoleListResponse, RoleResponse, UpdateRoleRequest,
};
use voxnexus_realtime::PresenceHubMessage;

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};

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
    let actor = require_role_manager(&state, community_id, user.account_id, &request_id).await?;
    if !actor.is_owner && !actor.can_manage_roles {
        return Err(cannot_manage_roles(request_id));
    }
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
            permissions: permissions_with_manage_roles(body.manage_roles.unwrap_or(false)),
        },
    )
    .await
    .map_err(|error| map_auth(&error, request_id))?;
    let response = to_response(&role);
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
    if !actor.is_owner && !actor.can_manage_roles {
        return Err(cannot_manage_roles(request_id));
    }
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
        if !can_manage_role_position(actor, current.position) {
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
    if !can_manage_role_position(actor, current.position) {
        return Err(hierarchy_denied(request_id.clone()));
    }
    if body.name.is_none()
        && body.position.is_none()
        && body.color.is_none()
        && body.hoist.is_none()
        && body.mentionable.is_none()
        && body.manage_roles.is_none()
    {
        return Err(validation(
            request_id,
            "Provide at least one field to update.",
        ));
    }
    let permissions = body.manage_roles.map(permissions_with_manage_roles);
    let name = if current.is_everyone {
        None
    } else {
        body.name.map(|value| value.trim().to_owned())
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
            color: body.color.map(|value| value.trim().to_owned()),
            hoist: body.hoist,
            mentionable: body.mentionable,
            permissions,
        },
    )
    .await
    .map_err(|error| map_auth(&error, request_id))?;
    let response = to_response(&role);
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
    if !can_manage_role_position(actor, current.position) {
        return Err(hierarchy_denied(request_id.clone()));
    }
    let deleted = persist_delete(&state.pool, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    if !deleted {
        return Err(not_found(request_id));
    }
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
    if !can_manage_role_position(actor, current.position) {
        return Err(hierarchy_denied(request_id));
    }
    let role = persist_clone(&state.pool, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id))?;
    let response = to_response(&role);
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
    if !can_manage_role_position(actor, target_role.position) {
        return Err(hierarchy_denied(request_id.clone()));
    }
    persist_assign(&state.pool, community_id, account_id, body.role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
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
    if !can_manage_role_position(actor, target_role.position) {
        return Err(hierarchy_denied(request_id.clone()));
    }
    let removed = remove_role_assignment(&state.pool, community_id, account_id, role_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    if !removed {
        return Err(not_found(request_id));
    }
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
    role_actor(&state.pool, community_id, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "role actor lookup failed");
            internal(request_id.to_owned())
        })
}

fn to_response(role: &CommunityRole) -> RoleResponse {
    RoleResponse {
        id: role.id,
        community_id: role.community_id,
        name: role.name.clone(),
        position: role.position,
        color: role.color.clone(),
        hoist: role.hoist,
        mentionable: role.mentionable,
        permissions: role.permissions.clone(),
        is_everyone: role.is_everyone,
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
        color: role.color.clone(),
        hoist: role.hoist,
        mentionable: role.mentionable,
        permissions: role.permissions.clone(),
        is_everyone: role.is_everyone,
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

fn map_auth(error: &voxnexus_auth::AuthError, request_id: String) -> ApiError {
    match error {
        voxnexus_auth::AuthError::RoleNameTaken => ApiError::new(
            StatusCode::CONFLICT,
            error_codes::CONFLICT,
            "A role with this name already exists.",
            None,
            request_id,
        ),
        voxnexus_auth::AuthError::EveryoneRoleImmutable => {
            ApiError::permission_denied(request_id, "The @everyone role cannot be deleted.")
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
