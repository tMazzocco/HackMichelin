mod config;
mod db;
mod error;
mod http;
mod models;
mod mqtt;

use sqlx::postgres::PgPoolOptions;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env (ignored if absent — useful for docker where env vars are injected)
    let _ = dotenvy::dotenv();

    // Tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer())
        .init();

    let cfg = config::Config::from_env()?;
    info!("Starting MapsDataService on {}", cfg.http_addr);

    // DB pool
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await?;
    info!("PostgreSQL pool connected.");

    // Spawn MQTT subscriber in background
    {
        let cfg_mqtt  = cfg.clone();
        let pool_mqtt = pool.clone();
        tokio::spawn(async move {
            mqtt::run(cfg_mqtt, pool_mqtt).await;
        });
    }

    // HTTP server
    let app   = http::router(http::AppState { pool });
    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;

    info!("HTTP listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
