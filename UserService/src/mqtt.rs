use rumqttc::{AsyncClient, MqttOptions, QoS};
use tokio::task;
use tracing::{error, warn};

use crate::config::Config;

#[derive(Clone)]
pub struct MqttPublisher {
    client: AsyncClient,
}

impl MqttPublisher {
    /// Creates the MQTT client and spawns a background task that drains the
    /// event loop (required by rumqttc to keep the connection alive).
    pub fn new(cfg: &Config) -> Self {
        let mut opts =
            MqttOptions::new(&cfg.mqtt_client_id, &cfg.mqtt_host, cfg.mqtt_port);
        opts.set_keep_alive(std::time::Duration::from_secs(30));
        opts.set_clean_session(true);

        let (client, mut event_loop) = AsyncClient::new(opts, 64);

        // Drain the event loop in the background — rumqttc requires this
        // to actually send outgoing packets and acknowledge incoming ones.
        task::spawn(async move {
            loop {
                match event_loop.poll().await {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("MQTT event-loop error: {e}; reconnecting…");
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }
            }
        });

        Self { client }
    }

    /// Publish a JSON payload to the given topic (QoS 1, non-retained).
    pub async fn publish(&self, topic: &str, payload: String) {
        if let Err(e) = self
            .client
            .publish(topic, QoS::AtLeastOnce, false, payload.into_bytes())
            .await
        {
            error!("MQTT publish error on topic '{topic}': {e}");
        }
    }
}
