//! Community roles (F028).

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use voxnexus::http::{app, AppState};
use voxnexus_auth::{session_cookie_name, update_instance, InstancePatch};
use voxnexus_db::{connect_and_migrate, test_database_url, PgPool};
use voxnexus_domain::CommunityCreationMode;
use voxnexus_jobs::{connect, RedisConn};
use voxnexus_protocol::{AuthSessionResponse, CommunityResponse, RoleListResponse, RoleResponse};
use voxnexus_search::{MemorySearchEngine, SearchEngine};
use voxnexus_storage::{MemoryObjectStore, ObjectStore};

const INSTANCE_MODE_LOCK: i64 = 0x0190_0000_0000_0025;

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

async fn register(router: &axum::Router, email: &str) -> (String, uuid::Uuid) {
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
    let me = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/auth/me")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("me");
    assert_eq!(me.status(), StatusCode::OK);
    let account: AuthSessionResponse = serde_json::from_slice(&json_body(me).await).expect("me");
    (cookie, account.account.id)
}

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

async fn create_community(router: &axum::Router, cookie: &str, name: &str) -> CommunityResponse {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/communities")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, cookie)
                .body(Body::from(format!(
                    r#"{{"name":"{name}","description":"role test"}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::CREATED);
    serde_json::from_slice(&json_body(response).await).expect("community")
}

async fn create_role(
    router: &axum::Router,
    cookie: &str,
    community_id: uuid::Uuid,
    name: &str,
    manage_roles: bool,
) -> RoleResponse {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{community_id}/roles"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, cookie)
                .body(Body::from(format!(
                    r#"{{"name":"{name}","manage_roles":{manage_roles}}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("create role");
    assert_eq!(response.status(), StatusCode::CREATED);
    serde_json::from_slice(&json_body(response).await).expect("role")
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn role_manager_cannot_edit_equal_or_higher_role() {
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
    update_instance(
        &pool,
        InstancePatch {
            community_creation_mode: Some(CommunityCreationMode::Open),
            ..InstancePatch::default()
        },
    )
    .await
    .expect("open mode");

    let router = app(state(pool.clone(), redis));
    let (owner, _) = register(
        &router,
        &format!("role-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Role Hub").await;
    let admin = create_role(&router, &owner, community.id, "Admin", true).await;
    let _moderator = create_role(&router, &owner, community.id, "Moderator", false).await;

    let (manager, manager_id) = register(
        &router,
        &format!("role-manager-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let join = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/join", community.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &manager)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("join");
    assert_eq!(join.status(), StatusCode::CREATED);

    let assign = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/communities/{}/members/{}/roles",
                    community.id, manager_id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(format!(r#"{{"role_id":"{}"}}"#, admin.id)))
                .expect("request"),
        )
        .await
        .expect("assign");
    assert_eq!(assign.status(), StatusCode::NO_CONTENT);

    let denied_edit = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/roles/{}", admin.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &manager)
                .body(Body::from(r#"{"name":"Super Admin"}"#))
                .expect("request"),
        )
        .await
        .expect("patch");
    assert_eq!(denied_edit.status(), StatusCode::FORBIDDEN);

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn role_manager_cannot_assign_higher_role() {
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
    update_instance(
        &pool,
        InstancePatch {
            community_creation_mode: Some(CommunityCreationMode::Open),
            ..InstancePatch::default()
        },
    )
    .await
    .expect("open mode");

    let router = app(state(pool.clone(), redis));
    let (owner, _) = register(
        &router,
        &format!("role-owner2-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Role Assign").await;
    let helper = create_role(&router, &owner, community.id, "Helper", false).await;
    let admin = create_role(&router, &owner, community.id, "Admin", true).await;

    let (manager, manager_id) = register(
        &router,
        &format!("role-mgr2-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let (target, target_id) = register(
        &router,
        &format!("role-target-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;

    for cookie in [&manager, &target] {
        let join = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/communities/{}/join", community.id))
                    .header(header::ORIGIN, "http://127.0.0.1:8080")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("join");
        assert_eq!(join.status(), StatusCode::CREATED);
    }

    let assign_manager = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/communities/{}/members/{}/roles",
                    community.id, manager_id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(format!(r#"{{"role_id":"{}"}}"#, admin.id)))
                .expect("request"),
        )
        .await
        .expect("assign manager");
    assert_eq!(assign_manager.status(), StatusCode::NO_CONTENT);

    let denied = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/communities/{}/members/{}/roles",
                    community.id, target_id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &manager)
                .body(Body::from(format!(r#"{{"role_id":"{}"}}"#, admin.id)))
                .expect("request"),
        )
        .await
        .expect("assign admin");
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    let allowed = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/communities/{}/members/{}/roles",
                    community.id, target_id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &manager)
                .body(Body::from(format!(r#"{{"role_id":"{}"}}"#, helper.id)))
                .expect("request"),
        )
        .await
        .expect("assign helper");
    assert_eq!(allowed.status(), StatusCode::NO_CONTENT);

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn everyone_role_created_with_community() {
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
    update_instance(
        &pool,
        InstancePatch {
            community_creation_mode: Some(CommunityCreationMode::Open),
            ..InstancePatch::default()
        },
    )
    .await
    .expect("open mode");

    let router = app(state(pool.clone(), redis));
    let (owner, _) = register(
        &router,
        &format!("role-everyone-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Everyone Hub").await;

    let list = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/communities/{}/roles", community.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("list");
    assert_eq!(list.status(), StatusCode::OK);
    let roles: RoleListResponse = serde_json::from_slice(&json_body(list).await).expect("roles");
    assert!(roles
        .roles
        .iter()
        .any(|role| role.is_everyone && role.name == "@everyone"));

    unlock_instance_mode(&pool).await;
}
