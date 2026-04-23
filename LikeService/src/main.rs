mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod models;

use std::sync::Arc;

use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use scylla::SessionBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Clone)]
pub struct AppState {
    pub cassandra: Arc<scylla::Session>,
    pub config: config::Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let _ = dotenvy::dotenv();

    let cfg = config::Config::from_env();

    info!("Connecting to Cassandra at {}", cfg.cassandra_nodes);
    let session = SessionBuilder::new()
        .known_node(&cfg.cassandra_nodes)
        .build()
        .await?;

    session.use_keyspace("hackmichelin", false).await?;

    let session = Arc::new(session);

    let state = AppState {
        cassandra: session,
        config: cfg.clone(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Routes that require authentication
    let authed = Router::new()
        .route(
            "/api/likes/post/:post_id",
            post(handlers::like_post).delete(handlers::unlike_post),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));

    // Public routes
    let public = Router::new()
        .route(
            "/api/likes/post/:post_id/count",
            get(handlers::get_like_count),
        )
        .route("/api/likes/post/:post_id", get(handlers::list_likes));

    let app = Router::new()
        .merge(authed)
        .merge(public)
        .layer(cors)
        .with_state(state);

    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    info!("LikeService listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
