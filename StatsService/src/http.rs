use axum::{extract::{Path, State}, routing::get, Json, Router};
use sqlx::PgPool;
use crate::{db, error::AppError};

#[derive(Clone)]
pub struct AppState { pub pool: PgPool }

async fn health() -> &'static str { "ok" }

async fn get_stats(
    State(state): State<AppState>,
    Path(restaurant_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let stats = db::get_stats(&state.pool, &restaurant_id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(serde_json::to_value(stats).unwrap()))
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/restaurants/:id", get(get_stats))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
}
