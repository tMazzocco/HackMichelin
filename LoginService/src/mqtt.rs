use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use tracing::{error, info};

use crate::config::Config;

/// Thin async MQTT publisher.
/// Spawns a background task that drains the rumqttc event loop so
/// acknowledgements are processed and the internal buffer never fills up.
#[derive(Clone)]
pub struct MqttPublisher {
    client: AsyncClient,
}

impl MqttPublisher {
    /// Build the client from config and immediately start the drain task.
    pub fn new(cfg: &Config) -> Self {
        let mut opts = MqttOptions::new(
            &cfg.mqtt_client_id,
            &cfg.mqtt_host,
            cfg.mqtt_port,
        );
        opts.set_keep_alive(std::time::Duration::from_secs(30));
        opts.set_clean_session(true);

        let (client, eventloop) = AsyncClient::new(opts, 64);

        // Drain the event loop in the background so ACKs are consumed
        tokio::spawn(drain_eventloop(eventloop));

        info!(
            "MQTT publisher initialised ({}:{}, client_id={})",
            cfg.mqtt_host, cfg.mqtt_port, cfg.mqtt_client_id
        );

        Self { client }
    }

    /// Publish a JSON payload to `topic` with QoS AtLeastOnce.
    /// Errors are logged but not propagated — MQTT is best-effort here.
    pub async fn publish(&self, topic: &str, payload: &serde_json::Value) {
        let bytes = payload.to_string().into_bytes();
        if let Err(e) = self
            .client
            .publish(topic, QoS::AtLeastOnce, false, bytes)
            .await
        {
            error!("MQTT publish to '{topic}' failed: {e}");
        }
    }
}

/// Endlessly polls the event loop so the client stays connected and
/// PubAck packets are consumed (preventing buffer back-pressure).
async fn drain_eventloop(mut eventloop: EventLoop) {
    loop {
        match eventloop.poll().await {
            Ok(_event) => {} // We don't need to inspect events here
            Err(e) => {
                error!("MQTT event loop error: {e}. Reconnecting in 5s …");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}
