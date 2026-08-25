//! WebSocket gateway upgrade (`GET /api/v1/gateway`).

use std::sync::Arc;

use axum::body::Body;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{FromRequestParts, State};
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use voxnexus_auth::{ensure_profile, update_profile};
use voxnexus_domain::PresenceStatus;
use voxnexus_protocol::GATEWAY_SUBPROTOCOL;
use voxnexus_realtime::{run_session, GatewaySessionOptions};

use crate::error::ApiError;
use crate::extract_auth::resolve_auth_user;
use crate::http::{request_id_from_headers, AppState};
use crate::typing::handle_typing_start;

/// Upgrade to the gateway WebSocket when a valid session cookie is present.
///
/// Cookie auth is checked before the WebSocket upgrade extractor so missing
/// sessions return `401` instead of `426` from a failed upgrade parse.
pub async fn gateway_upgrade(State(state): State<AppState>, request: Request<Body>) -> Response {
    let request_id = request_id_from_headers(request.headers());
    let Some(user) = resolve_auth_user(&state, request.headers()).await else {
        return ApiError::unauthenticated(request_id).into_response();
    };

    let profile = match ensure_profile(&state.pool, user.account_id).await {
        Ok(profile) => profile,
        Err(error) => {
            tracing::error!(error = %error, "gateway profile load failed");
            return ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                voxnexus_protocol::error_codes::INTERNAL,
                "Unexpected server error.",
                None,
                request_id,
            )
            .into_response();
        }
    };

    let (mut parts, _body) = request.into_parts();
    let ws = match WebSocketUpgrade::from_request_parts(&mut parts, &state).await {
        Ok(ws) => ws,
        Err(rejection) => return rejection.into_response(),
    };

    let pool = state.pool.clone();
    let on_presence_change = Arc::new(
        move |account_id: uuid::Uuid,
              status: Option<PresenceStatus>,
              custom_status: Option<String>| {
            let pool = pool.clone();
            tokio::spawn(async move {
                let _ = update_profile(
                    &pool,
                    account_id,
                    None,
                    None,
                    status,
                    custom_status.as_deref(),
                )
                .await;
            });
        },
    );

    let typing_state = state.clone();
    let on_typing_start = Arc::new(move |account_id: uuid::Uuid, channel_id: uuid::Uuid| {
        let state = typing_state.clone();
        tokio::spawn(async move {
            handle_typing_start(&state, account_id, channel_id).await;
        });
    });

    let options = GatewaySessionOptions {
        heartbeat_interval: state.gateway_heartbeat_interval,
        account_id: user.account_id,
        allow_dev_ping: state.gateway_allow_unauth,
        resume_store: Arc::clone(&state.resume_store),
        presence_hub: Arc::clone(&state.presence_hub),
        stored_presence: profile.presence_status,
        stored_custom_status: profile.custom_status.clone(),
        on_presence_change: Some(on_presence_change),
        on_typing_start: Some(on_typing_start),
    };

    ws.protocols([GATEWAY_SUBPROTOCOL])
        .on_upgrade(move |socket| async move {
            run_session(socket, options).await;
        })
}
