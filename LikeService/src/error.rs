use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Cassandra error: {0}")]
    Cassandra(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Conflict: already liked")]
    Conflict,

    #[error("Not found")]
    NotFound,
}

impl From<scylla::transport::errors::QueryError> for AppError {
    fn from(e: scylla::transport::errors::QueryError) -> Self {
        AppError::Cassandra(e.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Cassandra(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Conflict => (StatusCode::CONFLICT, "Already liked".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
