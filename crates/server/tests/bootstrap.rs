use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use voxnexus::http::{app, AppState};
use voxnexus_auth::{bootstrap_instance_admin, create_local_account, BootstrapResult};
use voxnexus_db::{connect_and_migrate, test_database_url, PgPool};
use voxnexus_jobs::{connect, RedisConn};
use voxnexus_protocol::AuthSessionResponse;
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
        registration_open: true,
        gateway_allow_unauth: false,
        gateway_heartbeat_interval: std::time::Duration::from_secs(15),
        storage: Arc::new(MemoryObjectStore::new_ready()) as Arc<dyn ObjectStore>,
        redis,
        search: Arc::new(MemorySearchEngine::new_ready()) as Arc<dyn SearchEngine>,
        web_dist: None,
        resume_store: Arc::new(voxnexus_realtime::ResumeStore::new()),
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

async fn register(router: &axum::Router, email: &str) -> axum::response::Response {
    router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .body(Body::from(format!(
                    r#"{{"email":"{email}","password":"password123"}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("oneshot")
}

async fn is_instance_admin(pool: &PgPool, email: &str) -> bool {
    sqlx::query_scalar(
        r"
        SELECT is_instance_admin
        FROM accounts
        WHERE email = $1 AND deleted_at IS NULL
        ",
    )
    .bind(email.trim().to_ascii_lowercase())
    .fetch_one(pool)
    .await
    .expect("admin flag")
}

async fn instance_admin_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar(
        r"
        SELECT COUNT(*)::bigint
        FROM accounts
        WHERE is_instance_admin = TRUE AND deleted_at IS NULL
        ",
    )
    .fetch_one(pool)
    .await
    .expect("count")
}

#[tokio::test]
async fn second_registered_user_is_not_instance_admin() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let router = app(state(pool.clone(), redis));

    let first_email = format!("first-{}@example.com", uuid::Uuid::now_v7());
    let second_email = format!("second-{}@example.com", uuid::Uuid::now_v7());

    let first = register(&router, &first_email).await;
    assert_eq!(first.status(), StatusCode::CREATED);
    let first_body: AuthSessionResponse =
        serde_json::from_slice(&json_body(first).await).expect("json");
    assert!(first_body.account.is_instance_admin);

    let second = register(&router, &second_email).await;
    assert_eq!(second.status(), StatusCode::CREATED);
    let second_body: AuthSessionResponse =
        serde_json::from_slice(&json_body(second).await).expect("json");
    assert!(!second_body.account.is_instance_admin);

    assert!(is_instance_admin(&pool, &first_email).await);
    assert!(!is_instance_admin(&pool, &second_email).await);
    assert_eq!(instance_admin_count(&pool).await, 1);
}

#[tokio::test]
async fn concurrent_bootstrap_creates_single_instance_admin() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let router = app(state(pool.clone(), redis));

    let email_a = format!("a-{}@example.com", uuid::Uuid::now_v7());
    let email_b = format!("b-{}@example.com", uuid::Uuid::now_v7());

    let (response_a, response_b) =
        tokio::join!(register(&router, &email_a), register(&router, &email_b));

    assert_eq!(response_a.status(), StatusCode::CREATED);
    assert_eq!(response_b.status(), StatusCode::CREATED);
    assert_eq!(instance_admin_count(&pool).await, 1);

    let admin_a = is_instance_admin(&pool, &email_a).await;
    let admin_b = is_instance_admin(&pool, &email_b).await;
    assert_ne!(admin_a, admin_b);
    assert!(admin_a || admin_b);
}

#[tokio::test]
async fn bootstrap_env_creates_instance_admin_once() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let email = format!("bootstrap-{}@example.com", uuid::Uuid::now_v7());

    let created = bootstrap_instance_admin(&pool, &email, "bootstrap-password")
        .await
        .expect("bootstrap");
    assert_eq!(created, BootstrapResult::Created);
    assert!(is_instance_admin(&pool, &email).await);

    let again = bootstrap_instance_admin(&pool, &email, "bootstrap-password")
        .await
        .expect("bootstrap again");
    assert_eq!(again, BootstrapResult::AlreadyBootstrapped);
    assert_eq!(instance_admin_count(&pool).await, 1);
}

#[tokio::test]
async fn bootstrap_env_admin_blocks_registration_bootstrap_for_others() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let admin_email = format!("env-admin-{}@example.com", uuid::Uuid::now_v7());
    bootstrap_instance_admin(&pool, &admin_email, "bootstrap-password")
        .await
        .expect("bootstrap");

    let other = create_local_account(
        &pool,
        &format!("other-{}@example.com", uuid::Uuid::now_v7()),
        "password123",
        true,
    )
    .await
    .expect("register");
    assert!(!other.is_instance_admin);
    assert_eq!(instance_admin_count(&pool).await, 1);
}
