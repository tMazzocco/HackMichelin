use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct LikeCount {
    pub post_id: Uuid,
    pub likes: i64,
}

#[derive(Debug, Serialize)]
pub struct LikeEntry {
    pub user_id: Uuid,
    pub username: String,
    pub liked_at: DateTime<Utc>,
}
