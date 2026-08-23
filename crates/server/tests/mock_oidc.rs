//! Minimal OIDC provider for integration tests (authorization code + PKCE + RS256 ID tokens).

#![allow(clippy::missing_panics_doc, dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::{Form, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rsa::pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey};
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use uuid::Uuid;

/// Fixed key so tests never block on RSA keygen / entropy (CI hang risk).
const TEST_RSA_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----
MIIEogIBAAKCAQEAo+oW/Ncfq/WWHePYYMArpg0q9aOC9Nu0ZmGKfLo2g+L/N13c
bmInq8zNS71Bj3d8S4E8oB8HWyC7vZSh0nBYBRvssFldLFyMGMYVHic4cNsCUFSv
Z9Rlnf2WduKthESTc6BAnsBVT1FCmXFyJtT3Mx5GVnUFUknlBFbjRwgiPnPlnoMs
+1NNwsYN+UJuEVWXuR/X/Sf1ylzOjU0VnuSHUtR0NQ0AHlARQLnVSvc8lt2HgaOQ
k11UtmOk+Sr8E/LzdAhw+Ay0h5sv9GXjZ2IWSc995+Ta649sYlAkHD7wA8U5egke
KoLzTPQelkjZnonWuLnkEt8Kv4EQ9KqpBqNR7QIDAQABAoIBAEU775i9PsXx1gXr
Aq6fDPC22CHn/jzxfenOomGbf8JGQ5l9vkkrkWkZ+M7YchQommoD/Pj/EirWESZ6
3L0Xsb36tQcpv9aogo5GQI47b7YPc5M9qdcX2hIZFhBCH5tiIcvxcMn9ICt//br1
NYucYYuIappDFEvVJXQnRRlwIJKy0KBEjI/DpgSsmIbtZSurQswz9hPC9D9Lmkx7
0AxZ5oiaA/qwg5Hmyh+9QSb+kazcUPAwXf02QNYdLxDgeq3jSqohZdV/xF32Bw2t
lVDeK1AV7f2LpIRDIP2VScM6sTh3I0GMV7jmNTmbF1Bl6S1zvLkWWWSzhMkX3wYJ
bktQAwsCgYEA35a52+usgJlojGoXQlat9/iVI94XOBYvtNhjCdnU2RnaVY36Iooc
qc3lcZ5Zvmy4Na+yZC3DLF0mHxp5sKRPydQjvkt7EvCluWQ91MDxEsSFGsNfPgSM
OmX7v6xLznY6UGjhhpgLa/9UVmaCQJgVOJxKzcxcnjmJ85IAJ46cd7MCgYEAu6zg
XBiF7AYja/zqf464+BSi98hdVoQvTiITxdy6PSWHJa3juv3xwH5Aiq4l+Vqembbj
gf3T2LXQKutaTDbSfAHIjnNPAT19USE00KIXI7X1+ZcZdgGGenE0OEOJT4hjXRrx
W+bF1wOQFFu6zRj/qRXj6Jp7jFEr3Ce1yjiWP98CgYBiNECkAIp+3WKXMc3PfGTi
4lMXMuf94XjItLYjUIL1bC6Cn157JzBZwK6DTerbAcOTCP2QlK0B4lPpG2bRmAnX
ew7L+TkwY3RWzll+BdScyqYv0BoYEkVJLRv63wFYyILqaHaN+GAj6jyvykxxdJr1
h2gvphAUCu+1hK3+sdu1kwKBgE1zTKv1Gt+KsPeRypyVo9QNgCvNrmdT6cnO2mYf
b2RopltwZbj3r9sGv0/8CoPbV/SLu1wcCl82uQ/dTMiDH145xjCzeXlDjQH8ODWZ
jv8XyskUCFfgzUSejzRg+rutx4PW6KBKnn7bY4xjRrX5iRiYhhOqHS6NGRKj+KvZ
qnf1AoGAIwlD1nOfPxolqxksYMlVdmbNyhWcVQNEZVimZZp4KTJgd+lJ/3X5L0Ey
C1/lQdYnoL1yja4gTfbojxMOHtkWuViqEtpqNAaX7M8lur9CceRUBZaHypxMiyHp
wE7IYzctIYFNKiNpsKnzHfm0JEKvsbaaTFXBCY0s/6lA7PJJZNo=
-----END RSA PRIVATE KEY-----";

fn test_rsa_key() -> RsaPrivateKey {
    RsaPrivateKey::from_pkcs1_pem(TEST_RSA_PEM).expect("test rsa pem")
}

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
    #[must_use]
    pub fn start(mut config: MockOidcConfig) -> Self {
        let private_key = test_rsa_key();
        let (addr_tx, addr_rx) = std::sync::mpsc::channel();
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let cfg_for_server = config.clone();
        // Own OS thread + runtime so oneshot tests on current_thread cannot stall the IdP.
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("mock oidc runtime");
            runtime.block_on(async move {
                let listener = TcpListener::bind("127.0.0.1:0")
                    .await
                    .expect("bind mock oidc");
                let base_url = format!("http://{}", listener.local_addr().expect("addr"));
                let mut config = cfg_for_server;
                config.issuer.clone_from(&base_url);
                let state = MockOidcState {
                    config: config.clone(),
                    private_key,
                    codes: Arc::new(Mutex::new(HashMap::new())),
                };
                addr_tx
                    .send((base_url, config))
                    .expect("send mock oidc addr");
                let router = Router::new()
                    .route("/.well-known/openid-configuration", get(discovery))
                    .route("/authorize", get(authorize))
                    .route("/token", post(token))
                    .route("/jwks", get(jwks))
                    .with_state(state);
                axum::serve(listener, router)
                    .with_graceful_shutdown(async {
                        let _ = shutdown_rx.await;
                    })
                    .await
                    .expect("mock oidc serve");
            });
        });
        let (base_url, started_config) = addr_rx.recv().expect("mock oidc ready");
        config = started_config;
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
        "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post"],
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

async fn token(
    State(state): State<MockOidcState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    let grant_type = form.get("grant_type").map_or("", String::as_str);
    let code = form.get("code").cloned().unwrap_or_default();
    let code_verifier = form.get("code_verifier").cloned();

    let (client_id, client_secret) = match (
        form.get("client_id").map(String::as_str),
        form.get("client_secret").map(String::as_str),
        headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok()),
    ) {
        (Some(id), Some(secret), _) => (id.to_owned(), secret.to_owned()),
        (_, _, Some(auth)) if auth.starts_with("Basic ") => {
            let encoded = auth.trim_start_matches("Basic ").trim();
            let Ok(bytes) = STANDARD
                .decode(encoded)
                .or_else(|_| URL_SAFE_NO_PAD.decode(encoded))
            else {
                return (StatusCode::UNAUTHORIZED, "invalid_client").into_response();
            };
            let Ok(decoded) = String::from_utf8(bytes) else {
                return (StatusCode::UNAUTHORIZED, "invalid_client").into_response();
            };
            let Some((id, secret)) = decoded.split_once(':') else {
                return (StatusCode::UNAUTHORIZED, "invalid_client").into_response();
            };
            (id.to_owned(), secret.to_owned())
        }
        _ => return (StatusCode::UNAUTHORIZED, "invalid_client").into_response(),
    };

    if grant_type != "authorization_code"
        || client_id != state.config.client_id
        || client_secret != state.config.client_secret
    {
        return (StatusCode::UNAUTHORIZED, "invalid_client").into_response();
    }
    let mut codes = state.codes.lock().expect("lock");
    let Some(entry) = codes.get_mut(&code) else {
        return (StatusCode::BAD_REQUEST, "invalid_grant").into_response();
    };
    if entry.used {
        return (StatusCode::BAD_REQUEST, "invalid_grant").into_response();
    }
    if let Some(verifier) = code_verifier.as_deref() {
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        if challenge != entry.code_challenge {
            return (StatusCode::BAD_REQUEST, "invalid_grant").into_response();
        }
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
