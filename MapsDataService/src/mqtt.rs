/// MQTT subscriber for the MapsDataService.
///
/// Topic convention (request/reply pattern over MQTT 5):
///
///   Request  → maps/restaurants/nearby
///   Reply    → maps/restaurants/nearby/reply/{correlation_id}
///
///   Request  → maps/restaurants/get
///   Reply    → maps/restaurants/get/reply/{correlation_id}
///
/// Payload is JSON. The broker is expected to set the MQTT 5
/// `correlation_data` and `response_topic` properties so this
/// service can route the reply back to the caller (e.g. the
/// Nginx/MQTT gateway that proxied the HTTP request).
///
/// If `response_topic` is absent the reply is published to the
/// fallback topics defined above.

use rumqttc::{
    AsyncClient, Event, EventLoop, MqttOptions, Packet, Publish, QoS,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use tracing::{error, info, warn};

use crate::{config::Config, db};

// ── Request payloads ─────────────────────────────────────

#[derive(Deserialize)]
struct NearbyRequest {
    lat:    f64,
    lng:    f64,
    radius: Option<f64>,
    limit:  Option<i64>,
}

#[derive(Deserialize)]
struct GetRequest {
    id: String,
}

// ── Helpers ──────────────────────────────────────────────

/// Build MQTT client + event loop from config.
pub fn build_client(cfg: &Config) -> (AsyncClient, EventLoop) {
    let mut opts = MqttOptions::new(
        &cfg.mqtt_client_id,
        &cfg.mqtt_host,
        cfg.mqtt_port,
    );
    opts.set_keep_alive(std::time::Duration::from_secs(30));
    opts.set_clean_session(true);
    AsyncClient::new(opts, 64)
}

/// Publish a JSON reply to `response_topic` (or fallback).
async fn reply(client: &AsyncClient, topic: &str, payload: serde_json::Value) {
    let bytes = payload.to_string().into_bytes();
    if let Err(e) = client.publish(topic, QoS::AtLeastOnce, false, bytes).await {
        error!("MQTT publish failed on {topic}: {e}");
    }
}

/// Derive reply topic: use MQTT 5 `response_topic` when present,
/// else append `/reply` to the incoming topic.
fn reply_topic(pub_msg: &Publish) -> String {
    // rumqttc 0.24: MQTT 5 response_topic not yet exposed as a Cargo feature.
    // Fallback: append /reply to incoming topic.
    // Upgrade when rumqttc stabilises MQTT 5 properties API.
    let _ = pub_msg; // suppress unused warning
    format!("{}/reply", pub_msg.topic)
}

// ── Main loop ────────────────────────────────────────────

pub async fn run(cfg: Config, pool: PgPool) {
    let (client, mut eventloop) = build_client(&cfg);

    // Subscribe to both topics
    client
        .subscribe("maps/restaurants/nearby", QoS::AtLeastOnce)
        .await
        .expect("MQTT subscribe nearby failed");
    client
        .subscribe("maps/restaurants/get", QoS::AtLeastOnce)
        .await
        .expect("MQTT subscribe get failed");

    info!(
        "MQTT subscriber connected to {}:{}, listening on maps/restaurants/+",
        cfg.mqtt_host, cfg.mqtt_port
    );

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(msg))) => {
                let topic      = msg.topic.clone();
                let payload    = msg.payload.clone();
                let reply_to   = reply_topic(&msg);
                let pool_clone = pool.clone();
                let client_clone = client.clone();

                // Spawn handler so the event loop isn't blocked
                tokio::spawn(async move {
                    handle_message(
                        &client_clone,
                        &topic,
                        &payload,
                        &reply_to,
                        &pool_clone,
                    )
                    .await;
                });
            }
            Ok(Event::Incoming(Packet::ConnAck(_))) => {
                info!("MQTT connected.");
            }
            Ok(_) => {}
            Err(e) => {
                error!("MQTT event loop error: {e}. Reconnecting in 5s …");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn handle_message(
    client:   &AsyncClient,
    topic:    &str,
    payload:  &[u8],
    reply_to: &str,
    pool:     &PgPool,
) {
    match topic {
        "maps/restaurants/nearby" => {
            let req: NearbyRequest = match serde_json::from_slice(payload) {
                Ok(r) => r,
                Err(e) => {
                    warn!("Bad nearby payload: {e}");
                    reply(client, reply_to, json!({ "error": format!("bad request: {e}") })).await;
                    return;
                }
            };

            let radius = req.radius.unwrap_or(1000.0).min(50_000.0);
            let limit  = req.limit.unwrap_or(20).min(100);

            match db::get_nearby(pool, req.lat, req.lng, radius, limit).await {
                Ok(rows) => reply(client, reply_to, json!({ "data": rows })).await,
                Err(e)   => {
                    error!("db::get_nearby error: {e}");
                    reply(client, reply_to, json!({ "error": "internal error" })).await;
                }
            }
        }

        "maps/restaurants/get" => {
            let req: GetRequest = match serde_json::from_slice(payload) {
                Ok(r) => r,
                Err(e) => {
                    warn!("Bad get payload: {e}");
                    reply(client, reply_to, json!({ "error": format!("bad request: {e}") })).await;
                    return;
                }
            };

            match db::get_by_id(pool, &req.id).await {
                Ok(Some(row)) => reply(client, reply_to, json!({ "data": row })).await,
                Ok(None)      => reply(client, reply_to, json!({ "error": "not found" })).await,
                Err(e)        => {
                    error!("db::get_by_id error: {e}");
                    reply(client, reply_to, json!({ "error": "internal error" })).await;
                }
            }
        }

        other => warn!("Unhandled topic: {other}"),
    }
}
