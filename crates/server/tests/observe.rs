use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use voxnexus::http::{app, health_router, AppState, REQUEST_ID_HEADER};
use voxnexus_db::{test_database_url, PgPool};

fn test_state(pool: PgPool, metrics_enabled: bool) -> AppState {
    AppState {
        pool,
        metrics_enabled,
        public_url: "http://127.0.0.1:8080".parse().expect("public url"),
    }
}

#[tokio::test]
async fn health_returns_200() {
    let response = health_router()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let body = String::from_utf8(body.to_vec()).expect("utf8");
    assert!(body.contains("\"status\":\"ok\"") || body.contains("\"status\": \"ok\""));
}

#[tokio::test]
async fn health_includes_request_id_header() {
    let response = health_router()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    let header = response
        .headers()
        .get(REQUEST_ID_HEADER)
        .expect("x-request-id")
        .to_str()
        .expect("header str");
    let parsed = uuid::Uuid::parse_str(header).expect("uuid request id");
    assert_eq!(parsed.get_version_num(), 7);
}

#[tokio::test]
async fn metrics_off_by_default_is_404() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required for full app router");
        return;
    };
    let pool = voxnexus_db::connect(&url).await.expect("connect");
    let response = app(test_state(pool, false))
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn metrics_enabled_exposes_prometheus_text() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required for /metrics");
        return;
    };
    let pool = voxnexus_db::connect(&url).await.expect("connect");
    let response = app(test_state(pool, true))
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let body = String::from_utf8(body.to_vec()).expect("utf8");
    assert!(body.contains("voxnexus_up"));
}

#[tokio::test]
async fn ready_requires_postgres() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required for /ready");
        return;
    };
    let pool = voxnexus_db::connect_and_migrate(&url)
        .await
        .expect("migrate");
    let response = app(test_state(pool, false))
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let body = String::from_utf8(body.to_vec()).expect("utf8");
    assert!(body.contains("postgres"));
    assert!(body.contains("skipped"));
}
