//! Community invites (F021).

use std::sync::{Arc, OnceLock};

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tokio::sync::Mutex;
use tower::ServiceExt;
use voxnexus::http::{app, AppState};
use voxnexus_auth::{
    session_cookie_name, update_community, update_instance, CommunityPatch, InstancePatch,
};
use voxnexus_db::{connect_and_migrate, test_database_url, PgPool};
use voxnexus_domain::{CommunityCreationMode, JoinMode};
use voxnexus_jobs::{connect, RedisConn};
use voxnexus_protocol::{
    error_codes, CommunityMemberResponse, CommunityResponse, InviteListResponse, InviteResponse,
};
use voxnexus_search::{MemorySearchEngine, SearchEngine};
use voxnexus_storage::{MemoryObjectStore, ObjectStore};

fn invite_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

const INSTANCE_MODE_LOCK: i64 = 0x0190_0000_0000_0019;

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
                .body(Body::from(format!(r#"{{"name":"{name}"}}"#)))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::CREATED);
    serde_json::from_slice(&json_body(response).await).expect("community")
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn invite_accept_joins_invite_only_community() {
    let _guard = invite_test_lock().lock().await;
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
    .expect("mode");
    let router = app(state(pool.clone(), redis));
    let owner = register(
        &router,
        &format!("inv-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Invite Only Club").await;
    update_community(
        &pool,
        community.id,
        CommunityPatch {
            join_mode: Some(JoinMode::Invite),
            ..CommunityPatch::default()
        },
    )
    .await
    .expect("invite mode");

    let create = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/invites", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(r#"{"max_uses":2}"#))
                .expect("request"),
        )
        .await
        .expect("create invite");
    let create_status = create.status();
    let create_body = json_body(create).await;
    assert_eq!(create_status, StatusCode::CREATED);
    let invite: InviteResponse = serde_json::from_slice(&create_body).expect("invite");

    let joiner = register(
        &router,
        &format!("inv-join-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let denied = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/join", community.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &joiner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("open join denied");
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    let accepted = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/invites/{}/accept", invite.code))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &joiner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("accept");
    assert_eq!(accepted.status(), StatusCode::CREATED);
    let member: CommunityMemberResponse =
        serde_json::from_slice(&json_body(accepted).await).expect("member");
    assert_eq!(member.community_id, community.id);

    let listed = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/communities/{}/invites", community.id))
                .header(header::COOKIE, &owner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("list");
    assert_eq!(listed.status(), StatusCode::OK);
    let page: InviteListResponse = serde_json::from_slice(&json_body(listed).await).expect("list");
    assert_eq!(page.invites[0].uses, 1);
    unlock_instance_mode(&pool).await;
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn expired_max_uses_and_paused_invites_fail() {
    let _guard = invite_test_lock().lock().await;
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
    .expect("mode");
    let router = app(state(pool.clone(), redis));
    let owner = register(
        &router,
        &format!("inv2-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Limits Club").await;

    let expired_create = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/invites", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(r#"{"expire_after":{"unit":"hours","value":1}}"#))
                .expect("request"),
        )
        .await
        .expect("expired invite");
    assert_eq!(expired_create.status(), StatusCode::CREATED);
    let expired: InviteResponse =
        serde_json::from_slice(&json_body(expired_create).await).expect("expired");
    // Force-expire in DB so accept hits InviteExpired without waiting.
    sqlx::query(
        r"
        UPDATE community_invites
        SET expires_at = NOW() - INTERVAL '1 hour'
        WHERE id = $1
        ",
    )
    .bind(expired.id)
    .execute(&pool)
    .await
    .expect("force expire");

    let user_a = register(
        &router,
        &format!("inv2-a-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let expired_accept = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/invites/{}/accept", expired.code))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &user_a)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("expired accept");
    assert_eq!(expired_accept.status(), StatusCode::FORBIDDEN);
    let body: voxnexus_protocol::ErrorBody =
        serde_json::from_slice(&json_body(expired_accept).await).expect("error");
    assert_eq!(body.code, error_codes::PERMISSION_DENIED);

    let limited = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/invites", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(r#"{"max_uses":1}"#))
                .expect("request"),
        )
        .await
        .expect("limited");
    let limited_invite: InviteResponse =
        serde_json::from_slice(&json_body(limited).await).expect("limited invite");
    let accept1 = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/invites/{}/accept", limited_invite.code))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &user_a)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("accept1");
    assert_eq!(accept1.status(), StatusCode::CREATED);
    let user_b = register(
        &router,
        &format!("inv2-b-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let accept2 = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/invites/{}/accept", limited_invite.code))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &user_b)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("accept2");
    assert_eq!(accept2.status(), StatusCode::FORBIDDEN);

    let pauseable = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/invites", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from("{}"))
                .expect("request"),
        )
        .await
        .expect("pauseable");
    let pause_invite: InviteResponse =
        serde_json::from_slice(&json_body(pauseable).await).expect("pause invite");
    let paused = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!(
                    "/api/v1/communities/{}/invites/{}",
                    community.id, pause_invite.id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(r#"{"paused":true}"#))
                .expect("request"),
        )
        .await
        .expect("pause");
    assert_eq!(paused.status(), StatusCode::OK);
    let pause_accept = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/invites/{}/accept", pause_invite.code))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &user_b)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("pause accept");
    assert_eq!(pause_accept.status(), StatusCode::FORBIDDEN);
    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn expire_after_and_max_uses_limits_are_enforced() {
    let _guard = invite_test_lock().lock().await;
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
    .expect("mode");
    let router = app(state(pool.clone(), redis));
    let owner = register(
        &router,
        &format!("inv3-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Limits UI Club").await;

    let too_many_hours = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/invites", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(
                    r#"{"expire_after":{"unit":"hours","value":25}}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("hours");
    assert_eq!(too_many_hours.status(), StatusCode::BAD_REQUEST);

    let too_many_uses = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/invites", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(r#"{"max_uses":1001}"#))
                .expect("request"),
        )
        .await
        .expect("uses");
    assert!(
        too_many_uses.status() == StatusCode::UNPROCESSABLE_ENTITY
            || too_many_uses.status() == StatusCode::BAD_REQUEST,
        "status={}",
        too_many_uses.status()
    );

    let ok = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/invites", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(
                    r#"{"max_uses":1000,"expire_after":{"unit":"months","value":3}}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("ok");
    assert_eq!(ok.status(), StatusCode::CREATED);
    unlock_instance_mode(&pool).await;
}
