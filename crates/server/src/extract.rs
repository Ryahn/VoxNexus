use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use axum::Json;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::ApiError;
use crate::http::request_id_from_headers;

/// JSON body that must deserialize. Syntax errors become [`ApiError`] `invalid_json`.
pub struct AppJson<T>(pub T);

/// JSON body that must deserialize and pass [`Validate`].
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for AppJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let request_id = request_id_from_headers(request.headers());
        let Json(value) = Json::<T>::from_request(request, state)
            .await
            .map_err(|rejection| json_rejection(&rejection, request_id))?;
        Ok(Self(value))
    }
}

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let request_id = request_id_from_headers(request.headers());
        let Json(value) = Json::<T>::from_request(request, state)
            .await
            .map_err(|rejection| json_rejection(&rejection, request_id.clone()))?;
        value
            .validate()
            .map_err(|errors| ApiError::validation(request_id, &errors))?;
        Ok(Self(value))
    }
}

fn json_rejection(rejection: &JsonRejection, request_id: String) -> ApiError {
    ApiError::invalid_json(request_id, rejection.body_text())
}
