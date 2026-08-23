//! Instance settings HTTP handlers (F017).

#![allow(clippy::missing_errors_doc)]

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use voxnexus_auth::{get_instance, update_instance, InstanceError, InstancePatch};
use voxnexus_protocol::{InstanceSettingsResponse, UpdateInstanceSettingsRequest};

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::extract_auth::InstanceAdmin;
use crate::http::{request_id_from_headers, AppState};

fn map_instance_error(error: InstanceError, request_id: String) -> ApiError {
    match error {
        InstanceError::InvalidRow => ApiError::new(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            voxnexus_protocol::error_codes::INTERNAL,
            "Instance settings are invalid in the database.",
            None,
            request_id,
        ),
        InstanceError::Db(db) => {
            tracing::error!(error = %db, "instance settings database error");
            ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                voxnexus_protocol::error_codes::INTERNAL,
                "Unexpected server error.",
                None,
                request_id,
            )
        }
    }
}

fn to_response(
    instance: voxnexus_domain::Instance,
    community_creation_mode_locked: bool,
) -> InstanceSettingsResponse {
    InstanceSettingsResponse {
        id: instance.id,
        name: instance.name,
        public_url: instance.public_url,
        registration_mode: instance.registration_mode,
        community_creation_mode: instance.community_creation_mode,
        community_creation_mode_locked,
        oidc_enabled: instance.oidc_enabled,
        oidc_issuer: instance.oidc_issuer,
        oidc_client_id: instance.oidc_client_id,
        updated_at: instance.updated_at,
    }
}

/// Read instance settings (instance admin only).
#[utoipa::path(
    get,
    path = "/api/v1/instance/settings",
    operation_id = "getInstanceSettings",
    tag = "instance",
    responses(
        (status = 200, description = "Instance settings", body = InstanceSettingsResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not instance admin", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_instance_settings(
    State(state): State<AppState>,
    InstanceAdmin(_admin): InstanceAdmin,
    headers: HeaderMap,
) -> Result<Json<InstanceSettingsResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let instance = get_instance(&state.pool)
        .await
        .map_err(|error| map_instance_error(error, request_id.clone()))?;
    Ok(Json(to_response(
        instance,
        state.community_creation_mode_locked,
    )))
}

/// Update instance settings (instance admin only).
#[utoipa::path(
    patch,
    path = "/api/v1/instance/settings",
    operation_id = "updateInstanceSettings",
    tag = "instance",
    request_body = UpdateInstanceSettingsRequest,
    responses(
        (status = 200, description = "Updated instance settings", body = InstanceSettingsResponse),
        (status = 400, description = "Validation error", body = voxnexus_protocol::ErrorBody),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not instance admin", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn update_instance_settings(
    State(state): State<AppState>,
    InstanceAdmin(_admin): InstanceAdmin,
    headers: HeaderMap,
    ValidatedJson(body): ValidatedJson<UpdateInstanceSettingsRequest>,
) -> Result<Json<InstanceSettingsResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let community_creation_mode = if state.community_creation_mode_locked {
        None
    } else {
        body.community_creation_mode
    };
    let patch = InstancePatch {
        name: body.name,
        public_url: body.public_url,
        registration_mode: body.registration_mode,
        community_creation_mode,
        oidc_enabled: body.oidc_enabled,
        oidc_issuer: body.oidc_issuer,
        oidc_client_id: body.oidc_client_id,
    };
    let instance = update_instance(&state.pool, patch)
        .await
        .map_err(|error| map_instance_error(error, request_id.clone()))?;
    Ok(Json(to_response(
        instance,
        state.community_creation_mode_locked,
    )))
}
