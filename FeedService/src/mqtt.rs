use std::sync::Arc;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use tracing::{error, info, warn};
use uuid::Uuid;
use crate::{config::Config, db_cql, models::PostCreatedEvent};

pub async fn run(cfg: Config, cassandra: Arc<scylla::Session>) {
    let mut opts = MqttOptions::new(&cfg.mqtt_client_id, &cfg.mqtt_host, cfg.mqtt_port);
    opts.set_keep_alive(std::time::Duration::from_secs(30));
    let (client, mut eventloop) = AsyncClient::new(opts, 64);
    client.subscribe("post.created", QoS::AtLeastOnce).await
        .expect("MQTT subscribe post.created failed");
    info!("FeedService MQTT subscribed to post.created");
    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(msg))) => {
                let payload = msg.payload.clone();
                let cass = cassandra.clone();
                tokio::spawn(async move {
                    handle_post_created(cass, &payload).await;
                });
            }
            Ok(_) => {}
            Err(e) => {
                error!("FeedService MQTT error: {e}");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn handle_post_created(cassandra: Arc<scylla::Session>, payload: &[u8]) {
    let ev: PostCreatedEvent = match serde_json::from_slice(payload) {
        Ok(e) => e,
        Err(e) => { warn!("bad post.created payload: {e}"); return; }
    };
    let author_id = match ev.user_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => { warn!("invalid user_id in post.created"); return; }
    };
    let post_id = match ev.post_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => { warn!("invalid post_id in post.created"); return; }
    };
    let created_at = ev.created_at.as_deref()
        .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok())
        .unwrap_or_else(chrono::Utc::now);
    let followers = match db_cql::get_followers(&cassandra, author_id).await {
        Ok(f) => f,
        Err(e) => { error!("get_followers: {e:?}"); return; }
    };
    for viewer_id in followers {
        let cass = cassandra.clone();
        let author_name = ev.username.clone();
        let rid = ev.restaurant_id.clone();
        let rname = ev.restaurant_name.clone();
        let mtype = ev.media_type.clone();
        let murl = ev.media_url.clone();
        let turl = ev.thumbnail_url.clone();
        let cap = ev.caption.clone();
        let rat = ev.rating.clone();
        tokio::spawn(async move {
            db_cql::fan_out(cass, viewer_id, post_id, created_at, author_id, author_name, rid, rname, mtype, murl, turl, cap, rat).await;
        });
    }
}
