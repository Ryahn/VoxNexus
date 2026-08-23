//! Minimal OIDC provider for integration tests (authorization code + PKCE + RS256 ID tokens).

#![allow(clippy::missing_panics_doc, dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::{Form, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Clone)]
pub struct MockOidcConfig {
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub subject: String,
    pub email: String,
    pub wrong_issuer: bool,
}

#[derive(Clone)]
struct MockOidcState {
    config: MockOidcConfig,
    private_key: RsaPrivateKey,
    codes: Arc<Mutex<HashMap<String, PendingCode>>>,
}

#[derive(Clone)]
struct PendingCode {
    code_challenge: String,
    nonce: String,
    used: bool,
}

pub struct MockOidcServer {
    pub base_url: String,
    pub config: MockOidcConfig,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
}

impl MockOidcServer {
    pub async fn start(mut config: MockOidcConfig) -> Self {
        let private_key = RsaPrivateKey::new(&mut rand::thread_rng(), 2048).expect("rsa key");
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock oidc");
        let base_url = format!("http://{}", listener.local_addr().expect("addr"));
        config.issuer.clone_from(&base_url);
        let state = MockOidcState {
            config: config.clone(),
            private_key,
            codes: Arc::new(Mutex::new(HashMap::new())),
        };
        let router = Router::new()
            .route("/.well-known/openid-configuration", get(discovery))
            .route("/authorize", get(authorize))
            .route("/token", post(token))
            .route("/jwks", get(jwks))
            .with_state(state);
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .expect("mock oidc serve");
        });
        Self {
            base_url,
            config,
            shutdown: Some(shutdown_tx),
        }
    }
}

impl Drop for MockOidcServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

#[derive(Debug, Deserialize)]
struct AuthorizeQuery {
    client_id: String,
    redirect_uri: String,
    response_type: String,
    state: String,
    code_challenge: String,
    code_challenge_method: String,
    nonce: String,
}

#[derive(Debug, Deserialize)]
struct TokenForm {
    grant_type: String,
    code: String,
    #[allow(dead_code)]
    redirect_uri: String,
    client_id: String,
    client_secret: String,
    code_verifier: String,
}

async fn discovery(State(state): State<MockOidcState>) -> Json<serde_json::Value> {
    let issuer = &state.config.issuer;
    Json(json!({
        "issuer": issuer,
        "authorization_endpoint": format!("{issuer}/authorize"),
        "token_endpoint": format!("{issuer}/token"),
        "jwks_uri": format!("{issuer}/jwks"),
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "scopes_supported": ["openid", "email"],
        "token_endpoint_auth_methods_supported": ["client_secret_post"],
    }))
}

async fn authorize(
    State(state): State<MockOidcState>,
    Query(query): Query<AuthorizeQuery>,
) -> Response {
    if query.client_id != state.config.client_id || query.response_type != "code" {
        return StatusCode::BAD_REQUEST.into_response();
    }
    if query.code_challenge_method != "S256" {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let code = Uuid::now_v7().to_string();
    state.codes.lock().expect("lock").insert(
        code.clone(),
        PendingCode {
            code_challenge: query.code_challenge,
            nonce: query.nonce,
            used: false,
        },
    );
    let location = format!("{}?code={}&state={}", query.redirect_uri, code, query.state);
    Redirect::to(&location).into_response()
}

async fn token(State(state): State<MockOidcState>, Form(form): Form<TokenForm>) -> Response {
    if form.grant_type != "authorization_code"
        || form.client_id != state.config.client_id
        || form.client_secret != state.config.client_secret
    {
        return (StatusCode::UNAUTHORIZED, "invalid_client").into_response();
    }
    let mut codes = state.codes.lock().expect("lock");
    let Some(entry) = codes.get_mut(&form.code) else {
        return (StatusCode::BAD_REQUEST, "invalid_grant").into_response();
    };
    if entry.used {
        return (StatusCode::BAD_REQUEST, "invalid_grant").into_response();
    }
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(form.code_verifier.as_bytes()));
    if challenge != entry.code_challenge {
        return (StatusCode::BAD_REQUEST, "invalid_grant").into_response();
    }
    entry.used = true;
    let nonce = entry.nonce.clone();
    drop(codes);

    let issuer = if state.config.wrong_issuer {
        "http://wrong-issuer.example".to_owned()
    } else {
        state.config.issuer.clone()
    };
    let now = chrono::Utc::now().timestamp();
    let claims = IdTokenClaims {
        iss: issuer,
        sub: state.config.subject.clone(),
        aud: state.config.client_id.clone(),
        exp: now + 3600,
        iat: now,
        nonce,
        email: state.config.email.clone(),
        email_verified: true,
    };
    let pem = state
        .private_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .expect("pem");
    let key = EncodingKey::from_rsa_pem(pem.as_bytes()).expect("encoding key");
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test".to_owned());
    let id_token = encode(&header, &claims, &key).expect("id token");

    (
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "access_token": "mock-access-token",
            "token_type": "Bearer",
            "expires_in": 3600,
            "id_token": id_token,
        })),
    )
        .into_response()
}

async fn jwks(State(state): State<MockOidcState>) -> Json<serde_json::Value> {
    let public = state.private_key.to_public_key();
    let n = URL_SAFE_NO_PAD.encode(public.n().to_bytes_be());
    let e = URL_SAFE_NO_PAD.encode(public.e().to_bytes_be());
    Json(json!({
        "keys": [{
            "kty": "RSA",
            "kid": "test",
            "use": "sig",
            "alg": "RS256",
            "n": n,
            "e": e,
        }]
    }))
}

#[derive(Debug, Serialize)]
struct IdTokenClaims {
    iss: String,
    sub: String,
    aud: String,
    exp: i64,
    iat: i64,
    nonce: String,
    email: String,
    email_verified: bool,
}
