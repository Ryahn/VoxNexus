//! Password and email change (F015).

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use voxnexus::http::{app, AppState};
use voxnexus_auth::session_cookie_name;
use voxnexus_db::{connect_and_migrate, test_database_url, PgPool};
use voxnexus_jobs::{connect, RedisConn};
use voxnexus_protocol::{error_codes, AuthSessionResponse, ErrorBody};
use voxnexus_search::{MemorySearchEngine, SearchEngine};
use voxnexus_storage::{MemoryObjectStore, ObjectStore};

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
        resume_store: Arc::new(voxnexus_realtime::ResumeStore::new()),
        presence_hub: Arc::new(voxnexus_realtime::PresenceHub::with_default_grace()),
        oidc_client_secret: None,
        oidc_only: false,
        oidc_link_by_email: true,
        permission_cache: Arc::new(voxnexus_permissions::PermissionCache::default()),
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

async fn register_cookie(router: &axum::Router, email: &str) -> String {
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
async fn wrong_current_password_rejected() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let email = format!("pwd-wrong-{}@example.com", uuid::Uuid::now_v7());
    let router = app(state(pool, redis));
    let cookie = register_cookie(&router, &email).await;

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/me/password")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    r#"{"current_password":"wrong","new_password":"newpassword123"}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let err: ErrorBody = serde_json::from_slice(&json_body(response).await).expect("error");
    assert_eq!(err.code, error_codes::UNAUTHENTICATED);
}

#[tokio::test]
async fn password_change_then_login_with_new_password() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let email = format!("pwd-ok-{}@example.com", uuid::Uuid::now_v7());
    let router = app(state(pool, redis));
    let cookie = register_cookie(&router, &email).await;

    let change = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/me/password")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    r#"{"current_password":"password123","new_password":"newpassword456"}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(change.status(), StatusCode::NO_CONTENT);

    let old_login = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .body(Body::from(format!(
                    r#"{{"email":"{email}","password":"password123","display_name":"Test User"}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(old_login.status(), StatusCode::UNAUTHORIZED);

    let new_login = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .body(Body::from(format!(
                    r#"{{"email":"{email}","password":"newpassword456"}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(new_login.status(), StatusCode::OK);
}

#[tokio::test]
async fn duplicate_email_change_conflicts() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let email_a = format!("email-a-{}@example.com", uuid::Uuid::now_v7());
    let email_b = format!("email-b-{}@example.com", uuid::Uuid::now_v7());
    let router = app(state(pool, redis));
    register_cookie(&router, &email_a).await;
    let cookie_b = register_cookie(&router, &email_b).await;

    let response = router
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/auth/me/email")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &cookie_b)
                .body(Body::from(format!(
                    r#"{{"email":"{email_a}","current_password":"password123"}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::CONFLICT);
    let err: ErrorBody = serde_json::from_slice(&json_body(response).await).expect("error");
    assert_eq!(err.code, error_codes::CONFLICT);
}

#[tokio::test]
async fn revoke_other_sessions_on_password_change() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let email = format!("revoke-{}@example.com", uuid::Uuid::now_v7());
    let router = app(state(pool.clone(), redis));
    let cookie_a = register_cookie(&router, &email).await;

    let login_b = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .body(Body::from(format!(
                    r#"{{"email":"{email}","password":"password123","display_name":"Test User"}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(login_b.status(), StatusCode::OK);
    let cookie_b = cookie_from(&login_b);

    let change = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/me/password")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &cookie_a)
                .body(Body::from(
                    r#"{"current_password":"password123","new_password":"newpassword789","revoke_other_sessions":true}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(change.status(), StatusCode::NO_CONTENT);

    let me_a = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .header(header::COOKIE, &cookie_a)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(me_a.status(), StatusCode::OK);

    let me_b = router
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .header(header::COOKIE, &cookie_b)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(me_b.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn email_change_applies_immediately() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let email = format!("old-{}@example.com", uuid::Uuid::now_v7());
    let new_email = format!("new-{}@example.com", uuid::Uuid::now_v7());
    let router = app(state(pool, redis));
    let cookie = register_cookie(&router, &email).await;

    let patch = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/auth/me/email")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &cookie)
                .body(Body::from(format!(
                    r#"{{"email":"{new_email}","current_password":"password123"}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(patch.status(), StatusCode::OK);
    let body: AuthSessionResponse = serde_json::from_slice(&json_body(patch).await).expect("json");
    assert_eq!(body.account.email.as_deref(), Some(new_email.as_str()));

    let me = router
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(me.status(), StatusCode::OK);
    let me_body: AuthSessionResponse = serde_json::from_slice(&json_body(me).await).expect("json");
    assert_eq!(me_body.account.email.as_deref(), Some(new_email.as_str()));
}
