mod config;
mod error;
mod http;

use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env (ignored if absent — docker injects env vars directly)
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer())
        .init();

    let cfg = config::Config::from_env()?;
    info!("Starting DownloadService on {}", cfg.http_addr);
    info!("Serving media from  {}", cfg.media_dir);

    let app = http::router(http::AppState { config: cfg.clone() });
    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;

    info!("HTTP listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
