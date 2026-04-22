use axum::{
    body::Body,
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub:      String,
    pub username: String,
    pub exp:      usize,
}

/// Decode and verify a Bearer JWT, returning its Claims on success.
pub fn verify_token(secret: &str, token: &str) -> Result<Claims, AppError> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let data = decode::<Claims>(token, &key, &Validation::default())
        .map_err(|_| AppError::Unauthorized)?;
    Ok(data.claims)
}

/// Axum middleware: extracts `Authorization: Bearer <token>`, verifies it,
/// and inserts the resulting `Claims` into request extensions.
pub async fn require_auth(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Read the JWT_SECRET we stored in extensions by the router setup.
    let secret = req
        .extensions()
        .get::<JwtSecret>()
        .map(|s| s.0.clone())
        .unwrap_or_default();

    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims = verify_token(&secret, token)?;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

/// Newtype wrapper so we can store the secret in Axum extensions.
#[derive(Clone)]
pub struct JwtSecret(pub String);
