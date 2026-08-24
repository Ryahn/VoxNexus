mod mock_oidc;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use mock_oidc::{MockOidcConfig, MockOidcServer};
use tokio::sync::Mutex;
use tower::ServiceExt;
use url::Url;
use voxnexus::http::{app, AppState};
use voxnexus_config::Secret;
use voxnexus_db::{connect_and_migrate, test_database_url, PgPool};
use voxnexus_domain::DEFAULT_INSTANCE_ID;
use voxnexus_jobs::{connect, RedisConn};
use voxnexus_search::{MemorySearchEngine, SearchEngine};
use voxnexus_storage::{MemoryObjectStore, ObjectStore};

/// Instance OIDC columns are shared across tests on one Postgres DB.
static OIDC_TEST_LOCK: Mutex<()> = Mutex::const_new(());

async fn test_redis() -> Option<RedisConn> {
    let url = std::env::var("REDIS_URL_TEST")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "redis://127.0.0.1:6379".to_owned());
    connect(&url).await.ok()
}

fn oidc_state(pool: PgPool, redis: RedisConn, public_url: &str, secret: &str) -> AppState {
    AppState {
        pool,
        metrics_enabled: false,
        public_url: public_url.parse().expect("url"),
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
        oidc_client_secret: Some(Secret::new(secret.to_owned())),
        oidc_only: false,
        oidc_link_by_email: true,
        permission_cache: Arc::new(voxnexus_permissions::PermissionCache::default()),
    }
}

async fn enable_oidc(pool: &PgPool, issuer: &str, client_id: &str) {
    sqlx::query(
        r"
        UPDATE instances
        SET oidc_enabled = TRUE, oidc_issuer = $1, oidc_client_id = $2, updated_at = now()
        WHERE id = $3
        ",
    )
    .bind(issuer)
    .bind(client_id)
    .bind(DEFAULT_INSTANCE_ID)
    .execute(pool)
    .await
    .expect("enable oidc");
}

fn location(response: &axum::response::Response) -> String {
    response
        .headers()
        .get(header::LOCATION)
        .expect("location")
        .to_str()
        .expect("location str")
        .to_owned()
}

fn html_bounce_target(body: &str) -> String {
    let marker = "location.replace(";
    let start = body.find(marker).expect("location.replace in bounce html") + marker.len();
    let rest = &body[start..];
    let parsed: String =
        serde_json::from_str(rest.split(')').next().expect("js call")).expect("url json");
    parsed
}

/// Hit the mock IdP authorize endpoint without following its redirect to PUBLIC_URL
/// (oneshot router is not listening on a real TCP port).
async fn fetch_authorize_redirect(authorize_url: &str) -> String {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("reqwest");
    let authorize = client.get(authorize_url).send().await.expect("authorize");
    assert_eq!(authorize.status(), StatusCode::SEE_OTHER);
    authorize
        .headers()
        .get(reqwest::header::LOCATION)
        .expect("callback location")
        .to_str()
        .expect("callback str")
        .to_owned()
}

fn callback_request_path(callback_url: &str) -> String {
    let parsed = Url::parse(callback_url).expect("callback url");
    match parsed.query() {
        Some(query) => format!("{}?{}", parsed.path(), query),
        None => parsed.path().to_owned(),
    }
}

async fn follow_oidc_login(
    router: axum::Router,
    _mock: &MockOidcServer,
) -> axum::response::Response {
    let start = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/oidc/start")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("start");
    assert_eq!(start.status(), StatusCode::OK);
    assert_eq!(
        start
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("text/html; charset=utf-8")
    );
    let body = axum::body::to_bytes(start.into_body(), usize::MAX)
        .await
        .expect("start body");
    let authorize_url = html_bounce_target(std::str::from_utf8(&body).expect("utf8"));
    let callback_url = fetch_authorize_redirect(&authorize_url).await;
    let callback_path = callback_request_path(&callback_url);
    router
        .oneshot(
            Request::builder()
                .uri(callback_path)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("callback")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oidc_jit_login_creates_account_once() {
    let _guard = OIDC_TEST_LOCK.lock().await;
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let mock = MockOidcServer::start(MockOidcConfig {
        issuer: String::new(),
        client_id: "voxnexus-test".to_owned(),
        client_secret: "test-secret".to_owned(),
        subject: "oidc-subject-1".to_owned(),
        email: "oidc-user@example.com".to_owned(),
        wrong_issuer: false,
    })
    .await;
    let public_url = "http://127.0.0.1:8080";
    enable_oidc(&pool, &mock.config.issuer, &mock.config.client_id).await;
    let router = app(oidc_state(
        pool.clone(),
        redis,
        public_url,
        &mock.config.client_secret,
    ));

    let first = follow_oidc_login(router.clone(), &mock).await;
    assert_eq!(first.status(), StatusCode::OK);
    assert!(first.headers().get(header::SET_COOKIE).is_some());
    assert_eq!(
        first
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("text/html; charset=utf-8")
    );

    let count_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE email = $1")
        .bind("oidc-user@example.com")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(count_before, 1);

    let second = follow_oidc_login(router, &mock).await;
    assert_eq!(second.status(), StatusCode::OK);
    let count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE email = $1")
        .bind("oidc-user@example.com")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(count_after, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn replayed_oidc_code_fails() {
    let _guard = OIDC_TEST_LOCK.lock().await;
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let mock = MockOidcServer::start(MockOidcConfig {
        issuer: String::new(),
        client_id: "voxnexus-test".to_owned(),
        client_secret: "test-secret".to_owned(),
        subject: "oidc-subject-replay".to_owned(),
        email: "oidc-replay@example.com".to_owned(),
        wrong_issuer: false,
    })
    .await;
    let public_url = "http://127.0.0.1:8080";
    enable_oidc(&pool, &mock.config.issuer, &mock.config.client_id).await;
    let router = app(oidc_state(
        pool,
        redis,
        public_url,
        &mock.config.client_secret,
    ));

    let start = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/oidc/start")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("start");
    let start_body = axum::body::to_bytes(start.into_body(), usize::MAX)
        .await
        .expect("start body");
    let authorize_url = html_bounce_target(std::str::from_utf8(&start_body).expect("utf8"));
    let callback_url = fetch_authorize_redirect(&authorize_url).await;

    let first = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(callback_request_path(&callback_url))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("callback");
    assert_eq!(first.status(), StatusCode::OK);

    let replay = router
        .oneshot(
            Request::builder()
                .uri(callback_request_path(&callback_url))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("replay");
    assert_eq!(replay.status(), StatusCode::SEE_OTHER);
    let replay_location = location(&replay);
    assert!(replay_location.contains("oidc_error=invalid_state"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oidc_wrong_issuer_is_rejected() {
    let _guard = OIDC_TEST_LOCK.lock().await;
    let Some(url) = test_database_url() else {
        eprintln!("skipping: DATABASE_URL_TEST required");
        return;
    };
    let Some(redis) = test_redis().await else {
        eprintln!("skipping: Redis not reachable");
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate");
    let mock = MockOidcServer::start(MockOidcConfig {
        issuer: String::new(),
        client_id: "voxnexus-test".to_owned(),
        client_secret: "test-secret".to_owned(),
        subject: "oidc-subject-wrong-issuer".to_owned(),
        email: "oidc-wrong@example.com".to_owned(),
        wrong_issuer: true,
    })
    .await;
    let public_url = "http://127.0.0.1:8080";
    enable_oidc(&pool, &mock.config.issuer, &mock.config.client_id).await;
    let router = app(oidc_state(
        pool,
        redis,
        public_url,
        &mock.config.client_secret,
    ));

    let response = follow_oidc_login(router, &mock).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let location = location(&response);
    assert!(location.contains("oidc_error=wrong_issuer"));
}
