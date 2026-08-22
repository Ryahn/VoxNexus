//! WebSocket gateway upgrade (`GET /api/v1/gateway`).

use axum::extract::ws::WebSocketUpgrade;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::response::Response;
use voxnexus_protocol::GATEWAY_SUBPROTOCOL;
use voxnexus_realtime::{run_session, GatewaySessionOptions};

use crate::error::ApiError;
use crate::http::{request_id_from_headers, AppState};

/// Upgrade to the gateway WebSocket when `GATEWAY_ALLOW_UNAUTH` is enabled.
pub async fn gateway_upgrade(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let request_id = request_id_from_headers(&headers);
    if !state.gateway_allow_unauth {
        return ApiError::gateway_unavailable(request_id).into_response();
    }

    let options = GatewaySessionOptions {
        heartbeat_interval: state.gateway_heartbeat_interval,
        allow_dev_ping: true,
    };

    ws.protocols([GATEWAY_SUBPROTOCOL])
        .on_upgrade(move |socket| async move {
            run_session(socket, options).await;
        })
}
