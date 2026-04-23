use axum::{extract::{Request, State}, middleware::Next, response::Response};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use crate::{error::AppError, http::AppState};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims { pub sub: String, pub username: String, pub exp: usize }
pub fn verify_token(secret: &str, token: &str) -> Result<Claims, AppError> {
    let mut val = Validation::new(Algorithm::HS256);
    val.validate_exp = true;
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &val)
        .map(|d| d.claims).map_err(|_| AppError::Unauthorized)
}
pub async fn require_auth(State(state): State<AppState>, mut req: Request, next: Next) -> Result<Response, AppError> {
    let token = req.headers().get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok()).and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;
    let claims = verify_token(&state.config.jwt_secret, token)?;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
