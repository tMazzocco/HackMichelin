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
use scylla::IntoTypedRows;
use tracing::{info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer())
        .init();

    let cfg = config::Config::from_env()?;
    info!("Starting PostService on {}", cfg.http_addr);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cfg.database_url)
        .await?;

    let cassandra = Arc::new(
        scylla::SessionBuilder::new()
            .known_node(&cfg.cassandra_nodes)
            .build()
            .await?,
    );

    for table in &["posts", "user_posts", "restaurant_posts"] {
        match cassandra.query(
            "SELECT column_name, type FROM system_schema.columns WHERE keyspace_name = 'hackmichelin' AND table_name = ?",
            (table,),
        ).await {
            Ok(result) => {
                if let Some(rows) = result.rows {
                    let cols: Vec<String> = rows
                        .into_typed::<(String, String)>()
                        .filter_map(|r| r.ok())
                        .map(|(col, typ)| format!("{col} ({typ})"))
                        .collect();
                    info!("hackmichelin.{table} columns: [{}]", cols.join(", "));
                }
            }
            Err(e) => warn!("Could not introspect {table} schema: {e}"),
        }
    }

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
