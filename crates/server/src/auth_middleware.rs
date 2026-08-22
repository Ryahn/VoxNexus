//! Require a session cookie on protected `/api/v1` routes.

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::error::ApiError;
use crate::extract_auth::{is_public_api_path, resolve_auth_user};
use crate::http::{request_id_from_headers, AppState};

/// Insert [`crate::extract_auth::AuthUser`] for non-public `/api/v1` paths; otherwise `401`.
pub async fn require_api_session(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    if !path.starts_with("/api/v1") || is_public_api_path(path) {
        return next.run(request).await;
    }

    let request_id = request_id_from_headers(request.headers());
    match resolve_auth_user(&state, request.headers()).await {
        Some(user) => {
            request.extensions_mut().insert(user);
            next.run(request).await
        }
        None => ApiError::unauthenticated(request_id).into_response(),
    }
}
