//! Community membership (F020).

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
    error_codes, CommunityMemberListResponse, CommunityMemberResponse, CommunityResponse,
};
use voxnexus_search::{MemorySearchEngine, SearchEngine};
use voxnexus_storage::{MemoryObjectStore, ObjectStore};

fn membership_test_lock() -> &'static Mutex<()> {
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

async fn create_open_community(
    router: &axum::Router,
    cookie: &str,
    name: &str,
) -> CommunityResponse {
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
async fn second_account_joins_open_community_and_lists_members() {
    let _guard = membership_test_lock().lock().await;
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
    .expect("open create");
    let router = app(state(pool.clone(), redis));

    let owner = register(
        &router,
        &format!("owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_open_community(&router, &owner, "Join Lab").await;
    let joiner = register(
        &router,
        &format!("join-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;

    let join = router
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
        .expect("join");
    assert_eq!(join.status(), StatusCode::CREATED);
    let member: CommunityMemberResponse =
        serde_json::from_slice(&json_body(join).await).expect("member");
    assert_eq!(member.community_id, community.id);
    assert_eq!(member.role, voxnexus_domain::CommunityMemberRole::Member);

    let nick = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/communities/{}/members/me", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &joiner)
                .body(Body::from(r#"{"nickname":"Nova"}"#))
                .expect("request"),
        )
        .await
        .expect("nick");
    assert_eq!(nick.status(), StatusCode::OK);
    let updated: CommunityMemberResponse =
        serde_json::from_slice(&json_body(nick).await).expect("nick body");
    assert_eq!(updated.nickname, "Nova");

    let list = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/communities/{}/members", community.id))
                .header(header::COOKIE, &owner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("list");
    assert_eq!(list.status(), StatusCode::OK);
    let page: CommunityMemberListResponse =
        serde_json::from_slice(&json_body(list).await).expect("page");
    assert!(page.items.len() >= 2);
    assert!(page.items.iter().any(|m| m.nickname == "Nova"));

    let leave = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/leave", community.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &joiner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("leave");
    assert_eq!(leave.status(), StatusCode::NO_CONTENT);
    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn invite_mode_blocks_join_and_owner_cannot_leave() {
    let _guard = membership_test_lock().lock().await;
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
    .expect("open create");
    let router = app(state(pool.clone(), redis));

    let owner = register(
        &router,
        &format!("owner2-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_open_community(&router, &owner, "Invite Gate").await;
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

    let outsider = register(
        &router,
        &format!("out-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let denied = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/join", community.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &outsider)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("denied");
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    let body: voxnexus_protocol::ErrorBody =
        serde_json::from_slice(&json_body(denied).await).expect("error");
    assert_eq!(body.code, error_codes::PERMISSION_DENIED);

    let owner_leave = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/leave", community.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("owner leave");
    assert_eq!(owner_leave.status(), StatusCode::FORBIDDEN);
    unlock_instance_mode(&pool).await;
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn join_mode_matrix_via_http_settings() {
    let _guard = membership_test_lock().lock().await;
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
    .expect("open create");
    let router = app(state(pool.clone(), redis));

    let owner = register(
        &router,
        &format!("jm-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_open_community(&router, &owner, "Mode Matrix").await;

    let set_invite = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/communities/{}", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(r#"{"join_mode":"invite"}"#))
                .expect("request"),
        )
        .await
        .expect("patch invite");
    assert_eq!(set_invite.status(), StatusCode::OK);
    let updated: CommunityResponse =
        serde_json::from_slice(&json_body(set_invite).await).expect("community");
    assert_eq!(updated.join_mode, JoinMode::Invite);

    let outsider = register(
        &router,
        &format!("jm-out-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let denied = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/join", community.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &outsider)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("denied");
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    let reject_app = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/communities/{}", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(r#"{"join_mode":"application"}"#))
                .expect("request"),
        )
        .await
        .expect("patch application");
    assert_eq!(reject_app.status(), StatusCode::BAD_REQUEST);

    let set_discoverable = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/communities/{}", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(r#"{"discoverable_on_instance":false}"#))
                .expect("request"),
        )
        .await
        .expect("patch discoverable");
    assert_eq!(set_discoverable.status(), StatusCode::OK);
    let toggled: CommunityResponse =
        serde_json::from_slice(&json_body(set_discoverable).await).expect("community");
    assert!(!toggled.discoverable_on_instance);
    assert_eq!(toggled.join_mode, JoinMode::Invite);

    let set_open = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/communities/{}", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(r#"{"join_mode":"open"}"#))
                .expect("request"),
        )
        .await
        .expect("patch open");
    assert_eq!(set_open.status(), StatusCode::OK);

    let joined = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/join", community.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &outsider)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("join open");
    assert_eq!(joined.status(), StatusCode::CREATED);

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn owner_transfers_then_deletes_community() {
    let _guard = membership_test_lock().lock().await;
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
    .expect("open create");
    let router = app(state(pool.clone(), redis));

    let owner = register(
        &router,
        &format!("xfer-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_open_community(&router, &owner, "Transfer Me").await;
    let member = register(
        &router,
        &format!("xfer-member-{}@example.com", uuid::Uuid::now_v7()),
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
    let member_row: CommunityMemberResponse =
        serde_json::from_slice(&json_body(join).await).expect("member");

    let denied = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/transfer", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &member)
                .body(Body::from(format!(
                    r#"{{"account_id":"{}"}}"#,
                    member_row.account_id
                )))
                .expect("request"),
        )
        .await
        .expect("denied transfer");
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    let me = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/auth/me")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &member)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("me");
    let session: voxnexus_protocol::AuthSessionResponse =
        serde_json::from_slice(&json_body(me).await).expect("session");

    let transferred = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/transfer", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(format!(
                    r#"{{"account_id":"{}"}}"#,
                    session.account.id
                )))
                .expect("request"),
        )
        .await
        .expect("transfer");
    assert_eq!(transferred.status(), StatusCode::OK);
    let updated: CommunityResponse =
        serde_json::from_slice(&json_body(transferred).await).expect("community");
    assert_eq!(updated.owner_account_id, session.account.id);

    let owner_leave = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/leave", community.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("owner leave");
    assert_eq!(owner_leave.status(), StatusCode::NO_CONTENT);

    let deleted = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/delete", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &member)
                .body(Body::from(r#"{"confirm_name":"Transfer Me"}"#))
                .expect("request"),
        )
        .await
        .expect("delete");
    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

    let get = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/communities/{}", community.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &member)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("get");
    assert_eq!(get.status(), StatusCode::NOT_FOUND);

    let other = create_open_community(&router, &owner, "Still Here").await;
    let other_get = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/communities/{}", other.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("other get");
    assert_eq!(other_get.status(), StatusCode::OK);

    unlock_instance_mode(&pool).await;
}
