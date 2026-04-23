use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::{error::AppError, http::AppState};

/// JWT claims issued by LoginService.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: usize,
}

/// Decode and validate a HS256 JWT, returning the embedded claims.
pub fn verify_token(secret: &str, token: &str) -> Result<Claims, AppError> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    decode::<Claims>(token, &key, &validation)
        .map(|data| data.claims)
        .map_err(|e| {
            tracing::warn!("JWT validation failed: {e}");
            AppError::Unauthorized
        })
}

/// Axum middleware that enforces a valid Bearer JWT.
///
/// On success the `Claims` are inserted into request extensions so downstream
/// handlers can retrieve them with `req.extensions().get::<Claims>()`.
pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims = verify_token(&state.config.jwt_secret, token)?;
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
