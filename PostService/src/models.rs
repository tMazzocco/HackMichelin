use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub media_id: uuid::Uuid,
    pub restaurant_id: Option<String>,
    pub caption: Option<String>,
    pub rating: String, // "GOOD" or "BAD"
}

#[derive(Debug, Serialize)]
pub struct Post {
    pub post_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub username: Option<String>,
    pub restaurant_id: Option<String>,
    pub restaurant_name: Option<String>,
    pub media_id: Option<uuid::Uuid>,
    pub media_type: Option<String>,
    pub media_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub caption: Option<String>,
    pub rating: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub before: Option<String>, // ISO 8601 timestamp cursor
    pub limit: Option<i32>,
}
