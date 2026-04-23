use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use sqlx::PgPool;
use tracing::{error, info, warn};
use crate::{config::Config, db, models::PostEvent};

pub async fn run(cfg: Config, pool: PgPool) {
    let mut opts = MqttOptions::new(&cfg.mqtt_client_id, &cfg.mqtt_host, cfg.mqtt_port);
    opts.set_keep_alive(std::time::Duration::from_secs(30));
    let (client, mut eventloop) = AsyncClient::new(opts, 64);
    client.subscribe("post.created", QoS::AtLeastOnce).await.expect("subscribe post.created");
    client.subscribe("post.deleted", QoS::AtLeastOnce).await.expect("subscribe post.deleted");
    info!("StatsService MQTT subscribed to post.created + post.deleted");
    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(msg))) => {
                let topic = msg.topic.clone();
                let payload = msg.payload.clone();
                let pool2 = pool.clone();
                tokio::spawn(async move { handle(&pool2, &topic, &payload).await; });
            }
            Ok(_) => {}
            Err(e) => {
                error!("StatsService MQTT: {e}");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn handle(pool: &PgPool, topic: &str, payload: &[u8]) {
    let ev: PostEvent = match serde_json::from_slice(payload) {
        Ok(e) => e,
        Err(e) => { warn!("bad payload on {topic}: {e}"); return; }
    };
    let rid = match ev.restaurant_id.as_deref() {
        Some(r) if !r.is_empty() => r,
        _ => return,
    };
    let is_good = ev.rating.as_deref() == Some("GOOD");
    match topic {
        "post.created" => { if let Err(e) = db::upsert_on_create(pool, rid, is_good).await { error!("upsert_on_create: {e:?}"); } }
        "post.deleted" => { if let Err(e) = db::decrement_on_delete(pool, rid, is_good).await { error!("decrement_on_delete: {e:?}"); } }
        _ => {}
    }
}
