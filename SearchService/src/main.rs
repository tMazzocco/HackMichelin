mod config;
mod error;
mod es;
mod handlers;
mod mqtt;

use std::sync::Arc;

use axum::{
    http::{header, Method},
    routing::get,
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env (ignored if absent — env vars injected by Docker take precedence)
    let _ = dotenvy::dotenv();

    // Tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer())
        .init();

    let cfg = config::Config::from_env()?;
    info!("Starting SearchService on {}", cfg.http_addr);

    // Elasticsearch client
    let es_client = Arc::new(es::build_client(&cfg.es_host)?);
    info!("Elasticsearch client initialised ({})", cfg.es_host);

    // MQTT subscriber (background task)
    {
        let es2       = Arc::clone(&es_client);
        let mqtt_host = cfg.mqtt_host.clone();
        let mqtt_port = cfg.mqtt_port;
        tokio::spawn(async move {
            mqtt::start_mqtt_subscriber(es2, mqtt_host, mqtt_port).await;
        });
    }

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    // Router
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/search/restaurants", get(handlers::search_restaurants))
        .route("/api/search/users",       get(handlers::search_users))
        .layer(cors)
        .with_state(es_client);

    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    info!("HTTP listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
