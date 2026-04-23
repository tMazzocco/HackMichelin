mod auth;
mod config;
mod db_cql;
mod error;
mod http;
mod models;

use std::sync::Arc;

use scylla::SessionBuilder;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

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


    let session = Arc::new(session);

    let state = http::AppState {
        cassandra: session,
        config: cfg.clone(),
    };

    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    info!("CommentService listening on {}", addr);

    let router = http::build_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
