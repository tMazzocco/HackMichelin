use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum AppError {
    #[error("db: {0}")] Database(#[from] sqlx::Error),
    #[error("not found")] NotFound,
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::Database(e) => { tracing::error!("DB: {e}"); (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string()) }
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}
