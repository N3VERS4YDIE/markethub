use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Authentication(_) => StatusCode::UNAUTHORIZED,
            Self::Authorization(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code(&self) -> &str {
        match self {
            Self::Database(_) => "DATABASE_ERROR",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Authentication(_) => "AUTHENTICATION_ERROR",
            Self::Authorization(_) => "AUTHORIZATION_ERROR",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();
        let message = self.to_string();

        // Log internal errors
        if matches!(self, Self::Internal(_) | Self::Database(_)) {
            tracing::error!("Internal error: {}", message);
        }

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

// Helper function for validation errors
pub fn validation_error(message: impl Into<String>) -> AppError {
    AppError::Validation(message.into())
}
