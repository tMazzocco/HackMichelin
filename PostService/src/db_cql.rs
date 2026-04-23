use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::{error::AppError, models::Post};

pub async fn insert_post(
    session: &Arc<scylla::Session>,
    post_id: Uuid,
    user_id: Uuid,
    username: &str,
    restaurant_id: Option<&str>,
    restaurant_name: Option<&str>,
    media_id: Uuid,
    media_type: &str,
    media_url: &str,
    thumbnail_url: Option<&str>,
    caption: Option<&str>,
    rating: &str,
    created_at: DateTime<Utc>,
) -> Result<(), AppError> {
    // Canonical lookup table
    session.query(
        "INSERT INTO posts (post_id, user_id, username, restaurant_id, restaurant_name, media_id, media_type, media_url, thumbnail_url, caption, rating, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (post_id, user_id, username, restaurant_id, restaurant_name, media_id, media_type, media_url, thumbnail_url, caption, rating, created_at),
    ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;

    // User timeline
    session.query(
        "INSERT INTO user_posts (user_id, created_at, post_id, restaurant_id, restaurant_name, media_type, media_url, thumbnail_url, caption, rating) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (user_id, created_at, post_id, restaurant_id, restaurant_name, media_type, media_url, thumbnail_url, caption, rating),
    ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;

    // Restaurant feed (only if tagged)
    if let Some(rid) = restaurant_id {
        session.query(
            "INSERT INTO restaurant_posts (restaurant_id, created_at, post_id, user_id, username, media_type, media_url, thumbnail_url, caption, rating) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (rid, created_at, post_id, user_id, username, media_type, media_url, thumbnail_url, caption, rating),
        ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;
    }
    Ok(())
}

pub async fn get_post(session: &Arc<scylla::Session>, post_id: Uuid) -> Result<Option<Post>, AppError> {
    let result = session.query(
        "SELECT post_id, user_id, username, restaurant_id, restaurant_name, media_id, media_type, media_url, thumbnail_url, caption, rating, created_at FROM posts WHERE post_id = ?",
        (post_id,),
    ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;

    if let Some(rows) = result.rows {
        type Row = (Uuid, Option<Uuid>, Option<String>, Option<String>, Option<String>, Option<Uuid>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<DateTime<Utc>>);
        if let Some(Ok((pid, uid, uname, rid, rname, mid, mtype, murl, turl, cap, rat, cat))) = rows.into_typed::<Row>().next() {
            return Ok(Some(Post {
                post_id: pid,
                user_id: uid,
                username: uname,
                restaurant_id: rid,
                restaurant_name: rname,
                media_id: mid,
                media_type: mtype,
                media_url: murl,
                thumbnail_url: turl,
                caption: cap,
                rating: rat,
                created_at: cat,
            }));
        }
    }
    Ok(None)
}

pub async fn delete_post(
    session: &Arc<scylla::Session>,
    post_id: Uuid,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    restaurant_id: Option<&str>,
) -> Result<(), AppError> {
    session.query("DELETE FROM posts WHERE post_id = ?", (post_id,))
        .await.map_err(|e| AppError::Cassandra(e.to_string()))?;
    session.query(
        "DELETE FROM user_posts WHERE user_id = ? AND created_at = ? AND post_id = ?",
        (user_id, created_at, post_id),
    ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;
    if let Some(rid) = restaurant_id {
        session.query(
            "DELETE FROM restaurant_posts WHERE restaurant_id = ? AND created_at = ? AND post_id = ?",
            (rid, created_at, post_id),
        ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;
    }
    Ok(())
}

pub async fn list_user_posts(
    session: &Arc<scylla::Session>,
    user_id: Uuid,
    before: DateTime<Utc>,
    limit: i32,
) -> Result<Vec<Post>, AppError> {
    let result = session.query(
        "SELECT post_id, created_at, restaurant_id, restaurant_name, media_type, media_url, thumbnail_url, caption, rating FROM user_posts WHERE user_id = ? AND created_at < ? LIMIT ?",
        (user_id, before, limit),
    ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;
    let mut out = vec![];
    if let Some(rows) = result.rows {
        type Row = (Uuid, Option<DateTime<Utc>>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>);
        for row in rows.into_typed::<Row>() {
            if let Ok((pid, cat, rid, rname, mtype, murl, turl, cap, rat)) = row {
                out.push(Post {
                    post_id: pid, user_id: Some(user_id), username: None,
                    restaurant_id: rid, restaurant_name: rname,
                    media_id: None, media_type: mtype, media_url: murl,
                    thumbnail_url: turl, caption: cap, rating: rat, created_at: cat,
                });
            }
        }
    }
    Ok(out)
}

pub async fn list_restaurant_posts(
    session: &Arc<scylla::Session>,
    restaurant_id: &str,
    before: DateTime<Utc>,
    limit: i32,
) -> Result<Vec<Post>, AppError> {
    let result = session.query(
        "SELECT post_id, created_at, user_id, username, media_type, media_url, thumbnail_url, caption, rating FROM restaurant_posts WHERE restaurant_id = ? AND created_at < ? LIMIT ?",
        (restaurant_id, before, limit),
    ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;
    let mut out = vec![];
    if let Some(rows) = result.rows {
        type Row = (Uuid, Option<DateTime<Utc>>, Option<Uuid>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>);
        for row in rows.into_typed::<Row>() {
            if let Ok((pid, cat, uid, uname, mtype, murl, turl, cap, rat)) = row {
                out.push(Post {
                    post_id: pid, user_id: uid, username: uname,
                    restaurant_id: Some(restaurant_id.to_string()), restaurant_name: None,
                    media_id: None, media_type: mtype, media_url: murl,
                    thumbnail_url: turl, caption: cap, rating: rat, created_at: cat,
                });
            }
        }
    }
    Ok(out)
}
