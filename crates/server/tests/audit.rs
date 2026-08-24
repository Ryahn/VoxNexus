//! Community audit log (F033).

#![allow(clippy::too_many_lines)]

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
use voxnexus_protocol::{
    AuditEventListResponse, AuthSessionResponse, CommunityResponse, RoleResponse,
};
use voxnexus_search::{MemorySearchEngine, SearchEngine};
use voxnexus_storage::{MemoryObjectStore, ObjectStore};

const INSTANCE_MODE_LOCK: i64 = 0x0190_0000_0000_0026;

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
                    r#"{{"email":"{email}","password":"password123","display_name":"Audit User"}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::CREATED);
    let cookie = cookie_from(&response);
    let body: AuthSessionResponse =
        serde_json::from_slice(&json_body(response).await).expect("json");
    (cookie, body.account.id)
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
                    r#"{{"name":"{name}","description":"audit test"}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::CREATED);
    serde_json::from_slice(&json_body(response).await).expect("community")
}

#[tokio::test]
async fn create_role_produces_filterable_audit_row() {
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
        &format!("audit-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Audit Hub").await;

    let create = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/roles", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(r#"{"name":"Moderators","weight":40}"#))
                .expect("request"),
        )
        .await
        .expect("create role");
    assert_eq!(create.status(), StatusCode::CREATED);
    let role: RoleResponse = serde_json::from_slice(&json_body(create).await).expect("role");

    let list = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/communities/{}/audit-events?action=role.create&limit=20",
                    community.id
                ))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("list audit");
    assert_eq!(list.status(), StatusCode::OK);
    let page: AuditEventListResponse =
        serde_json::from_slice(&json_body(list).await).expect("audit page");
    assert!(
        page.items
            .iter()
            .any(|event| { event.action == "role.create" && event.target_id == Some(role.id) }),
        "expected role.create audit for {}",
        role.id
    );

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn audit_list_forbidden_without_view_audit() {
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
        &format!("audit-forbid-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Audit Forbid").await;
    let (member, _) = register(
        &router,
        &format!("audit-forbid-member-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let join = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/join", community.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &member)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("join");
    assert_eq!(join.status(), StatusCode::CREATED);

    let list = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/communities/{}/audit-events", community.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &member)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("list");
    assert_eq!(list.status(), StatusCode::FORBIDDEN);

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn audit_list_paginates() {
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
        &format!("audit-page-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Audit Page").await;

    for i in 0..3 {
        let create = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/communities/{}/roles", community.id))
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::ORIGIN, "http://127.0.0.1:8080")
                    .header(header::COOKIE, &owner)
                    .body(Body::from(format!(
                        r#"{{"name":"Role-{i}","weight":{}}}"#,
                        30 + i
                    )))
                    .expect("request"),
            )
            .await
            .expect("create");
        assert_eq!(create.status(), StatusCode::CREATED);
    }

    let page1 = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/communities/{}/audit-events?action=role.create&limit=2",
                    community.id
                ))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("page1");
    assert_eq!(page1.status(), StatusCode::OK);
    let body1: AuditEventListResponse =
        serde_json::from_slice(&json_body(page1).await).expect("page1 body");
    assert_eq!(body1.items.len(), 2);
    assert!(body1.has_more);
    let cursor = body1.items[1].id;

    let page2 = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/communities/{}/audit-events?action=role.create&limit=2&after={cursor}",
                    community.id
                ))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("page2");
    assert_eq!(page2.status(), StatusCode::OK);
    let body2: AuditEventListResponse =
        serde_json::from_slice(&json_body(page2).await).expect("page2 body");
    assert!(!body2.items.is_empty());
    assert!(body2.items.iter().all(|event| event.id < cursor));

    unlock_instance_mode(&pool).await;
}
