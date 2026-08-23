//! Local registration, login, logout, and session cookie handlers (F011–F012).

#![allow(clippy::missing_errors_doc)]

use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use voxnexus_auth::{
    authenticate_local, change_email, change_password, clear_session_cookie, create_local_account,
    create_session, revoke_other_sessions, revoke_session, session_cookie, session_cookie_name,
    AuthError, SessionCookieOptions,
};
use voxnexus_domain::Account;
use voxnexus_protocol::{
    error_codes, AccountResponse, AuthSessionResponse, ChangeEmailRequest, ChangePasswordRequest,
    LoginRequest, RegisterRequest,
};

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::http::{request_id_from_headers, AppState};

/// Register a local account and set the session cookie.
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    operation_id = "register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Account created", body = AuthSessionResponse),
        (status = 400, description = "Validation error", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Registration closed or CSRF", body = voxnexus_protocol::ErrorBody),
        (status = 409, description = "Email taken", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    ValidatedJson(body): ValidatedJson<RegisterRequest>,
) -> Response {
    let request_id = request_id_from_headers(&headers);
    let account = match voxnexus_auth::get_instance(&state.pool).await {
        Ok(instance) if !state.oidc_only && instance.registration_mode.allows_registration() => {
            match create_local_account(
                &state.pool,
                &body.email,
                &body.password,
                &body.display_name,
                true,
            )
            .await
            {
                Ok(account) => account,
                Err(error) => return map_auth_error(&error, request_id).into_response(),
            }
        }
        Ok(_) => {
            return map_auth_error(&AuthError::RegistrationClosed, request_id).into_response();
        }
        Err(error) => {
            tracing::error!(error = %error, "instance lookup failed");
            return ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::INTERNAL,
                "Unexpected server error.",
                None,
                request_id,
            )
            .into_response();
        }
    };
    issue_session(state, headers, account, StatusCode::CREATED, request_id).await
}

/// Log in with email and password.
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    operation_id = "login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Logged in", body = AuthSessionResponse),
        (status = 400, description = "Validation error", body = voxnexus_protocol::ErrorBody),
        (status = 401, description = "Invalid credentials", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    ValidatedJson(body): ValidatedJson<LoginRequest>,
) -> Response {
    let request_id = request_id_from_headers(&headers);
    if state.oidc_only {
        return ApiError::new(
            StatusCode::FORBIDDEN,
            error_codes::PERMISSION_DENIED,
            "Password sign-in is disabled on this instance.",
            None,
            request_id,
        )
        .into_response();
    }
    let account = match authenticate_local(&state.pool, &body.email, &body.password).await {
        Ok(account) => account,
        Err(error) => return map_auth_error(&error, request_id).into_response(),
    };
    issue_session(state, headers, account, StatusCode::OK, request_id).await
}

/// Clear the session cookie and delete the server session.
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    operation_id = "logout",
    tag = "auth",
    responses(
        (status = 204, description = "Logged out")
    )
)]
pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let options = SessionCookieOptions {
        secure: state.cookie_secure,
    };
    if let Some(token) = read_session_cookie(&headers, options.secure) {
        let _ = revoke_session(&state.pool, &token).await;
    }
    (
        StatusCode::NO_CONTENT,
        [(header::SET_COOKIE, clear_session_cookie(options))],
    )
        .into_response()
}

/// Current account for the session cookie.
#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    operation_id = "getMe",
    tag = "auth",
    responses(
        (status = 200, description = "Current account", body = AuthSessionResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn me(user: crate::extract_auth::AuthUser) -> Json<AuthSessionResponse> {
    Json(AuthSessionResponse {
        account: AccountResponse {
            id: user.account_id,
            email: user.email,
            is_bot: user.is_bot,
            is_instance_admin: user.is_instance_admin,
        },
    })
}

/// Change password (re-auth with current password).
#[utoipa::path(
    post,
    path = "/api/v1/auth/me/password",
    operation_id = "changeMyPassword",
    tag = "auth",
    request_body = ChangePasswordRequest,
    responses(
        (status = 204, description = "Password updated"),
        (status = 400, description = "Validation error", body = voxnexus_protocol::ErrorBody),
        (status = 401, description = "Wrong current password or not authenticated", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn change_my_password(
    State(state): State<AppState>,
    user: crate::extract_auth::AuthUser,
    headers: HeaderMap,
    ValidatedJson(body): ValidatedJson<ChangePasswordRequest>,
) -> Result<StatusCode, ApiError> {
    let request_id = request_id_from_headers(&headers);
    match change_password(
        &state.pool,
        user.account_id,
        &body.current_password,
        &body.new_password,
    )
    .await
    {
        Ok(()) => {}
        Err(AuthError::InvalidCredentials) => {
            return Err(wrong_current_password(request_id));
        }
        Err(error) => return Err(map_auth_error(&error, request_id)),
    }
    if body.revoke_other_sessions.unwrap_or(false) {
        if let Err(error) =
            revoke_other_sessions(&state.pool, user.account_id, user.session_id).await
        {
            return Err(map_auth_error(&error, request_id));
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Change email (re-auth; applied immediately until F117 adds confirmation mail).
#[utoipa::path(
    patch,
    path = "/api/v1/auth/me/email",
    operation_id = "changeMyEmail",
    tag = "auth",
    request_body = ChangeEmailRequest,
    responses(
        (status = 200, description = "Email updated", body = AuthSessionResponse),
        (status = 400, description = "Validation error", body = voxnexus_protocol::ErrorBody),
        (status = 401, description = "Wrong current password or not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 409, description = "Email taken", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn change_my_email(
    State(state): State<AppState>,
    user: crate::extract_auth::AuthUser,
    headers: HeaderMap,
    ValidatedJson(body): ValidatedJson<ChangeEmailRequest>,
) -> Result<Json<AuthSessionResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let account = match change_email(
        &state.pool,
        user.account_id,
        &body.email,
        &body.current_password,
    )
    .await
    {
        Ok(account) => account,
        Err(error) => return Err(map_auth_error(&error, request_id)),
    };
    Ok(Json(AuthSessionResponse {
        account: account_response(&account),
    }))
}

async fn issue_session(
    state: AppState,
    headers: HeaderMap,
    account: Account,
    status: StatusCode,
    request_id: String,
) -> Response {
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok());
    let (_session, token) = match create_session(&state.pool, account.id, user_agent, None).await {
        Ok(pair) => pair,
        Err(error) => return map_auth_error(&error, request_id).into_response(),
    };
    let options = SessionCookieOptions {
        secure: state.cookie_secure,
    };
    (
        status,
        [(header::SET_COOKIE, session_cookie(&token, options))],
        Json(AuthSessionResponse {
            account: account_response(&account),
        }),
    )
        .into_response()
}

fn account_response(account: &Account) -> AccountResponse {
    AccountResponse {
        id: account.id,
        email: account.email.clone(),
        is_bot: account.is_bot,
        is_instance_admin: account.is_instance_admin,
    }
}

fn read_session_cookie(headers: &HeaderMap, secure: bool) -> Option<String> {
    let name = session_cookie_name(secure);
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in cookie_header.split(';') {
        let part = part.trim();
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        if key == name && !value.is_empty() {
            return Some(value.to_owned());
        }
    }
    None
}

fn wrong_current_password(request_id: String) -> ApiError {
    ApiError::new(
        StatusCode::UNAUTHORIZED,
        error_codes::UNAUTHENTICATED,
        "Current password is incorrect.",
        None,
        request_id,
    )
}

fn map_auth_error(error: &AuthError, request_id: String) -> ApiError {
    match error {
        AuthError::RegistrationClosed => ApiError::new(
            StatusCode::FORBIDDEN,
            error_codes::PERMISSION_DENIED,
            "Registration is closed on this instance.",
            None,
            request_id,
        ),
        AuthError::EmailTaken => {
            ApiError::conflict(request_id, "An account with this email already exists.")
        }
        AuthError::SlugTaken => ApiError::conflict(request_id, "Community slug is already taken."),
        AuthError::IdentityTaken => {
            ApiError::conflict(request_id, "This identity is already linked to an account.")
        }
        AuthError::InvalidCredentials => ApiError::unauthenticated(request_id),
        AuthError::AlreadyMember
        | AuthError::NotMember
        | AuthError::AlreadySpaceMember
        | AuthError::NotSpaceMember
        | AuthError::SpaceJoinNotAllowed
        | AuthError::JoinNotAllowed
        | AuthError::OwnerCannotLeave
        | AuthError::NotCommunityOwner
        | AuthError::CategoryScopeMismatch
        | AuthError::ChannelScopeMismatch
        | AuthError::InviteNotFound
        | AuthError::InviteExpired
        | AuthError::InviteExhausted
        | AuthError::InvitePaused => {
            tracing::error!(error = %error, "unexpected community auth error in auth routes");
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::INTERNAL,
                "Unexpected server error.",
                None,
                request_id,
            )
        }
        AuthError::Password(_) | AuthError::Db(_) => {
            tracing::error!(error = %error, "auth failure");
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::INTERNAL,
                "Unexpected server error.",
                None,
                request_id,
            )
        }
    }
}
