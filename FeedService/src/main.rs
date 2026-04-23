mod auth;
mod config;
mod db_cql;
mod error;
mod http;
mod models;
mod mqtt;

use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer())
        .init();

    let cfg = config::Config::from_env()?;
    info!("Starting FeedService on {}", cfg.http_addr);

    let cassandra = Arc::new(
        scylla::SessionBuilder::new()
            .known_node(&cfg.cassandra_nodes)
            .build()
            .await?,
    );
    info!("Cassandra connected.");

    // MQTT subscriber runs in background
    {
        let cfg2 = cfg.clone();
        let cass2 = cassandra.clone();
        tokio::spawn(async move { mqtt::run(cfg2, cass2).await; });
    }

    let app = http::router(http::AppState { cassandra, config: cfg.clone() });
    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("HTTP listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
