use std::sync::Arc;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::{require_auth, Claims},
    config::Config,
    db_cql, db_pg,
    error::AppError,
    models::{FollowEntry, PageParams, UpdateProfileRequest},
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

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let profile = db_pg::get_profile(&state.pool, id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(serde_json::to_value(profile).unwrap()))
}

async fn update_me(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let profile = db_pg::update_profile(&state.pool, user_id, req.bio.clone(), req.avatar_url.clone()).await?;
    state.mqtt.publish("user.updated", &json!({
        "user_id": user_id.to_string(),
        "username": claims.username,
        "bio": req.bio,
        "avatar_url": req.avatar_url,
        "stars_collected": profile.stars_collected,
        "followers_count": profile.followers_count,
    })).await;
    Ok(Json(serde_json::to_value(profile).unwrap()))
}

async fn follow(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(target_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let me = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    if me == target_id { return Err(AppError::BadRequest("cannot follow yourself".into())); }
    let target = db_pg::get_profile(&state.pool, target_id).await?.ok_or(AppError::NotFound)?;
    let me_profile = db_pg::get_profile(&state.pool, me).await?.ok_or(AppError::NotFound)?;
    if db_cql::is_following(&state.cassandra, me, target_id).await? {
        return Err(AppError::Conflict("already following".into()));
    }
    db_cql::follow_user(&state.cassandra, me, target_id, &claims.username, &target.username).await?;
    db_pg::adjust_counter(&state.pool, target_id, "followers_count", 1).await?;
    db_pg::adjust_counter(&state.pool, me, "following_count", 1).await?;
    let _ = me_profile; // suppress unused warning
    Ok(StatusCode::NO_CONTENT)
}

async fn unfollow(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(target_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let me = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    db_cql::unfollow_user(&state.cassandra, me, target_id).await?;
    db_pg::adjust_counter(&state.pool, target_id, "followers_count", -1).await?;
    db_pg::adjust_counter(&state.pool, me, "following_count", -1).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_followers(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<FollowEntry>>, AppError> {
    let list = db_cql::list_followers(&state.cassandra, user_id).await?;
    Ok(Json(list.into_iter().map(|(id, name)| FollowEntry { user_id: id.to_string(), username: name }).collect()))
}

async fn get_following(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<FollowEntry>>, AppError> {
    let list = db_cql::list_following(&state.cassandra, user_id).await?;
    Ok(Json(list.into_iter().map(|(id, name)| FollowEntry { user_id: id.to_string(), username: name }).collect()))
}

async fn collect_star(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(restaurant_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    db_pg::collect_star(&state.pool, user_id, &restaurant_id).await?;
    db_pg::adjust_counter(&state.pool, user_id, "stars_collected", 1).await?;
    Ok(StatusCode::CREATED)
}

async fn uncollect_star(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(restaurant_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    db_pg::uncollect_star(&state.pool, user_id, &restaurant_id).await?;
    db_pg::adjust_counter(&state.pool, user_id, "stars_collected", -1).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_stars(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<PageParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.page.unwrap_or(0) * limit;
    let stars = db_pg::list_stars(&state.pool, user_id, limit, offset).await?;
    Ok(Json(json!({ "data": stars, "page": params.page.unwrap_or(0), "limit": limit })))
}

pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/me", patch(update_me))
        .route("/users/:id/follow", post(follow))
        .route("/users/:id/follow", delete(unfollow))
        .route("/me/stars/:restaurant_id", post(collect_star))
        .route("/me/stars/:restaurant_id", delete(uncollect_star))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let public = Router::new()
        .route("/health", get(health))
        .route("/users/:id", get(get_user))
        .route("/users/:id/followers", get(get_followers))
        .route("/users/:id/following", get(get_following))
        .route("/users/:id/stars", get(list_stars));

    Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
}
