//! Community create and settings (F019).

use std::sync::{Arc, OnceLock};

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tokio::sync::Mutex;
use tower::ServiceExt;
use voxnexus::http::{app, AppState};
use voxnexus_auth::{session_cookie_name, update_instance, InstancePatch};
use voxnexus_db::{connect_and_migrate, test_database_url, PgPool};
use voxnexus_domain::CommunityCreationMode;
use voxnexus_jobs::{connect, RedisConn};
use voxnexus_protocol::{error_codes, CommunityListResponse, CommunityResponse};
use voxnexus_search::{MemorySearchEngine, SearchEngine};
use voxnexus_storage::{MemoryObjectStore, ObjectStore};

fn community_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
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

async fn register(router: &axum::Router, email: &str) -> (String, bool) {
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
    let cookie = cookie_from(&response);
    let body: serde_json::Value = serde_json::from_slice(&json_body(response).await).expect("json");
    let is_admin = body["account"]["is_instance_admin"]
        .as_bool()
        .unwrap_or(false);
    (cookie, is_admin)
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

async fn set_mode(pool: &PgPool, mode: CommunityCreationMode) {
    update_instance(
        pool,
        InstancePatch {
            community_creation_mode: Some(mode),
            ..InstancePatch::default()
        },
    )
    .await
    .expect("set mode");
}

#[tokio::test]
async fn open_mode_any_user_can_create_and_list() {
    let _guard = community_test_lock().lock().await;
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
    set_mode(&pool, CommunityCreationMode::Open).await;
    let router = app(state(pool.clone(), redis));
    let email = format!("open-{}@example.com", uuid::Uuid::now_v7());
    let (cookie, _) = register(&router, &email).await;

    let create = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/communities")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &cookie)
                .body(Body::from(r#"{"name":"Open Lab"}"#))
                .expect("request"),
        )
        .await
        .expect("create");
    assert_eq!(create.status(), StatusCode::CREATED);
    let created: CommunityResponse =
        serde_json::from_slice(&json_body(create).await).expect("community");
    assert_eq!(created.name, "Open Lab");
    assert!(
        created.slug.starts_with("open-lab"),
        "unexpected slug {}",
        created.slug
    );

    let list = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/communities")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("list");
    assert_eq!(list.status(), StatusCode::OK);
    let listed: CommunityListResponse =
        serde_json::from_slice(&json_body(list).await).expect("list");
    assert!(listed.communities.iter().any(|c| c.id == created.id));
    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn admin_only_blocks_non_admin_allows_admin() {
    let _guard = community_test_lock().lock().await;
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
    set_mode(&pool, CommunityCreationMode::AdminOnly).await;
    let router = app(state(pool.clone(), redis));

    let user_email = format!("member-{}@example.com", uuid::Uuid::now_v7());
    let (user_cookie, is_admin) = register(&router, &user_email).await;
    assert!(!is_admin);

    let denied = router
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
        .expect("denied");
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    let err: serde_json::Value = serde_json::from_slice(&json_body(denied).await).expect("err");
    assert_eq!(err["code"], error_codes::PERMISSION_DENIED);

    // Promote via SQL so we have an admin without depending on bootstrap env.
    sqlx::query("UPDATE accounts SET is_instance_admin = TRUE WHERE email = $1")
        .bind(user_email.trim().to_ascii_lowercase())
        .execute(&pool)
        .await
        .expect("promote");

    let admin_email = format!("admin2-{}@example.com", uuid::Uuid::now_v7());
    let (admin_cookie, _) = register(&router, &admin_email).await;
    sqlx::query("UPDATE accounts SET is_instance_admin = TRUE WHERE email = $1")
        .bind(admin_email.trim().to_ascii_lowercase())
        .execute(&pool)
        .await
        .expect("promote2");

    // Re-login not required: AuthUser loads is_instance_admin from DB each request.
    let allowed = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/communities")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::from(r#"{"name":"Admin Hub"}"#))
                .expect("request"),
        )
        .await
        .expect("allowed");
    assert_eq!(allowed.status(), StatusCode::CREATED);
    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn single_mode_second_create_forbidden() {
    let _guard = community_test_lock().lock().await;
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
    set_mode(&pool, CommunityCreationMode::Single).await;
    let router = app(state(pool.clone(), redis));
    let email = format!("single-{}@example.com", uuid::Uuid::now_v7());
    let (cookie, _) = register(&router, &email).await;

    let denied = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/communities")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &cookie)
                .body(Body::from(r#"{"name":"Second"}"#))
                .expect("request"),
        )
        .await
        .expect("denied");
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn non_owner_cannot_patch_settings() {
    let _guard = community_test_lock().lock().await;
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
    set_mode(&pool, CommunityCreationMode::Open).await;
    let router = app(state(pool.clone(), redis));

    let (owner_cookie, _) = register(
        &router,
        &format!("owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let create = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/communities")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner_cookie)
                .body(Body::from(r#"{"name":"Private Club"}"#))
                .expect("request"),
        )
        .await
        .expect("create");
    let created: CommunityResponse =
        serde_json::from_slice(&json_body(create).await).expect("community");

    let (other_cookie, _) = register(
        &router,
        &format!("other-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let patch = router
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/communities/{}", created.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &other_cookie)
                .body(Body::from(r#"{"name":"Hijacked"}"#))
                .expect("request"),
        )
        .await
        .expect("patch");
    assert_eq!(patch.status(), StatusCode::FORBIDDEN);
    unlock_instance_mode(&pool).await;
}
