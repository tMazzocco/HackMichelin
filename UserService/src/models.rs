use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserProfile {
    pub id:              Uuid,
    pub username:        String,
    pub bio:             Option<String>,
    pub avatar_url:      Option<String>,
    pub stars_collected: i32,
    pub followers_count: i32,
    pub following_count: i32,
    pub created_at:      DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub bio:        Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StarEntry {
    pub restaurant_id:    String,
    pub restaurant_name:  String,
    pub michelin_award:   Option<String>,
    pub green_star:       bool,
    pub distinction_score: Option<i32>,
    pub collected_at:     DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct FollowEntry {
    pub user_id:  String, // UUID as string
    pub username: String,
}
