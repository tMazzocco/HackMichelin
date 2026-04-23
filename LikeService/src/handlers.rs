use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{auth::Claims, db, error::AppError, AppState};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i32>,
}

/// POST /api/likes/post/:post_id
/// Like a post — requires authentication.
pub async fn like_post(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(post_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: Uuid = claims
        .sub
        .parse()
        .map_err(|_| AppError::Cassandra("Invalid user_id in token".to_string()))?;

    db::like_post(&state.cassandra, post_id, user_id, &claims.username).await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "liked": true }))))
}

/// DELETE /api/likes/post/:post_id
/// Unlike a post — requires authentication.
pub async fn unlike_post(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(post_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: Uuid = claims
        .sub
        .parse()
        .map_err(|_| AppError::Cassandra("Invalid user_id in token".to_string()))?;

    db::unlike_post(&state.cassandra, post_id, user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/likes/post/:post_id/count
/// Return the total like count for a post — no auth required.
pub async fn get_like_count(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let count = db::get_like_count(&state.cassandra, post_id).await?;
    Ok((StatusCode::OK, Json(count)))
}

/// GET /api/likes/post/:post_id?limit=N
/// List users who liked the post — no auth required.
pub async fn list_likes(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
    Query(params): Query<ListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = params.limit.unwrap_or(50).max(1).min(200);
    let entries = db::list_likes(&state.cassandra, post_id, limit).await?;
    Ok((StatusCode::OK, Json(entries)))
}
