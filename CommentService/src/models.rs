use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateCommentBody {
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct Comment {
    pub comment_id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

/// Query params for GET /api/comments/post/:post_id
#[derive(Debug, Deserialize)]
pub struct ListCommentsParams {
    pub after: Option<String>,
    pub limit: Option<i32>,
}

/// Query params for DELETE /api/comments/:post_id/:comment_id
#[derive(Debug, Deserialize)]
pub struct DeleteCommentParams {
    pub created_at: Option<String>,
}
