use std::sync::Arc;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::{require_auth, Claims},
    config::Config,
    db_cql, db_pg,
    error::AppError,
    models::{CreatePostRequest, ListParams},
    mqtt::MqttPublisher,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub cassandra: Arc<scylla::Session>,
    pub config: Config,
    pub mqtt: MqttPublisher,
}

async fn health() -> &'static str { "ok" }

async fn create_post(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    if req.rating != "GOOD" && req.rating != "BAD" {
        return Err(AppError::BadRequest("rating must be GOOD or BAD".into()));
    }
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let (media_type, media_url, thumbnail_url) = db_pg::get_media(&state.pool, req.media_id, user_id)
        .await?
        .ok_or(AppError::BadRequest("media not found or not yours".into()))?;
    let restaurant_name = if let Some(ref rid) = req.restaurant_id {
        db_pg::get_restaurant_name(&state.pool, rid).await?
    } else {
        None
    };
    let post_id = Uuid::new_v4();
    let created_at = Utc::now();
    db_cql::insert_post(
        &state.cassandra,
        post_id, user_id, &claims.username,
        req.restaurant_id.as_deref(), restaurant_name.as_deref(),
        req.media_id, &media_type, &media_url, thumbnail_url.as_deref(),
        req.caption.as_deref(), &req.rating, created_at,
    ).await?;
    state.mqtt.publish("post.created", &json!({
        "post_id": post_id.to_string(),
        "user_id": user_id.to_string(),
        "username": claims.username,
        "restaurant_id": req.restaurant_id,
        "restaurant_name": restaurant_name,
        "media_type": media_type,
        "media_url": media_url,
        "thumbnail_url": thumbnail_url,
        "caption": req.caption,
        "rating": req.rating,
        "created_at": created_at.to_rfc3339(),
    })).await;
    Ok((StatusCode::CREATED, Json(json!({
        "post_id": post_id,
        "user_id": user_id,
        "username": claims.username,
        "restaurant_id": req.restaurant_id,
        "restaurant_name": restaurant_name,
        "media_type": media_type,
        "media_url": media_url,
        "rating": req.rating,
        "created_at": created_at,
    }))))
}

async fn get_post(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let post = db_cql::get_post(&state.cassandra, post_id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(serde_json::to_value(post).unwrap()))
}

async fn delete_post(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(post_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let post = db_cql::get_post(&state.cassandra, post_id).await?.ok_or(AppError::NotFound)?;
    if post.user_id != Some(user_id) { return Err(AppError::Forbidden); }
    let created_at = post.created_at.ok_or(AppError::Internal("missing created_at".into()))?;
    db_cql::delete_post(&state.cassandra, post_id, user_id, created_at, post.restaurant_id.as_deref()).await?;
    state.mqtt.publish("post.deleted", &json!({
        "post_id": post_id.to_string(),
        "user_id": user_id.to_string(),
        "restaurant_id": post.restaurant_id,
        "rating": post.rating,
    })).await;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_user_posts(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let before = params.before.as_deref()
        .and_then(|s| s.parse::<chrono::DateTime<Utc>>().ok())
        .unwrap_or_else(Utc::now);
    let limit = params.limit.unwrap_or(20).min(100);
    let posts = db_cql::list_user_posts(&state.cassandra, user_id, before, limit).await?;
    Ok(Json(json!({ "data": posts, "next_before": posts.last().and_then(|p| p.created_at).map(|t| t.to_rfc3339()) })))
}

async fn list_restaurant_posts(
    State(state): State<AppState>,
    Path(restaurant_id): Path<String>,
    Query(params): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let before = params.before.as_deref()
        .and_then(|s| s.parse::<chrono::DateTime<Utc>>().ok())
        .unwrap_or_else(Utc::now);
    let limit = params.limit.unwrap_or(20).min(100);
    let posts = db_cql::list_restaurant_posts(&state.cassandra, &restaurant_id, before, limit).await?;
    Ok(Json(json!({ "data": posts, "next_before": posts.last().and_then(|p| p.created_at).map(|t| t.to_rfc3339()) })))
}

pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/", post(create_post))
        .route("/:id", delete(delete_post))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let public = Router::new()
        .route("/health", get(health))
        .route("/:id", get(get_post))
        .route("/user/:user_id", get(list_user_posts))
        .route("/restaurant/:restaurant_id", get(list_restaurant_posts));

    Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
}
