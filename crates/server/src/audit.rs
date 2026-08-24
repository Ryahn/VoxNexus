//! Community audit log API (F033).

#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_many_arguments
)]

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;
use voxnexus_auth::{insert_audit_event, list_audit_events, NewAuditEvent};
use voxnexus_domain::AuditEvent;
use voxnexus_permissions::PermissionCode;
use voxnexus_protocol::{AuditEventListResponse, AuditEventResponse, ListAuditEventsQuery};

use crate::error::ApiError;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};
use crate::permissions::require_permission;

/// List community audit events (newest first). Requires `community.view_audit`.
#[utoipa::path(
    get,
    path = "/api/v1/communities/{community_id}/audit-events",
    operation_id = "listAuditEvents",
    tag = "audit",
    params(
        ("community_id" = Uuid, Path, description = "Community id"),
        ListAuditEventsQuery
    ),
    responses(
        (status = 200, description = "Audit page", body = AuditEventListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn list_community_audit_events(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    Query(query): Query<ListAuditEventsQuery>,
) -> Result<Json<AuditEventListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_permission(
        &state,
        community_id,
        user.account_id,
        None,
        PermissionCode::COMMUNITY_VIEW_AUDIT,
        request_id.clone(),
        "You do not have permission to view the audit log.",
    )
    .await?;

    let page = list_audit_events(
        &state.pool,
        community_id,
        query.after,
        query.before,
        query.resolved_limit(),
        query.actor_account_id,
        query.action.as_deref(),
        query.space_id,
    )
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "list audit events failed");
        ApiError::new(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            voxnexus_protocol::error_codes::INTERNAL,
            "Unexpected server error.",
            None,
            request_id,
        )
    })?;

    Ok(Json(AuditEventListResponse {
        items: page.items.iter().map(to_response).collect(),
        has_more: page.has_more,
    }))
}

/// Best-effort audit emit — never fails the calling handler.
pub async fn emit_audit(
    state: &AppState,
    community_id: Uuid,
    actor_account_id: Uuid,
    action: &str,
    summary: impl Into<String>,
    target_type: Option<&str>,
    target_id: Option<Uuid>,
    space_id: Option<Uuid>,
    metadata: Value,
) {
    let result = insert_audit_event(
        &state.pool,
        NewAuditEvent {
            community_id,
            actor_account_id: Some(actor_account_id),
            action: action.to_owned(),
            space_id,
            target_type: target_type.map(str::to_owned),
            target_id,
            summary: summary.into(),
            metadata,
        },
    )
    .await;
    if let Err(error) = result {
        tracing::error!(error = %error, action, community_id = %community_id, "audit emit failed");
    }
}

/// Convenience emit with empty metadata.
pub async fn emit_audit_simple(
    state: &AppState,
    community_id: Uuid,
    actor_account_id: Uuid,
    action: &str,
    summary: impl Into<String>,
    target_type: Option<&str>,
    target_id: Option<Uuid>,
) {
    emit_audit(
        state,
        community_id,
        actor_account_id,
        action,
        summary,
        target_type,
        target_id,
        None,
        json!({}),
    )
    .await;
}

fn to_response(event: &AuditEvent) -> AuditEventResponse {
    AuditEventResponse {
        id: event.id,
        community_id: event.community_id,
        actor_account_id: event.actor_account_id,
        action: event.action.clone(),
        space_id: event.space_id,
        target_type: event.target_type.clone(),
        target_id: event.target_id,
        summary: event.summary.clone(),
        metadata: event.metadata.clone(),
        created_at: event.created_at,
    }
}
