//! Authenticated account bound to the request session cookie.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;
use voxnexus_auth::{resolve_session, session_cookie_name};
use voxnexus_domain::Account;

use crate::error::ApiError;
use crate::http::{request_id_from_headers, AppState};

/// Account + session resolved from the `vn_session` / `__Host-vn_session` cookie.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub account_id: Uuid,
    pub session_id: Uuid,
    pub email: Option<String>,
    pub is_bot: bool,
    pub is_instance_admin: bool,
}

impl AuthUser {
    #[must_use]
    pub fn from_account(account: &Account, session_id: Uuid) -> Self {
        Self {
            account_id: account.id,
            session_id,
            email: account.email.clone(),
            is_bot: account.is_bot,
            is_instance_admin: account.is_instance_admin,
        }
    }
}

/// Authenticated account that is also the instance admin.
#[derive(Debug, Clone)]
pub struct InstanceAdmin(pub AuthUser);

impl FromRequestParts<AppState> for InstanceAdmin {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        if !user.is_instance_admin {
            let request_id = request_id_from_headers(&parts.headers);
            return Err(ApiError::permission_denied(
                request_id,
                "Instance administrator access required.",
            ));
        }
        Ok(InstanceAdmin(user))
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let request_id = request_id_from_headers(&parts.headers);
        if let Some(user) = parts.extensions.get::<AuthUser>() {
            return Ok(user.clone());
        }
        let user = resolve_auth_user(state, &parts.headers)
            .await
            .ok_or_else(|| ApiError::unauthenticated(request_id))?;
        parts.extensions.insert(user.clone());
        Ok(user)
    }
}

/// Paths under `/api/v1` that do not require a session cookie.
#[must_use]
pub fn is_public_api_path(path: &str) -> bool {
    matches!(
        path,
        "/api/v1/meta"
            | "/api/v1/auth/register"
            | "/api/v1/auth/login"
            | "/api/v1/auth/logout"
            | "/api/v1/gateway"
    )
}

/// Resolve [`AuthUser`] from cookie headers, if present and valid.
pub async fn resolve_auth_user(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Option<AuthUser> {
    let name = session_cookie_name(state.cookie_secure);
    let cookie_header = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    let mut token = None;
    for part in cookie_header.split(';') {
        let part = part.trim();
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        if key == name && !value.is_empty() {
            token = Some(value);
            break;
        }
    }
    let token = token?;
    let (session, account) = resolve_session(&state.pool, token).await.ok()??;
    Some(AuthUser::from_account(&account, session.id))
}
