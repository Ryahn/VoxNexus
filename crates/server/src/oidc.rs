//! OIDC authorization code + PKCE login (F018O).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::sync::OnceLock;

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata};
use openidconnect::{
    AsyncHttpClient, AuthorizationCode, ClientId, ClientSecret, CsrfToken, HttpRequest,
    HttpResponse, IssuerUrl, Nonce, PkceCodeChallenge, RedirectUrl, Scope,
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::warn;
use voxnexus_auth::{
    create_session, get_instance, resolve_oidc_login, session_cookie, OidcIdentity,
    SessionCookieOptions,
};
use voxnexus_domain::Account;
use voxnexus_protocol::error_codes;

use crate::error::ApiError;
use crate::http::{request_id_from_headers, AppState};

const OIDC_STATE_PREFIX: &str = "oidc:state:";
const OIDC_STATE_TTL_SECS: u64 = 600;

#[derive(Debug, Serialize, Deserialize)]
struct OidcPendingLogin {
    code_verifier: String,
    nonce: String,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct OidcCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

/// Redirect the browser to the configured OIDC provider (authorization code + PKCE).
#[utoipa::path(
    get,
    path = "/api/v1/auth/oidc/start",
    operation_id = "startOidcLogin",
    tag = "auth",
    responses(
        (status = 200, description = "HTML bounce to OIDC provider"),
        (status = 404, description = "OIDC not configured", body = voxnexus_protocol::ErrorBody),
        (status = 500, description = "Internal error", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn oidc_start(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let request_id = request_id_from_headers(&headers);
    let instance = match get_instance(&state.pool).await {
        Ok(instance) => instance,
        Err(error) => {
            tracing::error!(error = %error, "instance lookup failed");
            return internal_error(request_id);
        }
    };
    if !oidc_ready(&state, &instance) {
        return ApiError::new(
            StatusCode::NOT_FOUND,
            error_codes::NOT_FOUND,
            "OIDC sign-in is not configured on this instance.",
            None,
            request_id,
        )
        .into_response();
    }

    let Some(secret) = state.oidc_client_secret() else {
        return internal_error(request_id);
    };
    let redirect_uri = oidc_redirect_uri(&state.public_url);
    let Some(issuer) = instance.oidc_issuer.as_deref() else {
        return internal_error(request_id);
    };
    let Some(client_id) = instance.oidc_client_id.as_deref() else {
        return internal_error(request_id);
    };
    let provider_metadata = match discover_provider(issuer).await {
        Ok(metadata) => metadata,
        Err(error) => {
            warn!(error = %error, "oidc discovery failed");
            return login_error_redirect(
                &state,
                "discovery_failed",
                "Could not reach the identity provider. Check OIDC_ISSUER from the server process.",
            );
        }
    };
    let redirect = match RedirectUrl::new(redirect_uri) {
        Ok(value) => value,
        Err(error) => {
            warn!(error = %error, "oidc redirect uri invalid");
            return internal_error(request_id);
        }
    };
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(client_id.to_owned()),
        Some(ClientSecret::new(secret.to_string())),
    )
    .set_redirect_uri(redirect);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (authorize_url, csrf_state, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let pending = OidcPendingLogin {
        code_verifier: pkce_verifier.secret().clone(),
        nonce: nonce.secret().clone(),
    };
    if let Err(error) = store_oidc_state(&state.redis, csrf_state.secret(), &pending).await {
        tracing::error!(error = %error, "oidc state store failed");
        return internal_error(request_id);
    }

    // HTML bounce instead of bare 303 — empty redirect bodies show as a white page in
    // Chrome until refresh, especially when Location uses host.docker.internal.
    html_redirect_bounce(authorize_url.as_str(), None)
}

/// OIDC provider callback: exchange code, resolve account, set session cookie, redirect home.
#[utoipa::path(
    get,
    path = "/api/v1/auth/oidc/callback",
    operation_id = "oidcCallback",
    tag = "auth",
    responses(
        (status = 200, description = "HTML bounce to SPA with session cookie"),
        (status = 302, description = "Redirect to login on provider/login error"),
        (status = 400, description = "Invalid callback", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "OIDC not configured", body = voxnexus_protocol::ErrorBody)
    )
)]
#[allow(clippy::too_many_lines)]
pub async fn oidc_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<OidcCallbackQuery>,
) -> Response {
    let request_id = request_id_from_headers(&headers);
    if let Some(error) = query.error {
        let detail = query.error_description.unwrap_or(error);
        return login_error_redirect(&state, "provider_error", &detail);
    }

    let code = query.code.filter(|value| !value.is_empty());
    let csrf_state = query.state.filter(|value| !value.is_empty());
    let (Some(code), Some(csrf_state)) = (code, csrf_state) else {
        return ApiError::new(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "Missing OIDC code or state.",
            None,
            request_id,
        )
        .into_response();
    };

    let instance = match get_instance(&state.pool).await {
        Ok(instance) => instance,
        Err(error) => {
            tracing::error!(error = %error, "instance lookup failed");
            return internal_error(request_id);
        }
    };
    if !oidc_ready(&state, &instance) {
        return ApiError::new(
            StatusCode::NOT_FOUND,
            error_codes::NOT_FOUND,
            "OIDC sign-in is not configured on this instance.",
            None,
            request_id,
        )
        .into_response();
    }

    let pending = match take_oidc_state(&state.redis, &csrf_state).await {
        Ok(Some(pending)) => pending,
        Ok(None) => {
            return login_error_redirect(
                &state,
                "invalid_state",
                "Sign-in state expired or was already used.",
            );
        }
        Err(error) => {
            tracing::error!(error = %error, "oidc state load failed");
            return internal_error(request_id);
        }
    };

    let Some(secret) = state.oidc_client_secret() else {
        return internal_error(request_id);
    };
    let redirect_uri = oidc_redirect_uri(&state.public_url);
    let Some(issuer) = instance.oidc_issuer.as_deref() else {
        return internal_error(request_id);
    };
    let Some(client_id) = instance.oidc_client_id.as_deref() else {
        return internal_error(request_id);
    };
    let provider_metadata = match discover_provider(issuer).await {
        Ok(metadata) => metadata,
        Err(error) => {
            warn!(error = %error, "oidc discovery failed");
            return login_error_redirect(
                &state,
                "discovery_failed",
                "Could not reach the identity provider. Check OIDC_ISSUER from the server process.",
            );
        }
    };
    let redirect = match RedirectUrl::new(redirect_uri) {
        Ok(value) => value,
        Err(error) => {
            warn!(error = %error, "oidc redirect uri invalid");
            return internal_error(request_id);
        }
    };
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(client_id.to_owned()),
        Some(ClientSecret::new(secret.to_string())),
    )
    .set_redirect_uri(redirect);

    let token_request = match client.exchange_code(AuthorizationCode::new(code)) {
        Ok(request) => {
            request.set_pkce_verifier(openidconnect::PkceCodeVerifier::new(pending.code_verifier))
        }
        Err(error) => {
            warn!(error = %error, "oidc code exchange setup failed");
            return login_error_redirect(
                &state,
                "token_exchange",
                "Could not complete sign-in with the provider.",
            );
        }
    };
    let token_response = match token_request.request_async(&oidc_http_call).await {
        Ok(response) => response,
        Err(error) => {
            warn!(error = %error, "oidc token exchange failed");
            return login_error_redirect(
                &state,
                "token_exchange",
                "Could not complete sign-in with the provider.",
            );
        }
    };

    let Some(id_token) = token_response.extra_fields().id_token() else {
        return login_error_redirect(
            &state,
            "missing_id_token",
            "Provider did not return an ID token.",
        );
    };

    let nonce = Nonce::new(pending.nonce);
    let claims = match id_token.claims(&client.id_token_verifier(), &nonce) {
        Ok(claims) => claims,
        Err(openidconnect::ClaimsVerificationError::InvalidIssuer(_)) => {
            return login_error_redirect(
                &state,
                "wrong_issuer",
                "Provider issuer does not match configuration.",
            );
        }
        Err(error) => {
            warn!(error = %error, "oidc id token validation failed");
            return login_error_redirect(
                &state,
                "invalid_token",
                "Provider token validation failed.",
            );
        }
    };

    let expected_issuer = instance.oidc_issuer.as_deref().unwrap_or_default();
    if claims.issuer().as_str() != expected_issuer {
        return login_error_redirect(
            &state,
            "wrong_issuer",
            "Provider issuer does not match configuration.",
        );
    }

    let identity = OidcIdentity {
        issuer: claims.issuer().as_str().to_owned(),
        subject: claims.subject().as_str().to_owned(),
        email: claims.email().map(|value| value.to_string()),
        email_verified: claims.email_verified().unwrap_or(false),
    };

    let allow_jit = instance.registration_mode.allows_registration();
    let account =
        match resolve_oidc_login(&state.pool, &identity, state.oidc_link_by_email, allow_jit).await
        {
            Ok(account) => account,
            Err(voxnexus_auth::AuthError::RegistrationClosed) => {
                return login_error_redirect(
                    &state,
                    "registration_closed",
                    "No matching account and registration is closed.",
                );
            }
            Err(voxnexus_auth::AuthError::EmailTaken) => {
                return login_error_redirect(
                    &state,
                    "email_taken",
                    "An account with this email already exists.",
                );
            }
            Err(voxnexus_auth::AuthError::IdentityTaken) => {
                return login_error_redirect(
                    &state,
                    "identity_taken",
                    "This identity is already linked to another account.",
                );
            }
            Err(error) => {
                tracing::error!(error = %error, "oidc account resolution failed");
                return internal_error(request_id);
            }
        };

    finish_oidc_login(state, headers, account).await
}

pub(crate) fn oidc_ready(state: &AppState, instance: &voxnexus_domain::Instance) -> bool {
    instance.oidc_enabled
        && instance.oidc_issuer.is_some()
        && instance.oidc_client_id.is_some()
        && state.oidc_client_secret().is_some()
}

fn oidc_redirect_uri(public_url: &voxnexus_config::Url) -> String {
    let mut url = public_url.clone();
    url.set_path("/api/v1/auth/oidc/callback");
    url.set_query(None);
    url.to_string()
}

fn oidc_http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("reqwest oidc client")
    })
}

/// Optional Docker helper: dial `OIDC_OUTBOUND_HOST` while keeping Host as the issuer host
/// (so Authentik advertises `127.0.0.1` URLs the browser can open).
fn oidc_outbound_host() -> Option<String> {
    std::env::var("OIDC_OUTBOUND_HOST")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

async fn oidc_http_call(
    request: HttpRequest,
) -> Result<HttpResponse, openidconnect::HttpClientError<reqwest::Error>> {
    let outbound = oidc_outbound_host();
    let request = rewrite_oidc_outbound(request, outbound.as_deref());
    AsyncHttpClient::call(oidc_http_client(), request).await
}

fn rewrite_oidc_outbound(mut request: HttpRequest, outbound_host: Option<&str>) -> HttpRequest {
    let Some(outbound_host) = outbound_host else {
        return request;
    };
    let Ok(mut url) = url::Url::parse(&request.uri().to_string()) else {
        return request;
    };
    let Some(host) = url.host_str().map(str::to_owned) else {
        return request;
    };
    if host != "127.0.0.1" && host != "localhost" {
        return request;
    }
    let original_authority = match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.clone(),
    };
    if url.set_host(Some(outbound_host)).is_err() {
        return request;
    }
    if let Ok(uri) = url.as_str().parse() {
        *request.uri_mut() = uri;
    }
    if let Ok(value) = HeaderValue::from_str(&original_authority) {
        request.headers_mut().insert(header::HOST, value);
    }
    request
}

async fn discover_provider(issuer: &str) -> Result<CoreProviderMetadata, String> {
    let issuer_url = IssuerUrl::new(issuer.to_string()).map_err(|error| error.to_string())?;
    CoreProviderMetadata::discover_async(issuer_url, &oidc_http_call)
        .await
        .map_err(|error| error.to_string())
}

async fn store_oidc_state(
    redis: &voxnexus_jobs::RedisConn,
    state: &str,
    pending: &OidcPendingLogin,
) -> Result<(), redis::RedisError> {
    let payload = serde_json::to_string(pending).map_err(|error| {
        redis::RedisError::from((
            redis::ErrorKind::TypeError,
            "oidc pending json",
            error.to_string(),
        ))
    })?;
    let key = format!("{OIDC_STATE_PREFIX}{state}");
    let mut conn = redis.clone();
    conn.set_ex::<_, _, ()>(key, payload, OIDC_STATE_TTL_SECS)
        .await?;
    Ok(())
}

async fn take_oidc_state(
    redis: &voxnexus_jobs::RedisConn,
    state: &str,
) -> Result<Option<OidcPendingLogin>, redis::RedisError> {
    let key = format!("{OIDC_STATE_PREFIX}{state}");
    let mut conn = redis.clone();
    let payload: Option<String> = conn.get(&key).await?;
    if payload.is_some() {
        conn.del::<_, ()>(&key).await?;
    }
    payload
        .map(|value| {
            serde_json::from_str(&value).map_err(|error| {
                redis::RedisError::from((
                    redis::ErrorKind::TypeError,
                    "oidc pending json",
                    error.to_string(),
                ))
            })
        })
        .transpose()
}

async fn finish_oidc_login(state: AppState, headers: HeaderMap, account: Account) -> Response {
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok());
    let (_session, token) = match create_session(&state.pool, account.id, user_agent, None).await {
        Ok(pair) => pair,
        Err(error) => {
            tracing::error!(error = %error, "oidc session create failed");
            return internal_error(request_id_from_headers(&headers));
        }
    };
    let options = SessionCookieOptions {
        secure: state.cookie_secure,
    };
    let cookie = session_cookie(&token, options);
    // 200 HTML bounce (not 302) so the browser commits Set-Cookie before loading the
    // SPA — a direct 303 to / can paint a blank shell until a hard refresh.
    let mut home = state.public_url.clone();
    home.set_path("/");
    home.set_query(None);
    home.set_fragment(None);
    html_redirect_bounce(home.as_str(), Some(cookie.as_str()))
}

/// Browser-facing redirect via HTML/`location.replace` (avoids blank pages on bare 303).
fn html_redirect_bounce(target: &str, cookie: Option<&str>) -> Response {
    let target_js = serde_json::to_string(target).unwrap_or_else(|_| "\"/\"".to_owned());
    let body = format!(
        "<!DOCTYPE html><html lang=\"en\"><head>\
         <meta charset=\"utf-8\">\
         <meta http-equiv=\"refresh\" content=\"0;url={escaped}\">\
         <title>Redirecting…</title>\
         </head><body>\
         <p>Redirecting…</p>\
         <script>location.replace({target_js});</script>\
         </body></html>",
        escaped = html_attr_escape(target),
    );
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    if let Some(cookie) = cookie {
        if let Ok(value) = HeaderValue::from_str(cookie) {
            response.headers_mut().insert(header::SET_COOKIE, value);
        }
    }
    response
}

fn html_attr_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

fn login_error_redirect(state: &AppState, code: &str, message: &str) -> Response {
    let mut url = state.public_url.clone();
    url.set_path("/login");
    url.set_query(Some(&format!(
        "oidc_error={}&oidc_message={}",
        urlencoding_encode(code),
        urlencoding_encode(message)
    )));
    Redirect::to(url.as_str()).into_response()
}

fn urlencoding_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn internal_error(request_id: String) -> Response {
    ApiError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        error_codes::INTERNAL,
        "Unexpected server error.",
        None,
        request_id,
    )
    .into_response()
}

impl AppState {
    pub(crate) fn oidc_client_secret(&self) -> Option<&str> {
        self.oidc_client_secret
            .as_ref()
            .map(voxnexus_config::Secret::expose)
    }
}
