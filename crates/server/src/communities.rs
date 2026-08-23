//! Community creation policy gate (stub until F019).

#![allow(clippy::missing_errors_doc)]

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use voxnexus_auth::get_instance;
use voxnexus_protocol::CommunityCreateAcceptedResponse;

use crate::error::ApiError;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};

/// Policy check for user-driven community creation (F017 stub; F019 persists communities).
#[utoipa::path(
    post,
    path = "/api/v1/communities",
    operation_id = "createCommunity",
    tag = "communities",
    responses(
        (status = 202, description = "Creation allowed (stub)", body = CommunityCreateAcceptedResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Creation disallowed by instance policy", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn create_community(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<CommunityCreateAcceptedResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    let instance = get_instance(&state.pool).await.map_err(|error| {
        tracing::error!(error = %error, "instance lookup failed");
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            voxnexus_protocol::error_codes::INTERNAL,
            "Unexpected server error.",
            None,
            request_id.clone(),
        )
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
    Ok((
        StatusCode::ACCEPTED,
        Json(CommunityCreateAcceptedResponse { accepted: true }),
    ))
}
