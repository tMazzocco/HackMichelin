use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct FeedItem {
    pub post_id: Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub author_id: Option<Uuid>,
    pub author_name: Option<String>,
    pub restaurant_id: Option<String>,
    pub restaurant_name: Option<String>,
    pub media_type: Option<String>,
    pub media_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub caption: Option<String>,
    pub rating: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FeedParams {
    pub before: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PostCreatedEvent {
    pub post_id: String,
    pub user_id: String,
    pub username: String,
    pub restaurant_id: Option<String>,
    pub restaurant_name: Option<String>,
    pub media_type: Option<String>,
    pub media_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub caption: Option<String>,
    pub rating: Option<String>,
    pub created_at: Option<String>,
}
