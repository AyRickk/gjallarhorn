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
        let (status, error_message, details, _error_type) = match &self {
            AppError::DatabaseError(err) => {
                // Structured error logging with detailed context
                tracing::error!(
                    error_type = "database_error",
                    error_details = ?err,
                    status_code = %StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    "Database error occurred"
                );
                // Record validation error metric
                crate::metrics::VALIDATION_ERRORS
                    .with_label_values(&["database"])
                    .inc();

                let details = if cfg!(debug_assertions) {
                    Some(err.to_string())
                } else {
                    None
                };
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error occurred".to_string(),
                    details,
                    "database_error",
                )
            }
            AppError::NotFound(msg) => {
                tracing::warn!(
                    error_type = "not_found",
                    message = %msg,
                    status_code = %StatusCode::NOT_FOUND.as_u16(),
                    "Resource not found"
                );
                (StatusCode::NOT_FOUND, msg.clone(), None, "not_found")
            }
            AppError::ValidationError(msg) => {
                tracing::warn!(
                    error_type = "validation_error",
                    message = %msg,
                    status_code = %StatusCode::BAD_REQUEST.as_u16(),
                    "Validation failed"
                );
                // Record validation error metric
                crate::metrics::VALIDATION_ERRORS
                    .with_label_values(&["validation"])
                    .inc();

                (StatusCode::BAD_REQUEST, msg.clone(), None, "validation_error")
            }
            AppError::AuthenticationError(msg) => {
                tracing::warn!(
                    error_type = "authentication_error",
                    message = %msg,
                    status_code = %StatusCode::UNAUTHORIZED.as_u16(),
                    "Authentication failed"
                );
                (StatusCode::UNAUTHORIZED, msg.clone(), None, "authentication_error")
            }
            AppError::InternalError(msg) => {
                tracing::error!(
                    error_type = "internal_error",
                    message = %msg,
                    status_code = %StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    "Internal server error"
                );
                // Record internal error metric
                crate::metrics::VALIDATION_ERRORS
                    .with_label_values(&["internal"])
                    .inc();

                let details = if cfg!(debug_assertions) {
                    Some(msg.clone())
                } else {
                    None
                };
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                    details,
                    "internal_error",
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
