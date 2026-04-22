use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{db, error::AppError, models::RestaurantNearby};

// ── Shared app state ─────────────────────────────────────
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

// ── Query params ─────────────────────────────────────────

/// GET /restaurants/nearby?lat=&lng=&radius=&limit=
#[derive(Deserialize)]
pub struct NearbyParams {
    pub lat:    f64,
    pub lng:    f64,
    /// Search radius in metres (default 1000, max 50 000)
    pub radius: Option<f64>,
    /// Max results (default 20, max 100)
    pub limit:  Option<i64>,
}

// ── Handlers ─────────────────────────────────────────────

/// Return restaurants within `radius` metres of (lat, lng).
async fn nearby(
    State(state): State<AppState>,
    Query(params): Query<NearbyParams>,
) -> Result<Json<Vec<RestaurantNearby>>, AppError> {
    let radius = params.radius.unwrap_or(1000.0).min(50_000.0);
    let limit  = params.limit.unwrap_or(20).min(100);

    if !(-90.0..=90.0).contains(&params.lat) {
        return Err(AppError::BadRequest("lat out of range [-90, 90]".into()));
    }
    if !(-180.0..=180.0).contains(&params.lng) {
        return Err(AppError::BadRequest("lng out of range [-180, 180]".into()));
    }
    if radius <= 0.0 {
        return Err(AppError::BadRequest("radius must be > 0".into()));
    }

    let rows = db::get_nearby(&state.pool, params.lat, params.lng, radius, limit).await?;
    Ok(Json(rows))
}

/// Return a single restaurant by Michelin objectID.
async fn by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::Restaurant>, AppError> {
    let row = db::get_by_id(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

/// Health-check endpoint for load balancer / Nginx upstream probe.
async fn health() -> &'static str {
    "ok"
}

// ── Router ───────────────────────────────────────────────

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health",                  get(health))
        .route("/restaurants/nearby",      get(nearby))
        .route("/restaurants/:id",         get(by_id))
        .with_state(state)
}
