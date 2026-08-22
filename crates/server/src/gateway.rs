//! WebSocket gateway upgrade (`GET /api/v1/gateway`).

use axum::body::Body;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{FromRequestParts, State};
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use voxnexus_protocol::GATEWAY_SUBPROTOCOL;
use voxnexus_realtime::{run_session, GatewaySessionOptions};

use crate::error::ApiError;
use crate::http::{request_id_from_headers, AppState};

/// Upgrade to the gateway WebSocket when `GATEWAY_ALLOW_UNAUTH` is enabled.
///
/// Auth is checked before the WebSocket upgrade extractor so disabled gateways
/// return `503 gateway_unavailable` instead of `426` from a failed upgrade parse.
pub async fn gateway_upgrade(State(state): State<AppState>, request: Request<Body>) -> Response {
    let request_id = request_id_from_headers(request.headers());
    if !state.gateway_allow_unauth {
        return ApiError::gateway_unavailable(request_id).into_response();
    }

    let (mut parts, _body) = request.into_parts();
    let ws = match WebSocketUpgrade::from_request_parts(&mut parts, &state).await {
        Ok(ws) => ws,
        Err(rejection) => return rejection.into_response(),
    };

    let options = GatewaySessionOptions {
        heartbeat_interval: state.gateway_heartbeat_interval,
        allow_dev_ping: true,
    };

    ws.protocols([GATEWAY_SUBPROTOCOL])
        .on_upgrade(move |socket| async move {
            run_session(socket, options).await;
        })
}
