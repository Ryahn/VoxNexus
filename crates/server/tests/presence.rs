//! Presence and custom status (F018).

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
use voxnexus_auth::{session_cookie_name, update_profile};
use voxnexus_db::{connect_and_migrate, test_database_url, PgPool};
use voxnexus_domain::PresenceStatus;
use voxnexus_jobs::{connect, RedisConn};
use voxnexus_protocol::{
    Envelope, EventType, IdentifyPayload, PresenceListResponse, GATEWAY_SUBPROTOCOL,
};
use voxnexus_realtime::{PresenceHub, ResumeStore};
use voxnexus_search::{MemorySearchEngine, SearchEngine};
use voxnexus_storage::{MemoryObjectStore, ObjectStore};

async fn test_redis() -> Option<RedisConn> {
    let url = std::env::var("REDIS_URL_TEST")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "redis://127.0.0.1:6379".to_owned());
    connect(&url).await.ok()
}

fn state(
    pool: PgPool,
    redis: RedisConn,
    gateway_heartbeat: Duration,
    presence_grace: Duration,
) -> AppState {
    AppState {
        pool,
        metrics_enabled: false,
        public_url: "http://127.0.0.1:8080".parse().expect("url"),
        cookie_secure: false,
        community_creation_mode_locked: false,
        gateway_allow_unauth: false,
        gateway_heartbeat_interval: gateway_heartbeat,
        storage: Arc::new(MemoryObjectStore::new_ready()) as Arc<dyn ObjectStore>,
        redis,
        search: Arc::new(MemorySearchEngine::new_ready()) as Arc<dyn SearchEngine>,
        web_dist: None,
        resume_store: Arc::new(ResumeStore::new()),
        presence_hub: Arc::new(PresenceHub::new(presence_grace)),
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

async fn register_cookie(router: axum::Router, email: &str) -> String {
    let response = router
        .clone()
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

async fn list_presence_http(router: &axum::Router, cookie: &str) -> PresenceListResponse {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/presence")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::OK);
    serde_json::from_slice(&json_body(response).await).expect("presence json")
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

async fn identify(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) {
    let hello = tokio::time::timeout(Duration::from_secs(5), ws.next())
        .await
        .expect("hello timeout")
        .expect("hello")
        .expect("ok");
    let _ = hello;
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

async fn close_ws(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) {
    let _ = tokio::time::timeout(Duration::from_secs(2), ws.close(None)).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn invisible_not_listed_online() {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let gateway_heartbeat = Duration::from_secs(15);
    let invisible_email = format!("invis-{}@example.com", uuid::Uuid::now_v7());
    let viewer_email = format!("viewer-{}@example.com", uuid::Uuid::now_v7());
    let shared_state = state(
        pool.clone(),
        redis,
        gateway_heartbeat,
        gateway_heartbeat * voxnexus_realtime::HEARTBEAT_TIMEOUT_FACTOR,
    );
    let router = app(shared_state.clone());
    let invisible_cookie = register_cookie(router.clone(), &invisible_email).await;
    let viewer_cookie = register_cookie(router.clone(), &viewer_email).await;

    let invisible_id = sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM accounts WHERE email = $1 AND deleted_at IS NULL",
    )
    .bind(invisible_email.trim().to_ascii_lowercase())
    .fetch_one(&pool)
    .await
    .expect("id");
    update_profile(
        &pool,
        invisible_id,
        None,
        None,
        Some(PresenceStatus::Invisible),
        None,
    )
    .await
    .expect("invisible");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });

    let mut invisible_ws = connect_gateway(addr, &invisible_cookie).await;
    identify(&mut invisible_ws).await;

    let list = list_presence_http(&app(shared_state.clone()), &viewer_cookie).await;
    assert!(!list
        .presences
        .iter()
        .any(|entry| entry.account_id == invisible_id));

    close_ws(&mut invisible_ws).await;
}
