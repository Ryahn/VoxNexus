//! Spaces (F022).

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
use voxnexus_protocol::{CommunityResponse, SpaceListResponse, SpaceResponse};
use voxnexus_search::{MemorySearchEngine, SearchEngine};
use voxnexus_storage::{MemoryObjectStore, ObjectStore};

const INSTANCE_MODE_LOCK: i64 = 0x0190_0000_0000_0022;

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
                    r#"{{"name":"{name}","description":"spaces test"}}"#
                )))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::CREATED);
    serde_json::from_slice(&json_body(response).await).expect("community")
}

#[tokio::test]
async fn owner_creates_two_spaces_in_one_community() {
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
    let _result = async {
        update_instance(
            &pool,
            InstancePatch {
                community_creation_mode: Some(CommunityCreationMode::Open),
                ..InstancePatch::default()
            },
        )
        .await
        .expect("open mode");

        let email = format!("spaces-owner-{}@example.com", uuid::Uuid::now_v7());
        let router = app(state(pool.clone(), redis));
        let cookie = register(&router, &email).await;
        let community = create_community(&router, &cookie, "Space Hub").await;

        let mut created = Vec::new();
        for name in ["Lobby", "Dev"] {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(format!("/api/v1/communities/{}/spaces", community.id))
                        .header(header::CONTENT_TYPE, "application/json")
                        .header(header::ORIGIN, "http://127.0.0.1:8080")
                        .header(header::COOKIE, &cookie)
                        .body(Body::from(format!(
                            r#"{{"name":"{name}","description":"{name} space","topic":"chat","game":"","visibility":"open"}}"#
                        )))
                        .expect("request"),
                )
                .await
                .expect("oneshot");
            assert_eq!(response.status(), StatusCode::CREATED);
            let space: SpaceResponse =
                serde_json::from_slice(&json_body(response).await).expect("space");
            assert_eq!(space.community_id, community.id);
            assert_eq!(space.name, name);
            created.push(space);
        }

        assert_eq!(created.len(), 2);
        assert_ne!(created[0].id, created[1].id);
        assert_eq!(created[0].community_id, created[1].community_id);

        let list = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/communities/{}/spaces", community.id))
                    .header(header::ORIGIN, "http://127.0.0.1:8080")
                    .header(header::COOKIE, &cookie)
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("oneshot");
        assert_eq!(list.status(), StatusCode::OK);
        let listed: SpaceListResponse =
            serde_json::from_slice(&json_body(list).await).expect("list");
        assert_eq!(listed.spaces.len(), 2);
        assert!(listed.spaces.iter().all(|s| s.community_id == community.id));

        // Schema has no parent — belonging is only community_id (cannot nest).
        let cols: i64 = sqlx::query_scalar(
            r"
            SELECT COUNT(*)::bigint
            FROM information_schema.columns
            WHERE table_name = 'spaces' AND column_name = 'parent_space_id'
            ",
        )
        .fetch_one(&pool)
        .await
        .expect("column check");
        assert_eq!(cols, 0, "spaces must not support nesting via parent_space_id");
    }
    .await;
    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn non_owner_cannot_create_space() {
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
    let _result = async {
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
            &format!("space-owner-{}@example.com", uuid::Uuid::now_v7()),
        )
        .await;
        let community = create_community(&router, &owner, "Gated").await;
        let member = register(
            &router,
            &format!("space-member-{}@example.com", uuid::Uuid::now_v7()),
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
            .expect("oneshot");
        assert_eq!(join.status(), StatusCode::CREATED);

        let denied = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/communities/{}/spaces", community.id))
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::ORIGIN, "http://127.0.0.1:8080")
                    .header(header::COOKIE, &member)
                    .body(Body::from(r#"{"name":"Nope"}"#))
                    .expect("request"),
            )
            .await
            .expect("oneshot");
        assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    }
    .await;
    unlock_instance_mode(&pool).await;
    let _ = _result;
}
