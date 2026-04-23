use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct RestaurantStats {
    pub restaurant_id: String,
    pub total_posts: i32,
    pub good_posts: i32,
    pub bad_posts: i32,
    pub good_pct: f64,
}

#[derive(Debug, Deserialize)]
pub struct PostEvent {
    pub restaurant_id: Option<String>,
    pub rating: Option<String>,
}
