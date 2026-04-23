use std::sync::Arc;
use chrono::{DateTime, Utc};
use scylla::IntoTypedRows;
use uuid::Uuid;
use crate::{error::AppError, models::FeedItem};

pub async fn get_followers(session: &Arc<scylla::Session>, author_id: Uuid) -> Result<Vec<Uuid>, AppError> {
    let result = session.query(
        "SELECT follower_id FROM hackmichelin.user_followers WHERE followed_id = ?",
        (author_id,),
    ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;
    let mut ids = vec![];
    if let Some(rows) = result.rows {
        for row in rows.into_typed::<(Uuid,)>() {
            if let Ok((id,)) = row { ids.push(id); }
        }
    }
    Ok(ids)
}

pub async fn fan_out(
    session: Arc<scylla::Session>,
    viewer_id: Uuid,
    post_id: Uuid,
    created_at: DateTime<Utc>,
    author_id: Uuid,
    author_name: String,
    restaurant_id: Option<String>,
    restaurant_name: Option<String>,
    media_type: Option<String>,
    media_url: Option<String>,
    thumbnail_url: Option<String>,
    caption: Option<String>,
    rating: Option<String>,
) {
    if let Err(e) = session.query(
        "INSERT INTO hackmichelin.user_feed (viewer_id, created_at, post_id, author_id, author_name, restaurant_id, restaurant_name, media_type, media_url, thumbnail_url, caption, rating) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) USING TTL 2592000",
        (viewer_id, created_at, post_id, author_id, &author_name, restaurant_id.as_deref(), restaurant_name.as_deref(), media_type.as_deref(), media_url.as_deref(), thumbnail_url.as_deref(), caption.as_deref(), rating.as_deref()),
    ).await {
        tracing::error!("fan_out to {viewer_id}: {e}");
    }
}

pub async fn get_feed(
    session: &Arc<scylla::Session>,
    viewer_id: Uuid,
    before: DateTime<Utc>,
    limit: i32,
) -> Result<Vec<FeedItem>, AppError> {
    let result = session.query(
        "SELECT post_id, created_at, author_id, author_name, restaurant_id, restaurant_name, media_type, media_url, thumbnail_url, caption, rating FROM hackmichelin.user_feed WHERE viewer_id = ? AND created_at < ? LIMIT ?",
        (viewer_id, before, limit),
    ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;
    let mut items = vec![];
    if let Some(rows) = result.rows {
        type Row = (Uuid, Option<DateTime<Utc>>, Option<Uuid>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>);
        for row in rows.into_typed::<Row>() {
            if let Ok((pid, cat, aid, aname, rid, rname, mtype, murl, turl, cap, rat)) = row {
                items.push(FeedItem {
                    post_id: pid, created_at: cat, author_id: aid, author_name: aname,
                    restaurant_id: rid, restaurant_name: rname,
                    media_type: mtype, media_url: murl, thumbnail_url: turl,
                    caption: cap, rating: rat,
                });
            }
        }
    }
    Ok(items)
}
