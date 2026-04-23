use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id:            Uuid,
    pub username:      String,
    pub email:         String,
    pub password_hash: String,
}

#[derive(Debug, Serialize)]
pub struct UserPublic {
    pub id:       Uuid,
    pub username: String,
    pub email:    String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email:    String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email:    String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token:         String,
    pub refresh_token: String,
    pub user:          UserPublic,
}
