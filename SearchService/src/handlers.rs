use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use elasticsearch::Elasticsearch;
use serde::Deserialize;

use crate::{error::AppError, es};

// ── Query-param structs ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RestaurantSearchParams {
    pub q:       Option<String>,
    pub city:    Option<String>,
    pub country: Option<String>,
    pub cuisine: Option<String>, // reserved for future index field; not used in ES query yet
    pub award:   Option<String>,
    pub lat:     Option<f64>,
    pub lng:     Option<f64>,
    /// Radius in metres; converted to "Xkm" for ES.
    pub radius:  Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UserSearchParams {
    pub q: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/search/restaurants
pub async fn search_restaurants(
    State(es): State<Arc<Elasticsearch>>,
    Query(params): Query<RestaurantSearchParams>,
) -> Result<impl IntoResponse, AppError> {
    let radius_str = params.radius.map(|m| format!("{}km", m / 1000.0));

    let results = es::search_restaurants(
        &es,
        params.q,
        params.city,
        params.country,
        params.award,
        params.lat,
        params.lng,
        radius_str,
    )
    .await?;

    Ok(Json(results))
}

/// GET /api/search/users
pub async fn search_users(
    State(es): State<Arc<Elasticsearch>>,
    Query(params): Query<UserSearchParams>,
) -> Result<impl IntoResponse, AppError> {
    // If no query string is provided, return empty array immediately.
    let q = match params.q.filter(|s| !s.trim().is_empty()) {
        Some(q) => q,
        None => return Ok(Json(vec![])),
    };

    let results = es::search_users(&es, q).await?;
    Ok(Json(results))
}
