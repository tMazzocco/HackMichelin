mod auth;
mod config;
mod db_cql;
mod db_pg;
mod error;
mod http;
mod models;
mod mqtt;

use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
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
    info!("Starting UserService on {}", cfg.http_addr);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await?;
    info!("PostgreSQL connected.");

    let cassandra = Arc::new(
        scylla::SessionBuilder::new()
            .known_node(&cfg.cassandra_nodes)
            .build()
            .await?,
    );
    info!("Cassandra connected.");

    let mqtt_pub = mqtt::MqttPublisher::new(&cfg);

    let app = http::router(http::AppState {
        pool,
        cassandra,
        config: cfg.clone(),
        mqtt: mqtt_pub,
    });
    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("HTTP listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
