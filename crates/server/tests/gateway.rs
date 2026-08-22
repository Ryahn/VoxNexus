use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use futures_util::{SinkExt, StreamExt};
use http_body_util::BodyExt;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::Message;
use tower::ServiceExt;
use voxnexus::http::{app, AppState};
use voxnexus_db::{test_database_url, PgPool};
use voxnexus_jobs::{connect, RedisConn};
use voxnexus_protocol::{
    error_codes, Envelope, ErrorBody, EventType, HeartbeatPayload, GATEWAY_SUBPROTOCOL,
};
use voxnexus_search::{MemorySearchEngine, SearchEngine};
use voxnexus_storage::{MemoryObjectStore, ObjectStore};

async fn test_redis() -> Option<RedisConn> {
    let url = std::env::var("REDIS_URL_TEST")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "redis://127.0.0.1:6379".to_owned());
    connect(&url).await.ok()
}

fn state(pool: PgPool, redis: RedisConn, allow_unauth: bool, heartbeat: Duration) -> AppState {
    AppState {
        pool,
        metrics_enabled: false,
        public_url: "http://127.0.0.1:8080".parse().expect("url"),
        cookie_secure: false,
        registration_open: true,
        gateway_allow_unauth: allow_unauth,
        gateway_heartbeat_interval: heartbeat,
        storage: Arc::new(MemoryObjectStore::new_ready()) as Arc<dyn ObjectStore>,
        redis,
        search: Arc::new(MemorySearchEngine::new_ready()) as Arc<dyn SearchEngine>,
        web_dist: None,
    }
}

#[tokio::test]
async fn gateway_refused_without_unauth_flag() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = voxnexus_db::connect(&url).await.expect("connect");
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
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let body: ErrorBody = serde_json::from_slice(&bytes).expect("error body");
    assert_eq!(body.code, error_codes::GATEWAY_UNAVAILABLE);
}

#[tokio::test]
async fn gateway_hello_and_heartbeat_ack() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = voxnexus_db::connect(&url).await.expect("connect");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    let router = app(state(pool, redis, true, Duration::from_secs(15)));
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

    let (mut ws, _) = tokio_tungstenite::connect_async(request)
        .await
        .expect("connect");

    let hello_msg = ws.next().await.expect("hello frame").expect("ok");
    let Message::Text(text) = hello_msg else {
        panic!("expected text hello, got {hello_msg:?}");
    };
    let hello: Envelope = serde_json::from_str(&text).expect("hello envelope");
    assert_eq!(hello.event_type, EventType::Hello);
    assert_eq!(hello.sequence, 1);

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
async fn gateway_heartbeat_timeout_disconnects() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = voxnexus_db::connect(&url).await.expect("connect");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    let router = app(state(pool, redis, true, Duration::from_millis(40)));
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
