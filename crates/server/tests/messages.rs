//! Channel messages (F034) + gateway fanout (F035).

#![allow(clippy::too_many_lines)]

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use futures_util::{SinkExt, StreamExt};
use http_body_util::BodyExt;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::Message;
use tower::ServiceExt;
use voxnexus::http::{app, AppState};
use voxnexus_auth::{session_cookie_name, update_instance, InstancePatch};
use voxnexus_db::{connect_and_migrate, test_database_url, PgPool};
use voxnexus_domain::CommunityCreationMode;
use voxnexus_jobs::{connect, RedisConn};
use voxnexus_protocol::{
    CategoryResponse, ChannelResponse, CommunityResponse, Envelope, EventType, IdentifyPayload,
    MessageCreatePayload, MessageListResponse, MessageResponse, SpaceResponse, GATEWAY_SUBPROTOCOL,
};
use voxnexus_search::{MemorySearchEngine, SearchEngine};
use voxnexus_storage::{MemoryObjectStore, ObjectStore};

const INSTANCE_MODE_LOCK: i64 = 0x0190_0000_0000_0027;

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
                    r#"{{"email":"{email}","password":"password123","display_name":"Msg User"}}"#
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
                    r#"{{"name":"{name}","description":"message test"}}"#
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

async fn post_message(
    router: &axum::Router,
    cookie: &str,
    channel_id: uuid::Uuid,
    content: &str,
    nonce: Option<&str>,
) -> axum::response::Response {
    let body = if let Some(nonce) = nonce {
        format!(r#"{{"content":"{content}","nonce":"{nonce}"}}"#)
    } else {
        format!(r#"{{"content":"{content}"}}"#)
    };
    router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/channels/{channel_id}/messages"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, cookie)
                .body(Body::from(body))
                .expect("request"),
        )
        .await
        .expect("oneshot")
}

#[tokio::test]
async fn members_can_send_and_list_messages() {
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
        &format!("msg-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let member = register(
        &router,
        &format!("msg-member-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Msg Hub").await;
    let space = create_space(&router, &owner, community.id, "Main").await;
    let category = create_category(&router, &owner, community.id, space.id, "General").await;
    let channel = create_text_channel(
        &router,
        &owner,
        community.id,
        space.id,
        category.id,
        "general",
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

    let first = post_message(&router, &owner, channel.id, "hello from owner", Some("n1")).await;
    assert_eq!(first.status(), StatusCode::CREATED);
    let first_msg: MessageResponse =
        serde_json::from_slice(&json_body(first).await).expect("first msg");

    let replay = post_message(&router, &owner, channel.id, "hello from owner", Some("n1")).await;
    assert_eq!(replay.status(), StatusCode::OK);
    let replay_msg: MessageResponse =
        serde_json::from_slice(&json_body(replay).await).expect("replay");
    assert_eq!(replay_msg.id, first_msg.id);

    let second = post_message(&router, &member, channel.id, "hello from member", None).await;
    assert_eq!(second.status(), StatusCode::CREATED);

    let list = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/channels/{}/messages?limit=50", channel.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &member)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("list");
    assert_eq!(list.status(), StatusCode::OK);
    let page: MessageListResponse = serde_json::from_slice(&json_body(list).await).expect("page");
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].content, "hello from member");
    assert_eq!(page.items[1].content, "hello from owner");
    assert!(!page.has_more);

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn no_view_lists_404_and_no_send_posts_403() {
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
        &format!("msg-deny-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let member = register(
        &router,
        &format!("msg-deny-member-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Msg Deny").await;
    let space = create_space(&router, &owner, community.id, "Main").await;
    let category = create_category(&router, &owner, community.id, space.id, "General").await;
    let channel = create_text_channel(
        &router,
        &owner,
        community.id,
        space.id,
        category.id,
        "locked",
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

    let roles = router
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
        .expect("roles");
    let role_list: voxnexus_protocol::RoleListResponse =
        serde_json::from_slice(&json_body(roles).await).expect("roles");
    let everyone = role_list
        .roles
        .iter()
        .find(|role| role.is_everyone)
        .expect("@everyone");

    // Deny send only (bit 2) — member can still view.
    let deny_send = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/api/v1/channels/{}/permission-overrides/roles/{}",
                    channel.id, everyone.id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(
                    r#"{"permissions":{"allow":{},"deny":{"text":2}}}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("deny send");
    assert_eq!(deny_send.status(), StatusCode::OK);

    let forbidden = post_message(&router, &member, channel.id, "nope", None).await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    let list_ok = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/channels/{}/messages", channel.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &member)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("list");
    assert_eq!(list_ok.status(), StatusCode::OK);

    // Deny view (bit 1) — list returns 404.
    let deny_view = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/api/v1/channels/{}/permission-overrides/roles/{}",
                    channel.id, everyone.id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(
                    r#"{"permissions":{"allow":{},"deny":{"text":1}}}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("deny view");
    assert_eq!(deny_view.status(), StatusCode::OK);

    let list_hidden = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/channels/{}/messages", channel.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &member)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("list hidden");
    assert_eq!(list_hidden.status(), StatusCode::NOT_FOUND);

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn message_pagination_is_newest_first() {
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
        &format!("msg-page-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Msg Page").await;
    let space = create_space(&router, &owner, community.id, "Main").await;
    let category = create_category(&router, &owner, community.id, space.id, "General").await;
    let channel =
        create_text_channel(&router, &owner, community.id, space.id, category.id, "log").await;

    for i in 1..=3 {
        let response = post_message(&router, &owner, channel.id, &format!("m{i}"), None).await;
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let page1 = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/channels/{}/messages?limit=2", channel.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("page1");
    assert_eq!(page1.status(), StatusCode::OK);
    let page1: MessageListResponse = serde_json::from_slice(&json_body(page1).await).expect("p1");
    assert_eq!(page1.items.len(), 2);
    assert!(page1.has_more);
    assert_eq!(page1.items[0].content, "m3");
    assert_eq!(page1.items[1].content, "m2");

    let before = page1.items[1].id;
    let page2 = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/channels/{}/messages?limit=2&before={before}",
                    channel.id
                ))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("page2");
    assert_eq!(page2.status(), StatusCode::OK);
    let page2: MessageListResponse = serde_json::from_slice(&json_body(page2).await).expect("p2");
    assert_eq!(page2.items.len(), 1);
    assert!(!page2.has_more);
    assert_eq!(page2.items[0].content, "m1");

    unlock_instance_mode(&pool).await;
}

async fn connect_gateway(
    addr: std::net::SocketAddr,
    cookie: &str,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let mut request = format!("ws://{addr}/api/v1/gateway")
        .into_client_request()
        .expect("request");
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        GATEWAY_SUBPROTOCOL.parse().expect("protocol header"),
    );
    request
        .headers_mut()
        .insert(header::COOKIE, cookie.parse().expect("cookie"));
    tokio::time::timeout(
        Duration::from_secs(5),
        tokio_tungstenite::connect_async(request),
    )
    .await
    .expect("connect timeout")
    .expect("connect")
    .0
}

async fn identify_gateway(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) {
    let _hello = tokio::time::timeout(Duration::from_secs(5), ws.next())
        .await
        .expect("hello timeout")
        .expect("hello")
        .expect("ok");
    ws.send(Message::Text(
        serde_json::to_string(&Envelope::new(0, EventType::Identify, IdentifyPayload {}))
            .expect("ser")
            .into(),
    ))
    .await
    .expect("identify");
    let _ready = tokio::time::timeout(Duration::from_secs(5), ws.next())
        .await
        .expect("ready timeout")
        .expect("ready")
        .expect("ok");
    let sync_msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
        .await
        .expect("sync timeout")
        .expect("sync")
        .expect("ok");
    let Message::Text(sync_text) = sync_msg else {
        panic!("sync");
    };
    let sync: Envelope = serde_json::from_str(&sync_text).expect("sync");
    assert_eq!(sync.event_type, EventType::PresenceSync);
}

async fn next_message_create(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> MessageCreatePayload {
    loop {
        let frame = tokio::time::timeout(Duration::from_secs(5), ws.next())
            .await
            .expect("frame timeout")
            .expect("frame")
            .expect("ok");
        let Message::Text(text) = frame else {
            continue;
        };
        let envelope: Envelope = serde_json::from_str(&text).expect("envelope");
        if envelope.event_type == EventType::MessageCreate {
            let scope = envelope.scope.expect("channel scope");
            assert_eq!(scope.scope_type, "channel");
            return serde_json::from_value(envelope.payload).expect("payload");
        }
    }
}

#[tokio::test]
async fn two_clients_receive_message_create_realtime() {
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
        &format!("msg-rt-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let member = register(
        &router,
        &format!("msg-rt-member-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Msg RT").await;
    let space = create_space(&router, &owner, community.id, "Main").await;
    let category = create_category(&router, &owner, community.id, space.id, "General").await;
    let channel =
        create_text_channel(&router, &owner, community.id, space.id, category.id, "live").await;

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

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });

    let mut owner_ws = connect_gateway(addr, &owner).await;
    let mut member_ws = connect_gateway(addr, &member).await;
    identify_gateway(&mut owner_ws).await;
    identify_gateway(&mut member_ws).await;

    let http = reqwest::Client::new();
    let post = http
        .post(format!(
            "http://{addr}/api/v1/channels/{}/messages",
            channel.id
        ))
        .header(header::ORIGIN, "http://127.0.0.1:8080")
        .header(header::COOKIE, &owner)
        .header(header::CONTENT_TYPE, "application/json")
        .body(r#"{"content":"realtime hello"}"#)
        .send()
        .await
        .expect("post");
    assert_eq!(post.status(), StatusCode::CREATED);

    let for_owner = next_message_create(&mut owner_ws).await;
    let for_member = next_message_create(&mut member_ws).await;
    assert_eq!(for_owner.content, "realtime hello");
    assert_eq!(for_member.content, "realtime hello");
    assert_eq!(for_owner.channel_id, channel.id);
    assert_eq!(for_member.id, for_owner.id);

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn hidden_channel_member_does_not_receive_message_create() {
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
        &format!("msg-hide-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let member = register(
        &router,
        &format!("msg-hide-member-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Msg Hide").await;
    let space = create_space(&router, &owner, community.id, "Main").await;
    let category = create_category(&router, &owner, community.id, space.id, "General").await;
    let channel = create_text_channel(
        &router,
        &owner,
        community.id,
        space.id,
        category.id,
        "secret",
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

    let roles = router
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
        .expect("roles");
    let role_list: voxnexus_protocol::RoleListResponse =
        serde_json::from_slice(&json_body(roles).await).expect("roles");
    let everyone = role_list
        .roles
        .iter()
        .find(|role| role.is_everyone)
        .expect("@everyone");

    let deny_view = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/api/v1/channels/{}/permission-overrides/roles/{}",
                    channel.id, everyone.id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(
                    r#"{"permissions":{"allow":{},"deny":{"text":1}}}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("deny view");
    assert_eq!(deny_view.status(), StatusCode::OK);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });

    let mut owner_ws = connect_gateway(addr, &owner).await;
    let mut member_ws = connect_gateway(addr, &member).await;
    identify_gateway(&mut owner_ws).await;
    identify_gateway(&mut member_ws).await;

    let http = reqwest::Client::new();
    let post = http
        .post(format!(
            "http://{addr}/api/v1/channels/{}/messages",
            channel.id
        ))
        .header(header::ORIGIN, "http://127.0.0.1:8080")
        .header(header::COOKIE, &owner)
        .header(header::CONTENT_TYPE, "application/json")
        .body(r#"{"content":"secret note"}"#)
        .send()
        .await
        .expect("post");
    assert_eq!(post.status(), StatusCode::CREATED);

    let for_owner = next_message_create(&mut owner_ws).await;
    assert_eq!(for_owner.content, "secret note");

    let leaked = tokio::time::timeout(Duration::from_millis(800), member_ws.next()).await;
    if let Ok(Some(Ok(Message::Text(text)))) = leaked {
        let envelope: Envelope = serde_json::from_str(&text).expect("envelope");
        assert_ne!(
            envelope.event_type,
            EventType::MessageCreate,
            "hidden member must not receive MESSAGE_CREATE"
        );
    }

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn non_author_cannot_edit_message() {
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
        &format!("msg-edit-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let member = register(
        &router,
        &format!("msg-edit-member-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Msg Edit").await;
    let space = create_space(&router, &owner, community.id, "Main").await;
    let category = create_category(&router, &owner, community.id, space.id, "General").await;
    let channel =
        create_text_channel(&router, &owner, community.id, space.id, category.id, "edit").await;

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

    let created = post_message(&router, &owner, channel.id, "original", None).await;
    assert_eq!(created.status(), StatusCode::CREATED);
    let msg: MessageResponse = serde_json::from_slice(&json_body(created).await).expect("msg");

    let forbidden = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!(
                    "/api/v1/channels/{}/messages/{}",
                    channel.id, msg.id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &member)
                .body(Body::from(r#"{"content":"hijack"}"#))
                .expect("request"),
        )
        .await
        .expect("patch");
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    let ok = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!(
                    "/api/v1/channels/{}/messages/{}",
                    channel.id, msg.id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(r#"{"content":"revised"}"#))
                .expect("request"),
        )
        .await
        .expect("patch ok");
    assert_eq!(ok.status(), StatusCode::OK);
    let edited: MessageResponse = serde_json::from_slice(&json_body(ok).await).expect("edited");
    assert_eq!(edited.content, "revised");
    assert!(edited.edited_at.is_some());

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn manage_messages_can_delete_others() {
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
        &format!("msg-del-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let mod_user = register(
        &router,
        &format!("msg-del-mod-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Msg Del").await;
    let space = create_space(&router, &owner, community.id, "Main").await;
    let category = create_category(&router, &owner, community.id, space.id, "General").await;
    let channel =
        create_text_channel(&router, &owner, community.id, space.id, category.id, "mod").await;

    let join = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/join", community.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &mod_user)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("join");
    assert_eq!(join.status(), StatusCode::CREATED);

    // text.manage_messages = 1 << 12 = 4096
    let create_role = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/communities/{}/roles", community.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(
                    r#"{"name":"Mod","weight":50,"permissions":{"allow":{"text":4096},"deny":{}}}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("role");
    assert_eq!(create_role.status(), StatusCode::CREATED);
    let role: voxnexus_protocol::RoleResponse =
        serde_json::from_slice(&json_body(create_role).await).expect("role");

    let me = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/auth/me")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &mod_user)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("me");
    let session: voxnexus_protocol::AuthSessionResponse =
        serde_json::from_slice(&json_body(me).await).expect("session");

    let assign = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/communities/{}/members/{}/roles",
                    community.id, session.account.id
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

    let created = post_message(&router, &owner, channel.id, "to delete", None).await;
    assert_eq!(created.status(), StatusCode::CREATED);
    let msg: MessageResponse = serde_json::from_slice(&json_body(created).await).expect("msg");

    let deleted = router
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/channels/{}/messages/{}",
                    channel.id, msg.id
                ))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &mod_user)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("delete");
    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

    let list = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/channels/{}/messages", channel.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("list");
    assert_eq!(list.status(), StatusCode::OK);
    let page: MessageListResponse = serde_json::from_slice(&json_body(list).await).expect("page");
    assert!(page.items.iter().all(|item| item.id != msg.id));

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn reply_requires_same_channel_and_returns_preview() {
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
        &format!("msg-reply-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Msg Reply").await;
    let space = create_space(&router, &owner, community.id, "Main").await;
    let category = create_category(&router, &owner, community.id, space.id, "General").await;
    let channel_a = create_text_channel(
        &router,
        &owner,
        community.id,
        space.id,
        category.id,
        "alpha",
    )
    .await;
    let channel_b =
        create_text_channel(&router, &owner, community.id, space.id, category.id, "beta").await;

    let parent_res = post_message(&router, &owner, channel_a.id, "parent line", None).await;
    assert_eq!(parent_res.status(), StatusCode::CREATED);
    let parent: MessageResponse =
        serde_json::from_slice(&json_body(parent_res).await).expect("parent");

    let other_res = post_message(&router, &owner, channel_b.id, "other channel", None).await;
    assert_eq!(other_res.status(), StatusCode::CREATED);
    let other: MessageResponse =
        serde_json::from_slice(&json_body(other_res).await).expect("other");

    let cross = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/channels/{}/messages", channel_a.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(format!(
                    r#"{{"content":"cross","referenced_message_id":"{}"}}"#,
                    other.id
                )))
                .expect("request"),
        )
        .await
        .expect("cross");
    assert_eq!(cross.status(), StatusCode::BAD_REQUEST);

    let reply_res = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/channels/{}/messages", channel_a.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::from(format!(
                    r#"{{"content":"reply body","referenced_message_id":"{}"}}"#,
                    parent.id
                )))
                .expect("request"),
        )
        .await
        .expect("reply");
    assert_eq!(reply_res.status(), StatusCode::CREATED);
    let reply: MessageResponse =
        serde_json::from_slice(&json_body(reply_res).await).expect("reply");
    assert_eq!(reply.referenced_message_id, Some(parent.id));
    let preview = reply.reply_to.expect("reply_to");
    assert_eq!(preview.message_id, parent.id);
    assert_eq!(preview.excerpt, "parent line");
    assert!(!preview.deleted);

    let list = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/channels/{}/messages", channel_a.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("list");
    assert_eq!(list.status(), StatusCode::OK);
    let page: MessageListResponse = serde_json::from_slice(&json_body(list).await).expect("page");
    let listed = page
        .items
        .iter()
        .find(|item| item.id == reply.id)
        .expect("listed reply");
    assert_eq!(
        listed.reply_to.as_ref().map(|p| p.message_id),
        Some(parent.id)
    );

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn attachment_rejects_executable_and_hides_download() {
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
        &format!("msg-att-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let outsider = register(
        &router,
        &format!("msg-att-out-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Msg Attach").await;
    let space = create_space(&router, &owner, community.id, "Main").await;
    let category = create_category(&router, &owner, community.id, space.id, "General").await;
    let channel = create_text_channel(
        &router,
        &owner,
        community.id,
        space.id,
        category.id,
        "files",
    )
    .await;

    let exe = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/channels/{}/attachments", channel.id))
                .header(header::CONTENT_TYPE, "application/octet-stream")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .header("x-filename", "payload.exe")
                .body(Body::from(b"MZ\0\0not-a-real-pe".as_slice()))
                .expect("request"),
        )
        .await
        .expect("exe");
    assert_eq!(exe.status(), StatusCode::BAD_REQUEST);

    let upload = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/channels/{}/attachments", channel.id))
                .header(header::CONTENT_TYPE, "application/octet-stream")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .header("x-filename", "note.txt")
                .body(Body::from("hello attachment"))
                .expect("request"),
        )
        .await
        .expect("upload");
    assert_eq!(upload.status(), StatusCode::CREATED);
    let att: voxnexus_protocol::AttachmentResponse =
        serde_json::from_slice(&json_body(upload).await).expect("att");
    assert_eq!(att.filename, "note.txt");

    let denied = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/attachments/{}", att.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &outsider)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("denied");
    assert_eq!(denied.status(), StatusCode::NOT_FOUND);

    let ok = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/attachments/{}", att.id))
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, &owner)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("download");
    assert_eq!(ok.status(), StatusCode::OK);
    let bytes = json_body(ok).await;
    assert_eq!(bytes.as_slice(), b"hello attachment");

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn everyone_mention_without_permission_is_rejected() {
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
        &format!("msg-mention-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let member = register(
        &router,
        &format!("msg-mention-member-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Msg Mentions").await;
    let space = create_space(&router, &owner, community.id, "Main").await;
    let category = create_category(&router, &owner, community.id, space.id, "General").await;
    let channel =
        create_text_channel(&router, &owner, community.id, space.id, category.id, "chat").await;

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

    let denied = post_message(&router, &member, channel.id, "ping @everyone", None).await;
    assert_eq!(denied.status(), StatusCode::BAD_REQUEST);

    let allowed = post_message(&router, &owner, channel.id, "ping @everyone", None).await;
    assert_eq!(allowed.status(), StatusCode::CREATED);
    let msg: MessageResponse = serde_json::from_slice(&json_body(allowed).await).expect("msg");
    assert!(msg.mentions.everyone);

    unlock_instance_mode(&pool).await;
}

#[tokio::test]
async fn typing_start_fans_out_rate_limited_and_scoped() {
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
        &format!("msg-typing-owner-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let member = register(
        &router,
        &format!("msg-typing-member-{}@example.com", uuid::Uuid::now_v7()),
    )
    .await;
    let community = create_community(&router, &owner, "Msg Typing").await;
    let space = create_space(&router, &owner, community.id, "Main").await;
    let category = create_category(&router, &owner, community.id, space.id, "General").await;
    let channel =
        create_text_channel(&router, &owner, community.id, space.id, category.id, "chat").await;
    let other = create_text_channel(
        &router,
        &owner,
        community.id,
        space.id,
        category.id,
        "other",
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

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });

    let mut owner_ws = connect_gateway(addr, &owner).await;
    let mut member_ws = connect_gateway(addr, &member).await;
    identify_gateway(&mut owner_ws).await;
    identify_gateway(&mut member_ws).await;

    let typing = Envelope::new(
        0,
        EventType::TypingStart,
        voxnexus_protocol::TypingStartRequest {
            channel_id: channel.id,
        },
    );
    owner_ws
        .send(Message::Text(
            serde_json::to_string(&typing).expect("ser").into(),
        ))
        .await
        .expect("send typing");

    let payload = next_typing_start(&mut member_ws).await;
    assert_eq!(payload.channel_id, channel.id);
    assert_ne!(payload.display_name, "");

    // Immediate second pulse is rate-limited (no second event).
    owner_ws
        .send(Message::Text(
            serde_json::to_string(&typing).expect("ser").into(),
        ))
        .await
        .expect("send typing again");
    let second = tokio::time::timeout(Duration::from_millis(400), member_ws.next()).await;
    assert!(second.is_err(), "rate-limited typing should not fan out");

    // Typing for another channel is ignored by UI clients watching `channel`
    // (server still fans to viewers; assert payload channel_id differs).
    let other_typing = Envelope::new(
        0,
        EventType::TypingStart,
        voxnexus_protocol::TypingStartRequest {
            channel_id: other.id,
        },
    );
    // Cooldown is per channel — other channel is allowed.
    owner_ws
        .send(Message::Text(
            serde_json::to_string(&other_typing).expect("ser").into(),
        ))
        .await
        .expect("send other typing");
    let other_payload = next_typing_start(&mut member_ws).await;
    assert_eq!(other_payload.channel_id, other.id);
    assert_ne!(
        other_payload.channel_id, channel.id,
        "wrong-channel typing must carry the other channel id"
    );

    unlock_instance_mode(&pool).await;
}

async fn next_typing_start(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> voxnexus_protocol::TypingStartPayload {
    loop {
        let frame = tokio::time::timeout(Duration::from_secs(5), ws.next())
            .await
            .expect("frame timeout")
            .expect("frame")
            .expect("ok");
        let Message::Text(text) = frame else {
            continue;
        };
        let envelope: Envelope = serde_json::from_str(&text).expect("envelope");
        if envelope.event_type == EventType::TypingStart {
            let scope = envelope.scope.expect("channel scope");
            assert_eq!(scope.scope_type, "channel");
            return serde_json::from_value(envelope.payload).expect("payload");
        }
    }
}
