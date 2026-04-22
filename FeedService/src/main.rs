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
    // Load .env (ignored if absent)
    let _ = dotenvy::dotenv();

    // Tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer())
        .init();

    let cfg = config::Config::from_env()?;
    info!("Starting FeedService on {}", cfg.http_addr);

    // Connect to Cassandra / ScyllaDB
    let session = scylla::SessionBuilder::new()
        .known_nodes(&cfg.cassandra_nodes)
        .build()
        .await?;
    let cassandra = Arc::new(session);
    info!("Cassandra session connected.");

    // Spawn MQTT subscriber in background
    {
        let cfg_mqtt = cfg.clone();
        let cassandra_mqtt = Arc::clone(&cassandra);
        tokio::spawn(async move {
            mqtt::run(cfg_mqtt, cassandra_mqtt).await;
        });
    }

    // HTTP server
    let app = http::router(http::AppState {
        cassandra: Arc::clone(&cassandra),
        config: cfg.clone(),
    });

    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    info!("HTTP listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
