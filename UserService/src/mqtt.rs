use rumqttc::{AsyncClient, MqttOptions, QoS};
use tracing::error;
use crate::config::Config;

#[derive(Clone)]
pub struct MqttPublisher {
    client: AsyncClient,
}

impl MqttPublisher {
    pub fn new(cfg: &Config) -> Self {
        let mut opts = MqttOptions::new(&cfg.mqtt_client_id, &cfg.mqtt_host, cfg.mqtt_port);
        opts.set_keep_alive(std::time::Duration::from_secs(30));
        let (client, mut eventloop) = AsyncClient::new(opts, 64);
        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("MQTT publisher eventloop: {e}");
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });
        Self { client }
    }

    pub async fn publish(&self, topic: &str, payload: &serde_json::Value) {
        let bytes = payload.to_string().into_bytes();
        if let Err(e) = self.client.publish(topic, QoS::AtLeastOnce, false, bytes).await {
            error!("MQTT publish to {topic}: {e}");
        }
    }
}
