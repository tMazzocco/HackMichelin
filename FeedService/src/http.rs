use std::sync::Arc;
use axum::{
    extract::{Extension, Query, State},
    middleware,
    routing::get,
    Json, Router,
};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use crate::{auth::{require_auth, Claims}, config::Config, db_cql, error::AppError, models::FeedParams};

#[derive(Clone)]
pub struct AppState {
    pub cassandra: Arc<scylla::Session>,
    pub config: Config,
}

async fn health() -> &'static str { "ok" }

async fn get_feed(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<FeedParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let viewer_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let before = params.before.as_deref()
        .and_then(|s| s.parse::<chrono::DateTime<Utc>>().ok())
        .unwrap_or_else(Utc::now);
    let limit = params.limit.unwrap_or(20).min(100);
    let items = db_cql::get_feed(&state.cassandra, viewer_id, before, limit).await?;
    let next_before = items.last().and_then(|i| i.created_at).map(|t| t.to_rfc3339());
    Ok(Json(json!({ "data": items, "next_before": next_before })))
}

pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/", get(get_feed))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));
    Router::new()
        .route("/health", get(health))
        .merge(protected)
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
}
