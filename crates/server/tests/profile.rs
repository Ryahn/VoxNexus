use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use voxnexus::http::{app, AppState};
use voxnexus_auth::session_cookie_name;
use voxnexus_db::{connect_and_migrate, test_database_url, PgPool};
use voxnexus_jobs::{connect, RedisConn};
use voxnexus_media::AVATAR_MAX_BYTES;
use voxnexus_protocol::{error_codes, ErrorBody, ProfileResponse};
use voxnexus_realtime::ResumeStore;
use voxnexus_search::{MemorySearchEngine, SearchEngine};
use voxnexus_storage::{MemoryObjectStore, ObjectStore};

/// Minimal valid 1×1 PNG.
fn tiny_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8,
        0xff, 0xff, 0x3f, 0x00, 0x05, 0xfe, 0x02, 0xfe, 0xdc, 0xcc, 0x59, 0xe7, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ]
}

async fn test_redis() -> Option<RedisConn> {
    let url = std::env::var("REDIS_URL_TEST")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "redis://127.0.0.1:6379".to_owned());
    connect(&url).await.ok()
}

fn state(pool: PgPool, redis: RedisConn) -> AppState {
    AppState {
        pool,
        metrics_enabled: false,
        public_url: "http://127.0.0.1:8080".parse().expect("url"),
        cookie_secure: false,
        community_creation_mode_locked: false,
        gateway_allow_unauth: false,
        gateway_heartbeat_interval: std::time::Duration::from_secs(15),
        storage: Arc::new(MemoryObjectStore::new_ready()) as Arc<dyn ObjectStore>,
        redis,
        search: Arc::new(MemorySearchEngine::new_ready()) as Arc<dyn SearchEngine>,
        web_dist: None,
        resume_store: Arc::new(ResumeStore::new()),
        presence_hub: Arc::new(voxnexus_realtime::PresenceHub::with_default_grace()),
        oidc_client_secret: None,
        oidc_only: false,
        oidc_link_by_email: true,
    }
}

async fn json_body(response: axum::response::Response) -> Vec<u8> {
    response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes()
        .to_vec()
}

fn cookie_from(response: &axum::response::Response) -> String {
    let set_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .expect("set-cookie")
        .to_str()
        .expect("cookie str");
    let name = session_cookie_name(false);
    let token = set_cookie
        .split(';')
        .next()
        .expect("pair")
        .strip_prefix(&format!("{name}="))
        .expect("token");
    format!("{name}={token}")
}

async fn register(router: &axum::Router, email: &str) -> String {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .body(Body::from(format!(
                    r#"{{"email":"{email}","password":"password123","display_name":"Test User"}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::CREATED);
    cookie_from(&response)
}

#[tokio::test]
async fn profile_update_and_avatar_round_trip() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let email = format!("prof-{}@example.com", uuid::Uuid::now_v7());
    let router = app(state(pool, redis));
    let cookie = register(&router, &email).await;

    let patch = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/me/profile")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    r#"{"display_name":"Nova","bio":"builds communities"}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(patch.status(), StatusCode::OK);
    let profile: ProfileResponse = serde_json::from_slice(&json_body(patch).await).expect("json");
    assert_eq!(profile.display_name, "Nova");
    assert_eq!(profile.bio, "builds communities");

    let upload = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/me/profile/avatar")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &cookie)
                .body(Body::from(tiny_png()))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(upload.status(), StatusCode::OK);
    let with_avatar: ProfileResponse =
        serde_json::from_slice(&json_body(upload).await).expect("json");
    assert!(with_avatar.has_avatar);
    let avatar_url = with_avatar.avatar_url.expect("avatar url");

    let image = router
        .oneshot(
            Request::builder()
                .uri(&avatar_url)
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(image.status(), StatusCode::OK);
    assert_eq!(
        image
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("image/png")
    );
    let bytes = json_body(image).await;
    assert_eq!(&bytes[..8], &tiny_png()[..8]);
}

#[tokio::test]
async fn oversized_avatar_rejected() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let email = format!("big-{}@example.com", uuid::Uuid::now_v7());
    let router = app(state(pool, redis));
    let cookie = register(&router, &email).await;

    let mut body = tiny_png();
    body.resize(AVATAR_MAX_BYTES + 1, 0);
    // Keep PNG magic so sniff would pass size check first.
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/me/profile/avatar")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &cookie)
                .body(Body::from(body))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let err: ErrorBody = serde_json::from_slice(&json_body(response).await).expect("error");
    assert_eq!(err.code, error_codes::VALIDATION_ERROR);
}

#[tokio::test]
async fn avatar_upload_cannot_affect_other_account() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let router = app(state(pool, redis));
    let cookie_a = register(&router, &format!("a-{}@example.com", uuid::Uuid::now_v7())).await;
    let cookie_b = register(&router, &format!("b-{}@example.com", uuid::Uuid::now_v7())).await;

    let me_b = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/me/profile")
                .header(header::COOKIE, &cookie_b)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    let profile_b: ProfileResponse = serde_json::from_slice(&json_body(me_b).await).expect("json");

    let upload = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/me/profile/avatar")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &cookie_a)
                .body(Body::from(tiny_png()))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(upload.status(), StatusCode::OK);

    let other = router
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/profiles/{}", profile_b.account_id))
                .header(header::COOKIE, &cookie_a)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(other.status(), StatusCode::OK);
    let still_b: ProfileResponse = serde_json::from_slice(&json_body(other).await).expect("json");
    assert!(!still_b.has_avatar);
}
