use std::sync::Arc;

use elasticsearch::Elasticsearch;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::Deserialize;
use serde_json::json;
use tracing::{error, info, warn};

use crate::es;

// ── Payload types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct MqttPayloadUserRegistered {
    pub user_id:  String,
    pub username: String,
    #[allow(dead_code)]
    pub email:    String,
}

#[derive(Debug, Deserialize)]
pub struct MqttPayloadUserUpdated {
    pub user_id:    String,
    pub username:   String,
    pub bio:        Option<String>,
    pub avatar_url: Option<String>,
}

// ── Subscriber ────────────────────────────────────────────────────────────────

/// Start a background MQTT subscriber that keeps the Elasticsearch `users`
/// index in sync with `user.registered` and `user.updated` events.
pub async fn start_mqtt_subscriber(es: Arc<Elasticsearch>, mqtt_host: String, mqtt_port: u16) {
    let mut opts = MqttOptions::new("search_service", &mqtt_host, mqtt_port);
    opts.set_keep_alive(std::time::Duration::from_secs(30));
    opts.set_clean_session(true);

    let (client, mut eventloop) = AsyncClient::new(opts, 64);

    client
        .subscribe("user.registered", QoS::AtLeastOnce)
        .await
        .expect("subscribe user.registered");
    client
        .subscribe("user.updated", QoS::AtLeastOnce)
        .await
        .expect("subscribe user.updated");

    info!("SearchService MQTT subscribed to user.registered + user.updated");

    // Keep the client alive so its subscriptions are not dropped.
    // We only need it for subscribing; the event loop does the real work.
    let _client = client;

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(msg))) => {
                let topic   = msg.topic.clone();
                let payload = msg.payload.clone();
                let es2     = Arc::clone(&es);

                tokio::spawn(async move {
                    handle(es2, &topic, &payload).await;
                });
            }
            Ok(_) => {}
            Err(e) => {
                error!("SearchService MQTT event loop error: {e}. Reconnecting in 5s …");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

// ── Event handler ─────────────────────────────────────────────────────────────

async fn handle(es: Arc<Elasticsearch>, topic: &str, payload: &[u8]) {
    match topic {
        "user.registered" => {
            let ev: MqttPayloadUserRegistered = match serde_json::from_slice(payload) {
                Ok(v) => v,
                Err(e) => {
                    warn!("bad payload on user.registered: {e}");
                    return;
                }
            };
            let doc = json!({
                "id":         ev.user_id,
                "username":   ev.username,
                "bio":        null,
                "avatar_url": null,
            });
            match es::index_user(&es, &ev.user_id.clone(), doc).await {
                Ok(_) => info!("Indexed user {} in ES", ev.user_id),
                Err(e) => error!("index_user failed for {}: {e}", ev.user_id),
            }
        }
        "user.updated" => {
            let ev: MqttPayloadUserUpdated = match serde_json::from_slice(payload) {
                Ok(v) => v,
                Err(e) => {
                    warn!("bad payload on user.updated: {e}");
                    return;
                }
            };
            let doc = json!({
                "username":   ev.username,
                "bio":        ev.bio,
                "avatar_url": ev.avatar_url,
            });
            match es::update_user(&es, &ev.user_id.clone(), doc).await {
                Ok(_) => info!("Updated user {} in ES", ev.user_id),
                Err(e) => error!("update_user failed for {}: {e}", ev.user_id),
            }
        }
        other => {
            warn!("SearchService MQTT: unexpected topic '{other}'");
        }
    }
}
