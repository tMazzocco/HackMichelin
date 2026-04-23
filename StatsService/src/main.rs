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
    let _ = dotenvy::dotenv();
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer()).init();
    let cfg = config::Config::from_env()?;
    info!("Starting StatsService on {}", cfg.http_addr);
    let pool = PgPoolOptions::new().max_connections(10).connect(&cfg.database_url).await?;
    {
        let cfg2 = cfg.clone();
        let pool2 = pool.clone();
        tokio::spawn(async move { mqtt::run(cfg2, pool2).await; });
    }
    let app = http::router(http::AppState { pool });
    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("HTTP listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
