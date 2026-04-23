use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub media_id: Uuid,
    pub restaurant_id: Option<String>,
    pub caption: Option<String>,
    pub rating: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Post {
    pub post_id: Uuid,
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub restaurant_id: Option<String>,
    pub restaurant_name: Option<String>,
    pub media_id: Option<Uuid>,
    pub media_type: Option<String>,
    pub media_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub caption: Option<String>,
    pub rating: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub before: Option<String>,
    pub limit: Option<i32>,
}
