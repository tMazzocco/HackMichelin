mod auth;
mod config;
mod db;
mod error;
mod http;
mod models;

use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env (silently ignored when absent — Docker injects env vars directly)
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer())
        .init();

    let cfg = config::Config::from_env()?;
    info!("Starting UploadService on {}", cfg.http_addr);
    info!("Media directory: {}", cfg.media_dir);
    info!("Max upload size: {} bytes", cfg.max_upload_bytes);

    // Ensure the media directory exists
    tokio::fs::create_dir_all(&cfg.media_dir).await?;

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await?;

    info!("Database connection pool established");

    let state = http::AppState {
        pool,
        config: cfg.clone(),
    };

    let app = http::router(state);
    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;

    info!("HTTP listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
