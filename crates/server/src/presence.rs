//! Instance presence HTTP (F018).

#![allow(clippy::missing_errors_doc)]

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use voxnexus_protocol::{PresenceEntry, PresenceListResponse};

use crate::error::ApiError;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};

/// List accounts currently online on this instance (invisible omitted).
#[utoipa::path(
    get,
    path = "/api/v1/presence",
    operation_id = "listPresence",
    tag = "presence",
    responses(
        (status = 200, description = "Online accounts", body = PresenceListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn list_presence(
    State(state): State<AppState>,
    _user: AuthUser,
    headers: HeaderMap,
) -> Result<Json<PresenceListResponse>, ApiError> {
    let _request_id = request_id_from_headers(&headers);
    let presences = state.presence_hub.list_public().await;
    Ok(Json(PresenceListResponse {
        presences: presences
            .into_iter()
            .map(|entry| PresenceEntry {
                account_id: entry.account_id,
                status: entry.status,
                custom_status: entry.custom_status,
            })
            .collect(),
    }))
}
