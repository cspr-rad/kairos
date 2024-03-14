use thiserror::Error;
use axum::{http::StatusCode, response::IntoResponse, response::Response, Json};
use serde_json::json;

use serde::de::value::Error as DeserializeError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to parse JSON")]
    JsonParseError(#[from] DeserializeError),
    #[error("Internal server error")]
    InternalServerError(),
}

// Return response based on AppError
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::InternalServerError() => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JsonParseError(_) => StatusCode::BAD_REQUEST,
        };

        let error_message = self.to_string();

        // Send JSON response based on error
        (status, Json(json!({"error": error_message}))).into_response()
    }
}