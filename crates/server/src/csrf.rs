//! CSRF hook. Enforcement lands in F012 when session cookies exist.

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

/// Pass-through middleware so mutating routes have a single place to attach Origin checks.
pub async fn csrf_hook(request: Request, next: Next) -> Response {
    next.run(request).await
}
