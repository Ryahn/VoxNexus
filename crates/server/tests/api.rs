use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::routing::post;
use axum::Router;
use http_body_util::BodyExt;
use serde::Deserialize;
use tower::ServiceExt;
use validator::Validate;
use voxnexus::extract::ValidatedJson;
use voxnexus::http::{not_found, with_middleware};
use voxnexus_protocol::error_codes;
use voxnexus_protocol::ErrorBody;

#[derive(Debug, Deserialize, Validate)]
struct NameBody {
    #[validate(length(min = 1, max = 8))]
    name: String,
}

async fn accept_name(ValidatedJson(_body): ValidatedJson<NameBody>) -> StatusCode {
    StatusCode::NO_CONTENT
}

fn api_test_router() -> Router {
    let public_url = "http://127.0.0.1:8080".parse().expect("public url");
    with_middleware(
        Router::new()
            .route("/api/v1/validate", post(accept_name))
            .fallback(not_found),
        &public_url,
        false,
    )
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

#[tokio::test]
async fn unknown_route_404_matches_error_schema() {
    let response = api_test_router()
        .oneshot(
            Request::builder()
                .uri("/api/v1/does-not-exist")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body: ErrorBody = serde_json::from_slice(&json_body(response).await).expect("error");
    assert_eq!(body.code, error_codes::NOT_FOUND);
    assert!(!body.message.is_empty());
    assert!(body.details.is_none());
    let parsed = uuid::Uuid::parse_str(&body.request_id).expect("request id uuid");
    assert_eq!(parsed.get_version_num(), 7);
}

#[tokio::test]
async fn validation_error_matches_error_schema() {
    let response = api_test_router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/validate")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .body(Body::from(r#"{"name":""}"#))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body: ErrorBody = serde_json::from_slice(&json_body(response).await).expect("error");
    assert_eq!(body.code, error_codes::VALIDATION_ERROR);
    assert!(!body.message.is_empty());
    let details = body.details.expect("details");
    assert!(details
        .get("fields")
        .and_then(|fields| fields.get("name"))
        .is_some());
    uuid::Uuid::parse_str(&body.request_id).expect("request id uuid");
}

#[tokio::test]
async fn invalid_json_matches_error_schema() {
    let response = api_test_router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/validate")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ORIGIN, "http://127.0.0.1:8080")
                .body(Body::from("{not-json"))
                .expect("request"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body: ErrorBody = serde_json::from_slice(&json_body(response).await).expect("error");
    assert_eq!(body.code, error_codes::INVALID_JSON);
    uuid::Uuid::parse_str(&body.request_id).expect("request id uuid");
}
