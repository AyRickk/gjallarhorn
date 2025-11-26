use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    ValidationError(String),
    AuthenticationError(String),
    InternalError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, details) = match self {
            AppError::DatabaseError(err) => {
                tracing::error!("Database error: {:?}", err);
                // Only expose detailed errors in debug mode (development)
                let details = if cfg!(debug_assertions) {
                    Some(err.to_string())
                } else {
                    None // Hide database details in production
                };
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error occurred".to_string(),
                    details,
                )
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg, None),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg, None),
            AppError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg, None),
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                // Only expose detailed errors in debug mode (development)
                let details = if cfg!(debug_assertions) {
                    Some(msg)
                } else {
                    None // Hide internal error details in production
                };
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                    details,
                )
            }
        };

        let body = ErrorResponse {
            error: error_message,
            details,
        };

        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for AppError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        AppError::InternalError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
