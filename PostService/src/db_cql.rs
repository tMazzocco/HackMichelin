use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use scylla::IntoTypedRows;
use tracing::{debug, error};
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
    debug!(post_id = %post_id, user_id = %user_id, "inserting post into hackmichelin.posts");
    // Canonical lookup table
    session.query(
        "INSERT INTO hackmichelin.posts (post_id, user_id, username, restaurant_id, restaurant_name, media_id, media_type, media_url, thumbnail_url, caption, rating, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (post_id, user_id, username, restaurant_id, restaurant_name, media_id, media_type, media_url, thumbnail_url, caption, rating, created_at),
    ).await.map_err(|e| { error!(post_id = %post_id, error = %e, "failed to insert into hackmichelin.posts"); AppError::Cassandra(e.to_string()) })?;

    debug!(post_id = %post_id, user_id = %user_id, "inserting post into hackmichelin.user_posts");
    // User timeline
    session.query(
        "INSERT INTO hackmichelin.user_posts (user_id, created_at, post_id, restaurant_id, restaurant_name, media_type, media_url, thumbnail_url, caption, rating) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (user_id, created_at, post_id, restaurant_id, restaurant_name, media_type, media_url, thumbnail_url, caption, rating),
    ).await.map_err(|e| { error!(post_id = %post_id, error = %e, "failed to insert into hackmichelin.user_posts"); AppError::Cassandra(e.to_string()) })?;

    // Restaurant feed (only if tagged)
    if let Some(rid) = restaurant_id {
        debug!(post_id = %post_id, restaurant_id = %rid, "inserting post into hackmichelin.restaurant_posts");
        session.query(
            "INSERT INTO hackmichelin.restaurant_posts (restaurant_id, created_at, post_id, user_id, username, media_type, media_url, thumbnail_url, caption, rating) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (rid, created_at, post_id, user_id, username, media_type, media_url, thumbnail_url, caption, rating),
        ).await.map_err(|e| { error!(post_id = %post_id, restaurant_id = %rid, error = %e, "failed to insert into hackmichelin.restaurant_posts"); AppError::Cassandra(e.to_string()) })?;
    }
    Ok(())
}

pub async fn get_post(session: &Arc<scylla::Session>, post_id: Uuid) -> Result<Option<Post>, AppError> {
    debug!(post_id = %post_id, "querying hackmichelin.posts by post_id");
    let result = session.query(
        "SELECT post_id, user_id, restaurant_id, media_id, media_type, media_url, thumbnail_url, caption, rating, created_at FROM hackmichelin.posts WHERE post_id = ?",
        (post_id,),
    ).await.map_err(|e| { error!(post_id = %post_id, error = %e, "failed to query hackmichelin.posts"); AppError::Cassandra(e.to_string()) })?;

    if let Some(rows) = result.rows {
        type Row = (Uuid, Option<Uuid>, Option<String>, Option<Uuid>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<DateTime<Utc>>);
        if let Some(Ok((pid, uid, rid, mid, mtype, murl, turl, cap, rat, cat))) = rows.into_typed::<Row>().next() {
            return Ok(Some(Post {
                post_id: pid,
                user_id: uid,
                username: None,
                restaurant_id: rid,
                restaurant_name: None,
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
    debug!(post_id = %post_id, user_id = %user_id, "deleting post from cassandra");
    session.query("DELETE FROM hackmichelin.posts WHERE post_id = ?", (post_id,))
        .await.map_err(|e| { error!(post_id = %post_id, error = %e, "failed to delete from hackmichelin.posts"); AppError::Cassandra(e.to_string()) })?;
    session.query(
        "DELETE FROM hackmichelin.user_posts WHERE user_id = ? AND created_at = ? AND post_id = ?",
        (user_id, created_at, post_id),
    ).await.map_err(|e| { error!(post_id = %post_id, user_id = %user_id, error = %e, "failed to delete from hackmichelin.user_posts"); AppError::Cassandra(e.to_string()) })?;
    if let Some(rid) = restaurant_id {
        session.query(
            "DELETE FROM hackmichelin.restaurant_posts WHERE restaurant_id = ? AND created_at = ? AND post_id = ?",
            (rid, created_at, post_id),
        ).await.map_err(|e| { error!(post_id = %post_id, restaurant_id = %rid, error = %e, "failed to delete from hackmichelin.restaurant_posts"); AppError::Cassandra(e.to_string()) })?;
    }
    Ok(())
}

pub async fn list_user_posts(
    session: &Arc<scylla::Session>,
    user_id: Uuid,
    before: DateTime<Utc>,
    limit: i32,
) -> Result<Vec<Post>, AppError> {
    debug!(user_id = %user_id, limit = limit, "querying hackmichelin.user_posts");
    let result = session.query(
        "SELECT post_id, created_at, restaurant_id, restaurant_name, media_type, media_url, thumbnail_url, caption, rating FROM hackmichelin.user_posts WHERE user_id = ? AND created_at < ? LIMIT ?",
        (user_id, before, limit),
    ).await.map_err(|e| { error!(user_id = %user_id, error = %e, "failed to query hackmichelin.user_posts"); AppError::Cassandra(e.to_string()) })?;
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

pub async fn get_random_posts(session: &Arc<scylla::Session>, count: usize) -> Result<Vec<Post>, AppError> {
    type Row = (Uuid, Option<Uuid>, Option<String>, Option<Uuid>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<DateTime<Utc>>);

    let fetch = (count * 10).max(50) as i32;
    debug!(count = count, fetch_limit = fetch, "querying random posts from hackmichelin.posts");
    let result = session.query(
        "SELECT post_id, user_id, restaurant_id, media_id, media_type, media_url, thumbnail_url, caption, rating, created_at FROM hackmichelin.posts LIMIT ?",
        (fetch,),
    ).await.map_err(|e| { error!(error = %e, query = "SELECT ... FROM hackmichelin.posts LIMIT ?", "get_random_posts query failed"); AppError::Cassandra(e.to_string()) })?;

    let mut posts: Vec<Post> = vec![];
    if let Some(rows) = result.rows {
        for row in rows.into_typed::<Row>() {
            if let Ok((pid, uid, rid, mid, mtype, murl, turl, cap, rat, cat)) = row {
                posts.push(Post {
                    post_id: pid, user_id: uid, username: None,
                    restaurant_id: rid, restaurant_name: None,
                    media_id: mid, media_type: mtype, media_url: murl,
                    thumbnail_url: turl, caption: cap, rating: rat, created_at: cat,
                });
            }
        }
    }

    use rand::seq::SliceRandom;
    posts.shuffle(&mut rand::thread_rng());
    posts.truncate(count);
    debug!(returned = posts.len(), "get_random_posts completed");
    Ok(posts)
}

pub async fn list_restaurant_posts(
    session: &Arc<scylla::Session>,
    restaurant_id: &str,
    before: DateTime<Utc>,
    limit: i32,
) -> Result<Vec<Post>, AppError> {
    debug!(restaurant_id = %restaurant_id, limit = limit, "querying hackmichelin.restaurant_posts");
    let result = session.query(
        "SELECT post_id, created_at, user_id, username, media_type, media_url, thumbnail_url, caption, rating FROM hackmichelin.restaurant_posts WHERE restaurant_id = ? AND created_at < ? LIMIT ?",
        (restaurant_id, before, limit),
    ).await.map_err(|e| { error!(restaurant_id = %restaurant_id, error = %e, "failed to query hackmichelin.restaurant_posts"); AppError::Cassandra(e.to_string()) })?;
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
