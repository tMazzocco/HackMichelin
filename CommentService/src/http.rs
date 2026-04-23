use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde_json::json;
use uuid::Uuid;

use crate::{
    auth::{require_auth, Claims},
    config::Config,
    db_cql,
    error::AppError,
    models::{CreateCommentBody, DeleteCommentParams, ListCommentsParams},
};

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    pub cassandra: Arc<scylla::Session>,
    pub config: Config,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/comments/post/:post_id
/// Auth required. Creates a new comment and returns 201 with Comment JSON.
async fn create_comment(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(post_id): Path<Uuid>,
    Json(body): Json<CreateCommentBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    if body.body.trim().is_empty() {
        return Err(AppError::BadRequest("body must not be empty".into()));
    }

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let comment_id = Uuid::new_v4();
    let now: DateTime<Utc> = Utc::now();

    db_cql::insert_comment(
        &state.cassandra,
        post_id,
        comment_id,
        user_id,
        &claims.username,
        &body.body,
        now,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "comment_id": comment_id,
            "post_id": post_id,
            "user_id": user_id,
            "username": claims.username,
            "body": body.body,
            "created_at": now,
        })),
    ))
}

/// GET /api/comments/post/:post_id[?after=&limit=]
/// No auth required. Returns comments ascending by created_at.
async fn list_comments(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
    Query(params): Query<ListCommentsParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let after: Option<DateTime<Utc>> = if let Some(ref after_str) = params.after {
        Some(
            DateTime::parse_from_rfc3339(after_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|_| AppError::BadRequest("invalid 'after' timestamp; expected ISO 8601".into()))?,
        )
    } else {
        None
    };

    let limit = params.limit.unwrap_or(20).min(100).max(1);

    let comments = db_cql::get_comments(&state.cassandra, post_id, after, limit).await?;

    Ok(Json(json!({ "data": comments })))
}

/// DELETE /api/comments/:post_id/:comment_id?created_at=<ISO8601>
/// Auth required. The caller must be the comment's author.
async fn delete_comment(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((post_id, comment_id)): Path<(Uuid, Uuid)>,
    Query(params): Query<DeleteCommentParams>,
) -> Result<StatusCode, AppError> {
    let created_at_str = params
        .created_at
        .ok_or_else(|| AppError::BadRequest("query param 'created_at' is required".into()))?;

    let created_at: DateTime<Utc> = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| AppError::BadRequest("invalid 'created_at' timestamp; expected ISO 8601".into()))?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    // Verify the comment exists and belongs to the authenticated user
    let comment = db_cql::get_comment(&state.cassandra, post_id, comment_id, created_at)
        .await?
        .ok_or(AppError::NotFound)?;

    if comment.user_id != user_id {
        return Err(AppError::Forbidden);
    }

    db_cql::delete_comment(&state.cassandra, post_id, comment_id, created_at).await?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

async fn health() -> &'static str {
    "ok"
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn build_router(state: AppState) -> Router {
    // Protected routes (require Bearer JWT)
    let protected = Router::new()
        .route(
            "/api/comments/post/:post_id",
            post(create_comment),
        )
        .route(
            "/api/comments/:post_id/:comment_id",
            delete(delete_comment),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    // Public routes
    let public = Router::new()
        .route("/health", get(health))
        .route("/api/comments/post/:post_id", get(list_comments));

    Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
}
