//! Permission engine integration (F029).

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
    AuthSessionResponse, CategoryResponse, ChannelListResponse, ChannelResponse,
    CommunityResponse, SpaceResponse,
};
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
    let body: AuthSessionResponse = serde_json::from_slice(&json_body(response).await).expect("json");
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
                    r#"{{"name":"{name}","description":"perm test"}}"#
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

async fn create_category(
    router: &axum::Router,
    cookie: &str,
    community_id: uuid::Uuid,
    space_id: uuid::Uuid,
    name: &str,
) -> CategoryResponse {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{community_id}/categories"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, cookie)
                .body(Body::from(format!(
                    r#"{{"name":"{name}","space_id":"{space_id}"}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::CREATED);
    serde_json::from_slice(&json_body(response).await).expect("category")
}

async fn create_text_channel(
    router: &axum::Router,
    cookie: &str,
    community_id: uuid::Uuid,
    space_id: uuid::Uuid,
    category_id: uuid::Uuid,
    name: &str,
) -> ChannelResponse {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{community_id}/channels"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, cookie)
                .body(Body::from(format!(
                    r#"{{"name":"{name}","type":"text","space_id":"{space_id}","category_id":"{category_id}"}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::CREATED);
    serde_json::from_slice(&json_body(response).await).expect("channel")
}

#[tokio::test]
async fn channel_list_filtered_without_text_view_grant() {
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
        &format!("perm-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Perm Hub").await;
    let space = create_space(&router, &owner, community.id, "Main").await;
    let category = create_category(&router, &owner, community.id, space.id, "General").await;
    let channel =
        create_text_channel(&router, &owner, community.id, space.id, category.id, "hidden").await;

    sqlx::query(
        r"
        UPDATE community_roles
        SET permissions = '{}'::jsonb, updated_at = now()
        WHERE community_id = $1 AND is_everyone
        ",
    )
    .bind(community.id)
    .execute(&pool)
    .await
    .expect("strip everyone view");

    let (member, _) = register(
        &router,
        &format!("perm-member-{}@example.com", uuid::Uuid::now_v7()),
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
                .uri(format!(
                    "/api/v1/communities/{}/channels?space_id={}&category_id={}",
                    community.id, space.id, category.id
                ))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &member)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("list");
    assert_eq!(list.status(), StatusCode::OK);
    let listed: ChannelListResponse = serde_json::from_slice(&json_body(list).await).expect("list");
    assert!(listed.channels.is_empty());

    let get = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/channels/{}", channel.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &member)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("get");
    assert_eq!(get.status(), StatusCode::NOT_FOUND);

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn administrator_can_manage_channels_without_owner_role() {
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
        &format!("perm-admin-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Admin Perm").await;
    let space = create_space(&router, &owner, community.id, "Main").await;
    let category = create_category(&router, &owner, community.id, space.id, "General").await;

    let create_role = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/roles", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(r#"{"name":"Server Admin"}"#))
                .expect("request"),
        )
        .await
        .expect("create role");
    assert_eq!(create_role.status(), StatusCode::CREATED);
    let role: voxnexus_protocol::RoleResponse =
        serde_json::from_slice(&json_body(create_role).await).expect("role");
    sqlx::query(
        r#"
        UPDATE community_roles
        SET permissions = '{"families":{"community":1}}'::jsonb, updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(role.id)
    .execute(&pool)
    .await
    .expect("grant administrator");

    let (admin_member, admin_id) = register(
        &router,
        &format!("perm-admin-member-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let join = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/join", community.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &admin_member)
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
                    community.id, admin_id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(format!(r#"{{"role_id":"{}"}}"#, role.id)))
                .expect("request"),
        )
        .await
        .expect("assign");
    assert_eq!(assign.status(), StatusCode::NO_CONTENT);

    let create = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/channels", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &admin_member)
                .body(Body::from(format!(
                    r#"{{"name":"staff-chat","type":"text","space_id":"{}","category_id":"{}"}}"#,
                    space.id, category.id
                )))
                .expect("request"),
        )
        .await
        .expect("create channel");
    assert_eq!(create.status(), StatusCode::CREATED);

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn lower_weight_deny_hides_channel_despite_everyone_allow() {
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
        &format!("perm-weight-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Weight Hub").await;
    let space = create_space(&router, &owner, community.id, "Main").await;
    let category = create_category(&router, &owner, community.id, space.id, "General").await;
    let channel =
        create_text_channel(&router, &owner, community.id, space.id, category.id, "lobby").await;

    let create_role = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/roles", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(r#"{"name":"Muted","weight":20}"#))
                .expect("request"),
        )
        .await
        .expect("create role");
    assert_eq!(create_role.status(), StatusCode::CREATED);
    let role: voxnexus_protocol::RoleResponse =
        serde_json::from_slice(&json_body(create_role).await).expect("role");
    sqlx::query(
        r#"
        UPDATE community_roles
        SET permissions = '{"allow":{},"deny":{"text":1}}'::jsonb, updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(role.id)
    .execute(&pool)
    .await
    .expect("deny view");

    let (member, member_id) = register(
        &router,
        &format!("perm-weight-member-{}@example.com", uuid::Uuid::now_v7()),
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

    let assign = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/communities/{}/members/{}/roles",
                    community.id, member_id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(format!(r#"{{"role_id":"{}"}}"#, role.id)))
                .expect("request"),
        )
        .await
        .expect("assign");
    assert_eq!(assign.status(), StatusCode::NO_CONTENT);

    let get = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/channels/{}", channel.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &member)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("get");
    assert_eq!(get.status(), StatusCode::NOT_FOUND);

    unlock_instance_mode(&pool).await;
}
