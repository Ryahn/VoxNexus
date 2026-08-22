use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};
use validator::ValidationErrors;
use voxnexus_protocol::error_codes;
use voxnexus_protocol::ErrorBody;

/// API error that serializes to [`ErrorBody`]. Never includes stack traces.
#[derive(Debug, Clone)]
pub struct ApiError {
    status: StatusCode,
    body: ErrorBody,
}

impl ApiError {
    #[must_use]
    pub fn new(
        status: StatusCode,
        code: &'static str,
        message: impl Into<String>,
        details: Option<Value>,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            status,
            body: ErrorBody {
                code: code.to_owned(),
                message: message.into(),
                details,
                request_id: request_id.into(),
            },
        }
    }

    #[must_use]
    pub fn not_found(request_id: impl Into<String>) -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            error_codes::NOT_FOUND,
            "The requested resource was not found.",
            None,
            request_id,
        )
    }

    #[must_use]
    pub fn invalid_json(request_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            error_codes::INVALID_JSON,
            message,
            None,
            request_id,
        )
    }

    #[must_use]
    pub fn validation(request_id: impl Into<String>, errors: &ValidationErrors) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "Request validation failed.",
            Some(validation_details(errors)),
            request_id,
        )
    }

    #[must_use]
    pub fn gateway_unavailable(request_id: impl Into<String>) -> Self {
        Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            error_codes::GATEWAY_UNAVAILABLE,
            "Gateway requires authentication (F013). Set GATEWAY_ALLOW_UNAUTH=true for local protocol development only.",
            None,
            request_id,
        )
    }

    #[must_use]
    pub fn unauthenticated(request_id: impl Into<String>) -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            error_codes::UNAUTHENTICATED,
            "Authentication required.",
            None,
            request_id,
        )
    }

    #[must_use]
    pub fn conflict(request_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::CONFLICT,
            error_codes::CONFLICT,
            message,
            None,
            request_id,
        )
    }

    #[must_use]
    pub fn body(&self) -> &ErrorBody {
        &self.body
    }

    #[must_use]
    pub fn status(&self) -> StatusCode {
        self.status
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

fn validation_details(errors: &ValidationErrors) -> Value {
    let mut fields = serde_json::Map::new();
    for (field, field_errors) in errors.field_errors() {
        let messages: Vec<Value> = field_errors
            .iter()
            .map(|error| {
                Value::String(
                    error
                        .message
                        .as_ref()
                        .map_or_else(|| error.code.to_string(), ToString::to_string),
                )
            })
            .collect();
        fields.insert((*field).to_owned(), Value::Array(messages));
    }
    json!({ "fields": fields })
}
