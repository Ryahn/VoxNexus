//! Channel categories (F026).

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
use voxnexus_protocol::{CategoryListResponse, CategoryResponse, CommunityResponse, SpaceResponse};
use voxnexus_search::{MemorySearchEngine, SearchEngine};
use voxnexus_storage::{MemoryObjectStore, ObjectStore};

const INSTANCE_MODE_LOCK: i64 = 0x0190_0000_0000_0023;

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
                    r#"{{"name":"{name}","description":"category test"}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::CREATED);
    serde_json::from_slice(&json_body(response).await).expect("community")
}

async fn create_space(
    router: &axum::Router,
    cookie: &str,
    community_id: uuid::Uuid,
    name: &str,
) -> SpaceResponse {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{community_id}/spaces"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, cookie)
                .body(Body::from(format!(r#"{{"name":"{name}"}}"#)))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::CREATED);
    serde_json::from_slice(&json_body(response).await).expect("space")
}

#[tokio::test]
async fn categories_persist_order_and_reorder() {
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
    let owner = register(
        &router,
        &format!("cat-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Cat Hub").await;
    let space = create_space(&router, &owner, community.id, "Dev").await;

    let mut created = Vec::new();
    for name in ["General", "Voice", "Project"] {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/communities/{}/categories", community.id))
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::ORIGIN, "http://127.0.0.1:8080")
                    .header(header::COOKIE, &owner)
                    .body(Body::from(format!(
                        r#"{{"name":"{name}","space_id":"{}"}}"#,
                        space.id
                    )))
                    .expect("request"),
            )
            .await
            .expect("create");
        assert_eq!(response.status(), StatusCode::CREATED);
        let cat: CategoryResponse =
            serde_json::from_slice(&json_body(response).await).expect("cat");
        created.push(cat);
    }

    let list = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/communities/{}/categories?space_id={}",
                    community.id, space.id
                ))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("list");
    assert_eq!(list.status(), StatusCode::OK);
    let listed: CategoryListResponse =
        serde_json::from_slice(&json_body(list).await).expect("list");
    assert_eq!(listed.categories.len(), 3);
    assert_eq!(listed.categories[0].name, "General");

    let reorder_ids = [created[2].id, created[0].id, created[1].id];
    let reordered = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/communities/{}/categories/reorder",
                    community.id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(format!(
                    r#"{{"category_ids":["{}","{}","{}"]}}"#,
                    reorder_ids[0], reorder_ids[1], reorder_ids[2]
                )))
                .expect("request"),
        )
        .await
        .expect("reorder");
    assert_eq!(reordered.status(), StatusCode::OK);
    let after: CategoryListResponse =
        serde_json::from_slice(&json_body(reordered).await).expect("reordered");
    assert_eq!(after.categories[0].id, created[2].id);
    assert_eq!(after.categories[1].id, created[0].id);

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn cannot_move_category_to_space_in_other_community() {
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
    let owner_a = register(
        &router,
        &format!("cat-a-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community_a = create_community(&router, &owner_a, "Alpha").await;
    let space_a = create_space(&router, &owner_a, community_a.id, "A Space").await;

    let owner_b = register(
        &router,
        &format!("cat-b-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community_b = create_community(&router, &owner_b, "Beta").await;
    let space_b = create_space(&router, &owner_b, community_b.id, "B Space").await;

    let create = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/categories", community_a.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner_a)
                .body(Body::from(format!(
                    r#"{{"name":"Lobby","space_id":"{}"}}"#,
                    space_a.id
                )))
                .expect("request"),
        )
        .await
        .expect("create cat");
    assert_eq!(create.status(), StatusCode::CREATED);
    let category: CategoryResponse =
        serde_json::from_slice(&json_body(create).await).expect("category");

    let denied = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/categories/{}", category.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner_a)
                .body(Body::from(format!(r#"{{"space_id":"{}"}}"#, space_b.id)))
                .expect("request"),
        )
        .await
        .expect("patch");
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    unlock_instance_mode(&pool).await;
}
