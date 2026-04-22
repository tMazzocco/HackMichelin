use jsonwebtoken::{encode, Header, EncodingKey};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub:      String, // user_id UUID as string
    pub username: String,
    pub exp:      usize,
}

/// Issue a signed HS256 JWT.
pub fn generate_token(
    secret:      &str,
    expires_secs: u64,
    user_id:     &str,
    username:    &str,
) -> Result<String, AppError> {
    let exp = (chrono::Utc::now()
        + chrono::Duration::seconds(expires_secs as i64))
        .timestamp() as usize;

    let claims = Claims {
        sub:      user_id.to_string(),
        username: username.to_string(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))
}

/// Generate a random 32-byte hex string suitable as a refresh token.
pub fn generate_refresh_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// SHA-256 hash of a token (used for DB storage).
pub fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
