use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    db,
    error::AppError,
    models::{CreateRestaurantPayload, Restaurant, RestaurantNearby, UpdateRestaurantPayload},
    search,
};

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
    pub radius: Option<f64>,
    pub limit:  Option<i64>,
}

/// GET /restaurants/search?q=&city=&award=&price=&lat=&lng=&radius=&limit=
#[derive(Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
    pub city: Option<String>,
    pub award: Option<String>,
    pub price: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub radius: Option<f64>,
    pub limit: Option<i64>,
}

/// GET /restaurants?limit=
#[derive(Deserialize)]
pub struct ListParams {
    pub limit: Option<i64>,
}

// ── Handlers ─────────────────────────────────────────────

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

async fn search_restaurants(
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<search::SearchRestaurantResult>>, AppError> {

    if let Some(lat) = params.lat {
        if !(-90.0..=90.0).contains(&lat) {
            return Err(AppError::BadRequest("lat out of range [-90, 90]".into()));
        }
    }

    if let Some(lng) = params.lng {
        if !(-180.0..=180.0).contains(&lng) {
            return Err(AppError::BadRequest("lng out of range [-180, 180]".into()));
        }
    }

    if let Some(radius) = params.radius {
        if radius <= 0.0 {
            return Err(AppError::BadRequest("radius must be > 0".into()));
        }
    }

    let q = params.q.unwrap_or_default();

    let result = search::search_with_filters(search::SearchFilters {
        q,
        city: params.city,
        award: params.award,
        price: params.price,
        lat: params.lat,
        lng: params.lng,
        radius_meters: params.radius,
        limit: params.limit,
    })
    .await
    .map_err(|e| AppError::BadRequest(format!("elasticsearch request failed: {}", e)))?;

    Ok(Json(result))
}

async fn list_restaurants(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<Restaurant>>, AppError> {
    let limit = params.limit.unwrap_or(50).min(200);

    if limit <= 0 {
        return Err(AppError::BadRequest("limit must be > 0".into()));
    }

    let rows = db::list_restaurants(&state.pool, limit).await?;
    Ok(Json(rows))
}

async fn by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Restaurant>, AppError> {
    let row = db::get_by_id(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn create_restaurant(
    State(state): State<AppState>,
    Json(payload): Json<CreateRestaurantPayload>,
) -> Result<Json<Restaurant>, AppError> {
    if payload.id.trim().is_empty() {
        return Err(AppError::BadRequest("id cannot be empty".into()));
    }
    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("name cannot be empty".into()));
    }

    let row = db::create_restaurant(&state.pool, payload).await?;

    search::index_restaurant(&row)
        .await
        .map_err(|e| AppError::BadRequest(format!("elasticsearch sync failed: {}", e)))?;

    Ok(Json(row))
}

async fn update_restaurant(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateRestaurantPayload>,
) -> Result<Json<Restaurant>, AppError> {
    if id.trim().is_empty() {
        return Err(AppError::BadRequest("id cannot be empty".into()));
    }
    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("name cannot be empty".into()));
    }

    let row = db::update_restaurant(&state.pool, &id, payload)
        .await?
        .ok_or(AppError::NotFound)?;

    search::index_restaurant(&row)
        .await
        .map_err(|e| AppError::BadRequest(format!("elasticsearch sync failed: {}", e)))?;

    Ok(Json(row))
}

async fn delete_restaurant(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    if id.trim().is_empty() {
        return Err(AppError::BadRequest("id cannot be empty".into()));
    }

    let affected = db::delete_restaurant(&state.pool, &id).await?;

    if affected == 0 {
        return Err(AppError::NotFound);
    }

    search::delete_restaurant(&id)
        .await
        .map_err(|e| AppError::BadRequest(format!("elasticsearch sync failed: {}", e)))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "deleted_id": id
    })))
}

async fn health() -> &'static str {
    "ok"
}

// ── Router ───────────────────────────────────────────────

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/restaurants", get(list_restaurants).post(create_restaurant))
        .route("/restaurants/nearby", get(nearby))
        .route("/restaurants/search", get(search_restaurants))
        .route("/restaurants/:id", get(by_id).put(update_restaurant).delete(delete_restaurant))
        .with_state(state)
}