//! HTTP surface: probes, `/api/v1`, SPA static files, error fallback, and shared middleware.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderName, HeaderValue, Method, Request, StatusCode};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use axum::{Json, Router};
use serde::Serialize;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{
    MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::Span;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;
use voxnexus_config::Url;
use voxnexus_db::PgPool;
use voxnexus_jobs::RedisConn;
use voxnexus_protocol::MetaResponse;
use voxnexus_realtime::ResumeStore;
use voxnexus_search::SearchEngine;
use voxnexus_storage::ObjectStore;

use crate::auth_middleware::require_api_session;
use crate::csrf::{csrf_hook, CsrfState};
use crate::error::ApiError;

/// Header used for request correlation (`x-request-id`).
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Maximum JSON (and other) request body size for the HTTP API.
pub const MAX_JSON_BODY_BYTES: usize = 1024 * 1024;

#[derive(Clone, Default)]
struct RequestIdV7;

impl MakeRequestId for RequestIdV7 {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let value = HeaderValue::from_str(&Uuid::now_v7().to_string()).ok()?;
        Some(RequestId::new(value))
    }
}

/// Shared application state for Axum handlers.
#[derive(Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct AppState {
    pub pool: PgPool,
    pub metrics_enabled: bool,
    pub public_url: Url,
    pub cookie_secure: bool,
    pub registration_open: bool,
    pub gateway_allow_unauth: bool,
    pub gateway_heartbeat_interval: Duration,
    pub storage: Arc<dyn ObjectStore>,
    pub redis: RedisConn,
    pub search: Arc<dyn SearchEngine>,
    /// Built SPA directory for production (`WEB_DIST`). None keeps JSON API-only fallbacks.
    pub web_dist: Option<PathBuf>,
    /// In-memory gateway resume tokens.
    pub resume_store: Arc<ResumeStore>,
}

#[derive(Serialize)]
struct HealthBody {
    status: &'static str,
}

#[derive(Serialize)]
struct ReadyBody {
    status: &'static str,
    postgres: &'static str,
    redis: &'static str,
    seaweedfs: &'static str,
    typesense: &'static str,
}

/// Router that serves `/health` without a database (for unit tests).
pub fn health_router() -> Router {
    with_observe(Router::new().route("/health", get(health)))
}

/// Application router: probes, `/api/v1`, optional SPA static files, and optional `/metrics`.
pub fn app(state: AppState) -> Router {
    let public_url = state.public_url.clone();
    let metrics_enabled = state.metrics_enabled;
    let web_dist = state.web_dist.clone();
    let (api, _) = api_v1().split_for_parts();
    let mut router = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/v1/gateway", get(crate::gateway::gateway_upgrade))
        .merge(api)
        .route("/api/{*rest}", any(not_found));
    if metrics_enabled {
        router = router.route("/metrics", get(metrics));
    }
    let cookie_secure = state.cookie_secure;
    let auth_state = state.clone();
    let router = match web_dist {
        Some(dir) if dir.is_dir() => {
            let index = dir.join("index.html");
            let spa = ServeDir::new(dir)
                .append_index_html_on_directories(true)
                .not_found_service(ServeFile::new(index));
            router.fallback_service(spa)
        }
        Some(dir) => {
            tracing::warn!(
                path = %dir.display(),
                "WEB_DIST set but directory missing; API-only fallbacks"
            );
            router.fallback(not_found)
        }
        None => router.fallback(not_found),
    };
    let router = router
        .layer(middleware::from_fn_with_state(
            auth_state,
            require_api_session,
        ))
        .with_state(state);
    with_middleware(router, &public_url, cookie_secure)
}

fn api_v1() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(meta))
        .routes(routes!(crate::auth::register))
        .routes(routes!(crate::auth::login))
        .routes(routes!(crate::auth::logout))
        .routes(routes!(crate::auth::me))
}

/// Request-id, body limit, compression, CORS, and CSRF Origin check.
pub fn with_middleware<S>(router: Router<S>, public_url: &Url, cookie_secure: bool) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let csrf = CsrfState {
        public_url: public_url.clone(),
        cookie_secure,
    };
    router
        .layer(middleware::from_fn_with_state(csrf, csrf_hook))
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::x_request_id(RequestIdV7))
                .layer(PropagateRequestIdLayer::x_request_id())
                .layer(TraceLayer::new_for_http().make_span_with(make_span))
                .layer(RequestBodyLimitLayer::new(MAX_JSON_BODY_BYTES))
                .layer(CompressionLayer::new())
                .layer(cors_layer(public_url)),
        )
}

fn with_observe<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.layer(
        ServiceBuilder::new()
            .layer(SetRequestIdLayer::x_request_id(RequestIdV7))
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(TraceLayer::new_for_http().make_span_with(make_span)),
    )
}

fn cors_layer(public_url: &Url) -> CorsLayer {
    let mut layer = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            HeaderName::from_static(REQUEST_ID_HEADER),
        ]);
    if let Some(origin) = origin_header_value(public_url) {
        layer = layer.allow_origin(origin).allow_credentials(true);
    }
    layer
}

fn origin_header_value(public_url: &Url) -> Option<HeaderValue> {
    let host = public_url.host_str()?;
    let mut origin = format!("{}://{host}", public_url.scheme());
    if let Some(port) = public_url.port() {
        origin.push(':');
        origin.push_str(&port.to_string());
    }
    HeaderValue::from_str(&origin).ok()
}

fn make_span<B>(request: &Request<B>) -> Span {
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("missing");
    tracing::info_span!(
        "request",
        request_id = request_id,
        method = %request.method(),
        uri = %request.uri()
    )
}

/// UUIDv7 from `x-request-id`, or `"missing"` if the layer has not run.
#[must_use]
pub fn request_id_from_headers(headers: &HeaderMap) -> String {
    headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("missing")
        .to_owned()
}

async fn health() -> impl IntoResponse {
    Json(HealthBody { status: "ok" })
}

async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    let postgres = match voxnexus_db::ping(&state.pool).await {
        Ok(()) => "ok",
        Err(error) => {
            tracing::warn!(error = %error, "postgres readiness check failed");
            "error"
        }
    };
    let redis = match voxnexus_jobs::ping(&state.redis).await {
        Ok(()) => "ok",
        Err(error) => {
            tracing::warn!(error = %error, "redis readiness check failed");
            "error"
        }
    };
    let seaweedfs = match state.storage.head_bucket().await {
        Ok(()) => "ok",
        Err(error) => {
            tracing::warn!(error = %error, "seaweedfs readiness check failed");
            "error"
        }
    };
    let typesense = match state.search.ping().await {
        Ok(()) => "ok",
        Err(error) => {
            tracing::warn!(error = %error, "typesense readiness check failed");
            "error"
        }
    };
    let ok = postgres == "ok" && redis == "ok" && seaweedfs == "ok" && typesense == "ok";
    let status = if ok { "ok" } else { "unavailable" };
    let code = if ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        code,
        Json(ReadyBody {
            status,
            postgres,
            redis,
            seaweedfs,
            typesense,
        }),
    )
        .into_response()
}

async fn metrics() -> Response {
    (
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        "# TYPE voxnexus_up gauge\nvoxnexus_up 1\n",
    )
        .into_response()
}

/// Instance name and crate version.
#[utoipa::path(
    get,
    path = "/api/v1/meta",
    operation_id = "getMeta",
    tag = "meta",
    responses(
        (status = 200, description = "Instance name and version", body = MetaResponse)
    )
)]
pub async fn meta() -> Json<MetaResponse> {
    Json(MetaResponse {
        name: "voxnexus".to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
    })
}

/// JSON [`voxnexus_protocol::ErrorBody`] for unknown routes (including hidden resources later).
pub async fn not_found(headers: HeaderMap) -> ApiError {
    ApiError::not_found(request_id_from_headers(&headers))
}
