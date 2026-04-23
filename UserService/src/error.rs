use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("cassandra error: {0}")]
    Cassandra(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::Database(e) => {
                tracing::error!("DB: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
            }
            AppError::Cassandra(e) => {
                tracing::error!("Cassandra: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
            }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::Internal(m) => {
                tracing::error!("Internal: {m}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
            }
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}
