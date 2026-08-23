//! Instance settings (F017).

use std::sync::{Arc, OnceLock};

use tokio::sync::Mutex;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use voxnexus::http::{app, AppState};
use voxnexus_auth::{
    session_cookie_name, sync_locked_community_creation_mode, update_instance, InstancePatch,
};
use voxnexus_db::{connect_and_migrate, test_database_url, PgPool};
use voxnexus_domain::{CommunityCreationMode, RegistrationMode};
use voxnexus_jobs::{connect, RedisConn};
use voxnexus_protocol::{error_codes, MetaResponse};
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

async fn register_admin(router: &axum::Router) -> String {
    let email = format!("admin-{}@example.com", uuid::Uuid::now_v7());
    let response = router
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
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::CREATED);
    cookie_from(&response)
}

async fn register_user(router: &axum::Router, email: &str) -> axum::response::Response {
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

async fn reset_instance(pool: &PgPool) {
    clear_instance_admins(pool).await;
    update_instance(
        pool,
        InstancePatch {
            registration_mode: Some(RegistrationMode::Open),
            community_creation_mode: Some(CommunityCreationMode::Open),
            ..InstancePatch::default()
        },
    )
    .await
    .expect("reset instance");
}

/// CI shares one Postgres across integration tests; clear admins so bootstrap grants admin to the next registrant.
async fn clear_instance_admins(pool: &PgPool) {
    sqlx::query(
        r"
        UPDATE accounts
        SET is_instance_admin = FALSE, updated_at = NOW()
        WHERE is_instance_admin = TRUE AND deleted_at IS NULL
        ",
    )
    .execute(pool)
    .await
    .expect("clear instance admins");
}

fn instance_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

const INSTANCE_MODE_LOCK: i64 = 0x0190_0000_0000_0019;

async fn lock_instance_mode(pool: &PgPool) {
    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(INSTANCE_MODE_LOCK)
        .execute(pool)
        .await
        .expect("advisory lock");
}

async fn unlock_instance_mode(pool: &PgPool) {
    sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(INSTANCE_MODE_LOCK)
        .execute(pool)
        .await
        .expect("advisory unlock");
}

#[tokio::test]
async fn non_admin_cannot_patch_instance_settings() {
    let _guard = instance_test_lock().lock().await;
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    lock_instance_mode(&pool).await;
    reset_instance(&pool).await;
    let router = app(state(pool.clone(), redis));
    let admin_cookie = register_admin(&router).await;
    let user_email = format!("user-{}@example.com", uuid::Uuid::now_v7());
    let user_response = register_user(&router, &user_email).await;
    assert_eq!(user_response.status(), StatusCode::CREATED);
    let user_cookie = cookie_from(&user_response);

    let admin_patch = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/instance/settings")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::from(r#"{"name":"Admin Instance"}"#))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(admin_patch.status(), StatusCode::OK);

    let user_patch = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/instance/settings")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &user_cookie)
                .body(Body::from(r#"{"name":"Hacked"}"#))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(user_patch.status(), StatusCode::FORBIDDEN);
    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn closed_registration_blocks_register() {
    let _guard = instance_test_lock().lock().await;
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    lock_instance_mode(&pool).await;
    reset_instance(&pool).await;
    update_instance(
        &pool,
        InstancePatch {
            registration_mode: Some(RegistrationMode::Closed),
            ..InstancePatch::default()
        },
    )
    .await
    .expect("close registration");
    let router = app(state(pool.clone(), redis));
    let email = format!("closed-{}@example.com", uuid::Uuid::now_v7());
    let response = register_user(&router, &email).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body: voxnexus_protocol::ErrorBody =
        serde_json::from_slice(&json_body(response).await).expect("error");
    assert_eq!(body.code, error_codes::PERMISSION_DENIED);
    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn single_mode_meta_and_non_admin_create_forbidden() {
    let _guard = instance_test_lock().lock().await;
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    lock_instance_mode(&pool).await;
    reset_instance(&pool).await;
    update_instance(
        &pool,
        InstancePatch {
            community_creation_mode: Some(CommunityCreationMode::Single),
            ..InstancePatch::default()
        },
    )
    .await
    .expect("single mode");
    let router = app(state(pool.clone(), redis));

    let meta = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/meta")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(meta.status(), StatusCode::OK);
    let meta_body: MetaResponse = serde_json::from_slice(&json_body(meta).await).expect("meta");
    assert_eq!(
        meta_body.community_creation_mode,
        CommunityCreationMode::Single
    );

    let admin_cookie = register_admin(&router).await;
    let user_email = format!("user-{}@example.com", uuid::Uuid::now_v7());
    let user_response = register_user(&router, &user_email).await;
    assert_eq!(user_response.status(), StatusCode::CREATED);
    let user_cookie = cookie_from(&user_response);

    let user_create = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/communities")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &user_cookie)
                .body(Body::from(r#"{"name":"Nope"}"#))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(user_create.status(), StatusCode::FORBIDDEN);

    let admin_create = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/communities")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::from(r#"{"name":"Nope"}"#))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(admin_create.status(), StatusCode::FORBIDDEN);
    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn locked_config_sync_updates_community_creation_mode() {
    let _guard = instance_test_lock().lock().await;
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    lock_instance_mode(&pool).await;
    reset_instance(&pool).await;

    sync_locked_community_creation_mode(&pool, CommunityCreationMode::Single)
        .await
        .expect("sync");

    let instance = voxnexus_auth::get_instance(&pool).await.expect("instance");
    assert_eq!(
        instance.community_creation_mode,
        CommunityCreationMode::Single
    );
    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn oidc_config_sync_updates_instance_row() {
    let _guard = instance_test_lock().lock().await;
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    lock_instance_mode(&pool).await;
    reset_instance(&pool).await;

    voxnexus_auth::sync_oidc_from_config(
        &pool,
        "http://127.0.0.1:9000/application/o/voxnexus/",
        Some("test-client-id"),
    )
    .await
    .expect("sync");

    let instance = voxnexus_auth::get_instance(&pool).await.expect("instance");
    assert!(instance.oidc_enabled);
    assert_eq!(
        instance.oidc_issuer.as_deref(),
        Some("http://127.0.0.1:9000/application/o/voxnexus/")
    );
    assert_eq!(instance.oidc_client_id.as_deref(), Some("test-client-id"));
    unlock_instance_mode(&pool).await;
}
