use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("pg error: {0}")]
    Pg(#[from] sqlx::Error),
    #[error("cassandra error: {0}")]
    Cassandra(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::Pg(e) => { tracing::error!("PG: {e}"); (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string()) }
            AppError::Cassandra(e) => { tracing::error!("Cassandra: {e}"); (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string()) }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::Internal(m) => { tracing::error!("Internal: {m}"); (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string()) }
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}
