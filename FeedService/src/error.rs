use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum AppError {
    #[error("cassandra: {0}")] Cassandra(String),
    #[error("unauthorized")]   Unauthorized,
    #[error("bad request: {0}")] BadRequest(String),
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::Cassandra(e) => { tracing::error!("Cass: {e}"); (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string()) }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}
