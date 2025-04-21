use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    status: String,
    data: T,
}

#[derive(Debug, Serialize)]
struct ApiError {
    status: String,
    message: String,
}

pub enum ApiResult<T> {
    Success(StatusCode, T),
    Error(StatusCode, String),
}

impl<T: Serialize> IntoResponse for ApiResult<T> {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiResult::Success(status, data) => (
                status,
                Json(ApiResponse {
                    status: "success".to_string(),
                    data,
                }),
            )
                .into_response(),
            ApiResult::Error(status, message) => (
                status,
                Json(ApiError {
                    status: "error".to_string(),
                    message,
                }),
            )
                .into_response(),
        }
    }
}
