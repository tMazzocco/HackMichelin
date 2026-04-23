use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub media_id: Uuid,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub media_type: String, // "photo" or "video"
}
