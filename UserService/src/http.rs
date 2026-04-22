use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::{require_auth, Claims, JwtSecret},
    config::Config,
    db_cql, db_pg,
    error::AppError,
    models::{FollowEntry, UpdateProfileRequest},
    mqtt::MqttPublisher,
};

// ── Application state ─────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub pool:      PgPool,
    pub cassandra: Arc<scylla::Session>,
    pub config:    Config,
    pub mqtt:      MqttPublisher,
}

// ── Pagination query params ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page:  Option<i64>,
    pub limit: Option<i64>,
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router(state: AppState) -> Router {
    let jwt_secret = JwtSecret(state.config.jwt_secret.clone());

    // Public routes (no auth required)
    let public = Router::new()
        .route("/health", get(health))
        .route("/users/:id", get(get_user_profile))
        .route("/users/:id/followers", get(get_followers))
        .route("/users/:id/following", get(get_following))
        .route("/users/:id/stars", get(get_user_stars));

    // Protected routes (JWT required)
    let protected = Router::new()
        .route("/me", patch(patch_me))
        .route("/users/:id/follow", post(follow_user))
        .route("/users/:id/follow", delete(unfollow_user))
        .route("/me/stars/:restaurant_id", post(collect_star))
        .route("/me/stars/:restaurant_id", delete(uncollect_star))
        .layer(Extension(jwt_secret.clone()))
        .layer(middleware::from_fn(require_auth));

    Router::new()
        .merge(public)
        .merge(protected)
        .layer(Extension(jwt_secret))
        .with_state(state)
        .layer(
            tower_http::cors::CorsLayer::permissive(),
        )
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn health() -> &'static str {
    "ok"
}

async fn get_user_profile(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let profile = db_pg::get_profile(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(profile))
}

async fn patch_me(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: Uuid = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;

    let profile = db_pg::update_profile(
        &state.pool,
        user_id,
        body.bio.clone(),
        body.avatar_url.clone(),
    )
    .await?;

    // Publish user.updated event
    let payload = json!({
        "user_id":         profile.id,
        "username":        profile.username,
        "bio":             profile.bio,
        "avatar_url":      profile.avatar_url,
        "stars_collected": profile.stars_collected,
        "followers_count": profile.followers_count,
    });
    state
        .mqtt
        .publish("user.updated", payload.to_string())
        .await;

    Ok(Json(profile))
}

async fn follow_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(target_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let follower_id: Uuid = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;

    // Prevent self-follow
    if follower_id == target_id {
        return Err(AppError::BadRequest("cannot follow yourself".into()));
    }

    // Ensure target user exists
    db_pg::get_profile(&state.pool, target_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Idempotency check
    if db_cql::is_following(&state.cassandra, follower_id, target_id).await? {
        return Err(AppError::Conflict("already following".into()));
    }

    // We need the usernames for the Cassandra rows
    let follower_profile = db_pg::get_profile(&state.pool, follower_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let target_profile = db_pg::get_profile(&state.pool, target_id)
        .await?
        .ok_or(AppError::NotFound)?;

    db_cql::follow_user(
        &state.cassandra,
        follower_id,
        target_id,
        &follower_profile.username,
        &target_profile.username,
    )
    .await?;

    db_pg::increment_followers(&state.pool, target_id).await?;
    db_pg::increment_following(&state.pool, follower_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn unfollow_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(target_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let follower_id: Uuid = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;

    db_cql::unfollow_user(&state.cassandra, follower_id, target_id).await?;
    db_pg::decrement_followers(&state.pool, target_id).await?;
    db_pg::decrement_following(&state.pool, follower_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_followers(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let pairs = db_cql::list_followers(&state.cassandra, user_id).await?;
    let entries: Vec<FollowEntry> = pairs
        .into_iter()
        .map(|(id, username)| FollowEntry {
            user_id: id.to_string(),
            username,
        })
        .collect();
    Ok(Json(entries))
}

async fn get_following(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let pairs = db_cql::list_following(&state.cassandra, user_id).await?;
    let entries: Vec<FollowEntry> = pairs
        .into_iter()
        .map(|(id, username)| FollowEntry {
            user_id: id.to_string(),
            username,
        })
        .collect();
    Ok(Json(entries))
}

async fn collect_star(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(restaurant_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: Uuid = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;

    db_pg::collect_star(&state.pool, user_id, &restaurant_id).await?;
    db_pg::increment_stars(&state.pool, user_id).await?;

    Ok(StatusCode::CREATED)
}

async fn uncollect_star(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(restaurant_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: Uuid = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;

    db_pg::uncollect_star(&state.pool, user_id, &restaurant_id).await?;
    db_pg::decrement_stars(&state.pool, user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_user_stars(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let limit = params.limit.unwrap_or(20).min(100).max(1);
    let page  = params.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let stars = db_pg::list_stars(&state.pool, user_id, limit, offset).await?;
    Ok(Json(stars))
}
