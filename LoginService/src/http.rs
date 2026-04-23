use axum::{
    extract::State,
    http::{header, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::{
    auth::{generate_refresh_token, generate_token, hash_token},
    config::Config,
    db,
    error::AppError,
    models::{AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, UserPublic},
    mqtt::MqttPublisher,
};

// ── Shared state ──────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub pool:   PgPool,
    pub config: Config,
    pub mqtt:   MqttPublisher,
}

// ── Handlers ─────────────────────────────────────────────

async fn health() -> &'static str {
    "ok"
}

/// POST /register
async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validation
    let username = body.username.trim().to_string();
    if username.len() < 3 || username.len() > 50 {
        return Err(AppError::BadRequest(
            "username must be between 3 and 50 characters".into(),
        ));
    }
    if body.password.len() < 8 {
        return Err(AppError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }

    // Hash password
    let password_hash =
        bcrypt::hash(&body.password, 12).map_err(|e| AppError::Internal(e.to_string()))?;

    // Persist user
    let user = db::create_user(&state.pool, &username, &body.email, &password_hash).await?;

    // Issue tokens
    let access_token = generate_token(
        &state.config.jwt_secret,
        state.config.jwt_expires_secs,
        &user.id.to_string(),
        &user.username,
    )?;
    let refresh_token = generate_refresh_token();
    let refresh_hash  = hash_token(&refresh_token);

    db::store_refresh_token(
        &state.pool,
        &refresh_hash,
        user.id,
        state.config.refresh_expires_days,
    )
    .await?;

    // Publish MQTT event
    let event = json!({
        "user_id":    user.id,
        "username":   user.username,
        "bio":        null,
        "avatar_url": null,
    });
    state.mqtt.publish("user.registered", &event).await;
    info!("Published user.registered for user_id={}", user.id);

    let response = AuthResponse {
        token: access_token,
        refresh_token,
        user: UserPublic {
            id:       user.id,
            username: user.username,
            email:    user.email,
        },
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// POST /login
async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = db::find_by_email(&state.pool, &body.email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let valid = bcrypt::verify(&body.password, &user.password_hash)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !valid {
        return Err(AppError::Unauthorized);
    }

    let access_token = generate_token(
        &state.config.jwt_secret,
        state.config.jwt_expires_secs,
        &user.id.to_string(),
        &user.username,
    )?;
    let refresh_token = generate_refresh_token();
    let refresh_hash  = hash_token(&refresh_token);

    db::store_refresh_token(
        &state.pool,
        &refresh_hash,
        user.id,
        state.config.refresh_expires_days,
    )
    .await?;

    Ok(Json(AuthResponse {
        token: access_token,
        refresh_token,
        user: UserPublic {
            id:       user.id,
            username: user.username,
            email:    user.email,
        },
    }))
}

/// POST /refresh
async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<impl IntoResponse, AppError> {
    let token_hash = hash_token(&body.refresh_token);

    let (user_id, username) = db::get_refresh_token(&state.pool, &token_hash)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let access_token = generate_token(
        &state.config.jwt_secret,
        state.config.jwt_expires_secs,
        &user_id.to_string(),
        &username,
    )?;

    Ok(Json(json!({ "token": access_token })))
}

/// POST /logout
async fn logout(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<impl IntoResponse, AppError> {
    let token_hash = hash_token(&body.refresh_token);
    db::delete_refresh_token(&state.pool, &token_hash).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Router ────────────────────────────────────────────────

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    Router::new()
        .route("/health",  get(health))
        .route("/register", post(register))
        .route("/login",    post(login))
        .route("/refresh",  post(refresh))
        .route("/logout",   post(logout))
        .layer(cors)
        .with_state(state)
}
