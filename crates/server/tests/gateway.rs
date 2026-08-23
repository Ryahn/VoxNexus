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
use voxnexus_auth::session_cookie_name;
use voxnexus_db::{connect_and_migrate, test_database_url, PgPool};
use voxnexus_jobs::{connect, RedisConn};
use voxnexus_protocol::{
    error_codes, Envelope, ErrorBody, EventType, HeartbeatPayload, HelloPayload, IdentifyPayload,
    ReadyPayload, ResumePayload, ResumedPayload, GATEWAY_SUBPROTOCOL,
};
use voxnexus_realtime::ResumeStore;
use voxnexus_search::{MemorySearchEngine, SearchEngine};
use voxnexus_storage::{MemoryObjectStore, ObjectStore};

async fn test_redis() -> Option<RedisConn> {
    let url = std::env::var("REDIS_URL_TEST")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "redis://127.0.0.1:6379".to_owned());
    connect(&url).await.ok()
}

fn state(pool: PgPool, redis: RedisConn, allow_dev_ping: bool, heartbeat: Duration) -> AppState {
    AppState {
        pool,
        metrics_enabled: false,
        public_url: "http://127.0.0.1:8080".parse().expect("url"),
        cookie_secure: false,
        community_creation_mode_locked: false,
        gateway_allow_unauth: allow_dev_ping,
        gateway_heartbeat_interval: heartbeat,
        storage: Arc::new(MemoryObjectStore::new_ready()) as Arc<dyn ObjectStore>,
        redis,
        search: Arc::new(MemorySearchEngine::new_ready()) as Arc<dyn SearchEngine>,
        web_dist: None,
        resume_store: Arc::new(ResumeStore::new()),
        presence_hub: Arc::new(voxnexus_realtime::PresenceHub::with_default_grace()),
    }
}

async fn register_cookie(router: axum::Router, email: &str) -> String {
    let response = router
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
    let set_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .expect("set-cookie")
        .to_str()
        .expect("str");
    let name = session_cookie_name(false);
    let token = set_cookie
        .split(';')
        .next()
        .expect("pair")
        .strip_prefix(&format!("{name}="))
        .expect("token");
    format!("{name}={token}")
}

#[tokio::test]
async fn gateway_refused_without_session_cookie() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let response = app(state(pool, redis, false, Duration::from_secs(15)))
        .oneshot(
            Request::builder()
                .uri("/api/v1/gateway")
                .header("connection", "upgrade")
                .header("upgrade", "websocket")
                .header("sec-websocket-version", "13")
                .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let body: ErrorBody = serde_json::from_slice(&bytes).expect("error body");
    assert_eq!(body.code, error_codes::UNAUTHENTICATED);
}

#[tokio::test]
async fn gateway_identify_ready_and_heartbeat() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let email = format!("gw-{}@example.com", uuid::Uuid::now_v7());
    let router = app(state(pool, redis, true, Duration::from_secs(15)));
    let cookie = register_cookie(router.clone(), &email).await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });

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

    let (mut ws, _) = tokio_tungstenite::connect_async(request)
        .await
        .expect("connect");

    let hello_msg = ws.next().await.expect("hello frame").expect("ok");
    let Message::Text(text) = hello_msg else {
        panic!("expected text hello, got {hello_msg:?}");
    };
    let hello: Envelope = serde_json::from_str(&text).expect("hello envelope");
    assert_eq!(hello.event_type, EventType::Hello);
    let hello_payload: HelloPayload = serde_json::from_value(hello.payload).expect("hello");

    let identify = Envelope::new(0, EventType::Identify, IdentifyPayload {});
    ws.send(Message::Text(
        serde_json::to_string(&identify).expect("ser").into(),
    ))
    .await
    .expect("send identify");

    let ready_msg = ws.next().await.expect("ready frame").expect("ok");
    let Message::Text(ready_text) = ready_msg else {
        panic!("expected text ready, got {ready_msg:?}");
    };
    let ready: Envelope = serde_json::from_str(&ready_text).expect("ready envelope");
    assert_eq!(ready.event_type, EventType::Ready);
    let ready_payload: ReadyPayload = serde_json::from_value(ready.payload).expect("ready");
    assert_eq!(ready_payload.session_id, hello_payload.session_id);
    assert!(!ready_payload.resume_token.is_empty());

    let sync_msg = ws.next().await.expect("sync frame").expect("ok");
    let Message::Text(sync_text) = sync_msg else {
        panic!("expected text sync, got {sync_msg:?}");
    };
    let sync: Envelope = serde_json::from_str(&sync_text).expect("sync envelope");
    assert_eq!(sync.event_type, EventType::PresenceSync);

    let heartbeat = Envelope::new(0, EventType::Heartbeat, HeartbeatPayload {});
    ws.send(Message::Text(
        serde_json::to_string(&heartbeat).expect("ser").into(),
    ))
    .await
    .expect("send heartbeat");

    let ack_msg = ws.next().await.expect("ack frame").expect("ok");
    let Message::Text(ack_text) = ack_msg else {
        panic!("expected text ack, got {ack_msg:?}");
    };
    let ack: Envelope = serde_json::from_str(&ack_text).expect("ack envelope");
    assert_eq!(ack.event_type, EventType::HeartbeatAck);

    ws.close(None).await.ok();
}

#[tokio::test]
async fn gateway_resume_after_reconnect() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let email = format!("resume-{}@example.com", uuid::Uuid::now_v7());
    let router = app(state(pool, redis, false, Duration::from_secs(15)));
    let cookie = register_cookie(router.clone(), &email).await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });

    let connect = |cookie: &str| {
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
        request
    };

    let (mut ws, _) = tokio_tungstenite::connect_async(connect(&cookie))
        .await
        .expect("connect");
    let hello_msg = ws.next().await.expect("hello").expect("ok");
    let Message::Text(hello_text) = hello_msg else {
        panic!("hello");
    };
    let hello: Envelope = serde_json::from_str(&hello_text).expect("hello");
    let hello_payload: HelloPayload = serde_json::from_value(hello.payload).expect("payload");

    ws.send(Message::Text(
        serde_json::to_string(&Envelope::new(0, EventType::Identify, IdentifyPayload {}))
            .expect("ser")
            .into(),
    ))
    .await
    .expect("identify");
    let ready_msg = ws.next().await.expect("ready").expect("ok");
    let Message::Text(ready_text) = ready_msg else {
        panic!("ready");
    };
    let ready: Envelope = serde_json::from_str(&ready_text).expect("ready");
    let ready_payload: ReadyPayload = serde_json::from_value(ready.payload).expect("payload");
    let sync_msg = ws.next().await.expect("sync").expect("ok");
    let Message::Text(sync_text) = sync_msg else {
        panic!("sync");
    };
    let sync: Envelope = serde_json::from_str(&sync_text).expect("sync");
    assert_eq!(sync.event_type, EventType::PresenceSync);
    ws.close(None).await.ok();

    let (mut ws2, _) = tokio_tungstenite::connect_async(connect(&cookie))
        .await
        .expect("reconnect");
    let _hello2 = ws2.next().await.expect("hello2").expect("ok");
    let resume = Envelope::new(
        0,
        EventType::Resume,
        ResumePayload {
            session_id: hello_payload.session_id,
            last_sequence: ready.sequence,
            resume_token: ready_payload.resume_token,
        },
    );
    ws2.send(Message::Text(
        serde_json::to_string(&resume).expect("ser").into(),
    ))
    .await
    .expect("resume");
    let resumed_msg = ws2.next().await.expect("resumed").expect("ok");
    let Message::Text(resumed_text) = resumed_msg else {
        panic!("resumed text");
    };
    let resumed: Envelope = serde_json::from_str(&resumed_text).expect("resumed");
    assert_eq!(resumed.event_type, EventType::Resumed);
    let _: ResumedPayload = serde_json::from_value(resumed.payload).expect("payload");
    let sync_msg = ws2.next().await.expect("sync2").expect("ok");
    let Message::Text(sync_text) = sync_msg else {
        panic!("sync2");
    };
    let sync: Envelope = serde_json::from_str(&sync_text).expect("sync2");
    assert_eq!(sync.event_type, EventType::PresenceSync);
    ws2.close(None).await.ok();
}

#[tokio::test]
async fn gateway_heartbeat_timeout_disconnects() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let email = format!("timeout-{}@example.com", uuid::Uuid::now_v7());
    let router = app(state(pool, redis, false, Duration::from_millis(40)));
    let cookie = register_cookie(router.clone(), &email).await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });

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

    let (mut ws, _) = tokio_tungstenite::connect_async(request)
        .await
        .expect("connect");

    let _hello = ws.next().await.expect("hello").expect("ok");

    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    let mut closed = false;
    while tokio::time::Instant::now() < deadline {
        let Ok(frame) = tokio::time::timeout(Duration::from_millis(200), ws.next()).await else {
            continue;
        };
        match frame {
            Some(Ok(Message::Close(_)) | Err(_)) | None => {
                closed = true;
                break;
            }
            Some(Ok(_)) => {}
        }
    }
    assert!(closed, "expected heartbeat timeout close");
}
