//! CSRF Origin / Referer checks for mutating requests (F012).

use axum::extract::Request;
use axum::extract::State;
use axum::http::{header, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use url::Url;
use voxnexus_protocol::error_codes;
use voxnexus_protocol::ErrorBody;

use crate::http::request_id_from_headers;

/// Public URL plus cookie mode for CSRF decisions.
#[derive(Clone)]
pub struct CsrfState {
    pub public_url: Url,
    pub cookie_secure: bool,
}

/// Reject cross-site mutating requests using `Origin` (preferred) or `Referer`.
pub async fn csrf_hook(State(csrf): State<CsrfState>, request: Request, next: Next) -> Response {
    if is_safe_method(request.method()) {
        return next.run(request).await;
    }

    let request_id = request_id_from_headers(request.headers());
    let expected = csrf.public_url.origin().ascii_serialization();
    let origin_ok = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|origin| origin_allowed(origin, &expected, csrf.cookie_secure));
    let referer_ok = request
        .headers()
        .get(header::REFERER)
        .and_then(|value| value.to_str().ok())
        .and_then(|referer| Url::parse(referer).ok())
        .is_some_and(|referer| {
            origin_allowed(
                &referer.origin().ascii_serialization(),
                &expected,
                csrf.cookie_secure,
            )
        });

    if origin_ok || referer_ok {
        return next.run(request).await;
    }

    (
        StatusCode::FORBIDDEN,
        Json(ErrorBody {
            code: error_codes::PERMISSION_DENIED.to_owned(),
            message: "CSRF check failed: Origin or Referer must match PUBLIC_URL.".to_owned(),
            details: None,
            request_id,
        }),
    )
        .into_response()
}

fn is_safe_method(method: &Method) -> bool {
    matches!(
        *method,
        Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE
    )
}

fn origin_allowed(origin: &str, expected: &str, cookie_secure: bool) -> bool {
    if origin == expected {
        return true;
    }
    // Vite (`:5173`) proxies `/api` while PUBLIC_URL stays on `:8080`.
    if cookie_secure {
        return false;
    }
    Url::parse(origin)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .is_some_and(|host| host == "localhost" || host == "127.0.0.1")
}
