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

#[tokio::test]
async fn protected_route_without_cookie_is_401() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let response = app(state(pool, redis))
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let err: ErrorBody = serde_json::from_slice(&json_body(response).await).expect("error");
    assert_eq!(err.code, error_codes::UNAUTHENTICATED);
}

#[tokio::test]
async fn register_login_logout_session_flow() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let email = format!("user-{}@example.com", uuid::Uuid::now_v7());
    let router = app(state(pool, redis));

    let register = router
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
    assert_eq!(register.status(), StatusCode::CREATED);
    let cookie = cookie_from(&register);
    let registered: AuthSessionResponse =
        serde_json::from_slice(&json_body(register).await).expect("json");
    assert_eq!(registered.account.email.as_deref(), Some(email.as_str()));

    let me = router
        .clone()
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

    let bad_login = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .body(Body::from(format!(
                    r#"{{"email":"{email}","password":"wrong-password"}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(bad_login.status(), StatusCode::UNAUTHORIZED);
    let err: ErrorBody = serde_json::from_slice(&json_body(bad_login).await).expect("error");
    assert_eq!(err.code, error_codes::UNAUTHENTICATED);

    let logout = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(logout.status(), StatusCode::NO_CONTENT);

    let me_after = router
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(me_after.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn duplicate_email_conflicts_and_sql_injection_is_literal() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let email = format!("dup-{}@example.com", uuid::Uuid::now_v7());
    let router = app(state(pool.clone(), redis));

    let first = router
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
    assert_eq!(first.status(), StatusCode::CREATED);

    let second = router
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
    assert_eq!(second.status(), StatusCode::CONFLICT);

    let injection = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .body(Body::from(
                    r#"{"email":"x'; DROP TABLE accounts;--@ex.com","password":"password123","display_name":"Test"}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert!(
        injection.status() == StatusCode::UNAUTHORIZED
            || injection.status() == StatusCode::BAD_REQUEST
    );
    let still_there: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM accounts")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert!(still_there.0 >= 1);
}

#[tokio::test]
async fn auth_identity_issuer_subject_unique() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let account = voxnexus_auth::create_local_account(
        &pool,
        &format!("oidc-{}@example.com", uuid::Uuid::now_v7()),
        "password123",
        "OIDC User",
        true,
    )
    .await
    .expect("account");
    let subject = format!("sub-{}", uuid::Uuid::now_v7());
    voxnexus_auth::insert_auth_identity(&pool, account.id, "https://idp.example", &subject)
        .await
        .expect("link");
    let err =
        voxnexus_auth::insert_auth_identity(&pool, account.id, "https://idp.example", &subject)
            .await
            .expect_err("duplicate");
    assert!(matches!(err, voxnexus_auth::AuthError::IdentityTaken));
}
