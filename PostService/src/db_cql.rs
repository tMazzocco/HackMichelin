use std::sync::Arc;

use chrono::{DateTime, Utc};
use scylla::FromRow;
use uuid::Uuid;

use crate::{error::AppError, models::Post};

// ── Internal row structs ──────────────────────────────────────────────────────

/// Full posts table row.
#[derive(Debug, FromRow)]
struct PostRow {
    post_id: Uuid,
    user_id: Uuid,
    username: Option<String>,
    restaurant_id: Option<String>,
    restaurant_name: Option<String>,
    media_id: Option<Uuid>,
    media_type: Option<String>,
    media_url: Option<String>,
    thumbnail_url: Option<String>,
    caption: Option<String>,
    rating: Option<String>,
    created_at: Option<chrono::Duration>, // scylla maps timestamp → chrono::Duration ms since epoch
}

/// user_posts clustering table row.
#[derive(Debug, FromRow)]
struct UserPostRow {
    user_id: Uuid,
    created_at: Option<chrono::Duration>,
    post_id: Uuid,
    restaurant_id: Option<String>,
    restaurant_name: Option<String>,
    media_type: Option<String>,
    media_url: Option<String>,
    thumbnail_url: Option<String>,
    caption: Option<String>,
    rating: Option<String>,
}

/// restaurant_posts clustering table row.
#[derive(Debug, FromRow)]
struct RestaurantPostRow {
    restaurant_id: String,
    created_at: Option<chrono::Duration>,
    post_id: Uuid,
    user_id: Option<Uuid>,
    username: Option<String>,
    media_type: Option<String>,
    media_url: Option<String>,
    thumbnail_url: Option<String>,
    caption: Option<String>,
    rating: Option<String>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn scylla_err(e: impl std::fmt::Display) -> AppError {
    AppError::Cassandra(e.to_string())
}

/// Convert a scylla `chrono::Duration` (milliseconds since Unix epoch) to
/// `DateTime<Utc>`.
fn duration_to_datetime(d: chrono::Duration) -> DateTime<Utc> {
    use chrono::TimeZone;
    Utc.timestamp_millis_opt(d.num_milliseconds())
        .single()
        .unwrap_or_else(|| Utc::now())
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Insert a post into the three Cassandra tables (posts, user_posts, and
/// optionally restaurant_posts).
#[allow(clippy::too_many_arguments)]
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
    let ts = chrono::Duration::milliseconds(created_at.timestamp_millis());

    // 1. posts table
    session
        .query(
            "INSERT INTO hackmichelin.posts \
             (post_id, user_id, username, restaurant_id, restaurant_name, \
              media_id, media_type, media_url, thumbnail_url, caption, rating, created_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                post_id,
                user_id,
                username,
                restaurant_id,
                restaurant_name,
                media_id,
                media_type,
                media_url,
                thumbnail_url,
                caption,
                rating,
                ts,
            ),
        )
        .await
        .map_err(scylla_err)?;

    // 2. user_posts table
    session
        .query(
            "INSERT INTO hackmichelin.user_posts \
             (user_id, created_at, post_id, restaurant_id, restaurant_name, \
              media_type, media_url, thumbnail_url, caption, rating) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                user_id,
                ts,
                post_id,
                restaurant_id,
                restaurant_name,
                media_type,
                media_url,
                thumbnail_url,
                caption,
                rating,
            ),
        )
        .await
        .map_err(scylla_err)?;

    // 3. restaurant_posts table (conditional)
    if let Some(rid) = restaurant_id {
        session
            .query(
                "INSERT INTO hackmichelin.restaurant_posts \
                 (restaurant_id, created_at, post_id, user_id, username, \
                  media_type, media_url, thumbnail_url, caption, rating) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    rid,
                    ts,
                    post_id,
                    user_id,
                    username,
                    media_type,
                    media_url,
                    thumbnail_url,
                    caption,
                    rating,
                ),
            )
            .await
            .map_err(scylla_err)?;
    }

    Ok(())
}

/// Fetch a single post by its primary key.
pub async fn get_post(
    session: &Arc<scylla::Session>,
    post_id: Uuid,
) -> Result<Option<Post>, AppError> {
    let result = session
        .query(
            "SELECT post_id, user_id, username, restaurant_id, restaurant_name, \
              media_id, media_type, media_url, thumbnail_url, caption, rating, created_at \
             FROM hackmichelin.posts \
             WHERE post_id = ?",
            (post_id,),
        )
        .await
        .map_err(scylla_err)?;

    let mut rows = result.rows_typed::<PostRow>().map_err(scylla_err)?;

    match rows.next() {
        Some(Ok(r)) => Ok(Some(Post {
            post_id: r.post_id,
            user_id: r.user_id,
            username: r.username,
            restaurant_id: r.restaurant_id,
            restaurant_name: r.restaurant_name,
            media_id: r.media_id,
            media_type: r.media_type,
            media_url: r.media_url,
            thumbnail_url: r.thumbnail_url,
            caption: r.caption,
            rating: r.rating,
            created_at: r.created_at.map(duration_to_datetime),
        })),
        Some(Err(e)) => Err(scylla_err(e)),
        None => Ok(None),
    }
}

/// Delete a post from all three tables.
pub async fn delete_post(
    session: &Arc<scylla::Session>,
    post_id: Uuid,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    restaurant_id: Option<&str>,
) -> Result<(), AppError> {
    let ts = chrono::Duration::milliseconds(created_at.timestamp_millis());

    // 1. posts
    session
        .query(
            "DELETE FROM hackmichelin.posts WHERE post_id = ?",
            (post_id,),
        )
        .await
        .map_err(scylla_err)?;

    // 2. user_posts
    session
        .query(
            "DELETE FROM hackmichelin.user_posts \
             WHERE user_id = ? AND created_at = ? AND post_id = ?",
            (user_id, ts, post_id),
        )
        .await
        .map_err(scylla_err)?;

    // 3. restaurant_posts (conditional)
    if let Some(rid) = restaurant_id {
        session
            .query(
                "DELETE FROM hackmichelin.restaurant_posts \
                 WHERE restaurant_id = ? AND created_at = ? AND post_id = ?",
                (rid, ts, post_id),
            )
            .await
            .map_err(scylla_err)?;
    }

    Ok(())
}

/// Return a page of posts for a user, ordered newest-first, with a
/// `created_at < before` cursor.
pub async fn list_user_posts(
    session: &Arc<scylla::Session>,
    user_id: Uuid,
    before: DateTime<Utc>,
    limit: i32,
) -> Result<Vec<Post>, AppError> {
    let ts = chrono::Duration::milliseconds(before.timestamp_millis());

    let result = session
        .query(
            "SELECT user_id, created_at, post_id, restaurant_id, restaurant_name, \
              media_type, media_url, thumbnail_url, caption, rating \
             FROM hackmichelin.user_posts \
             WHERE user_id = ? AND created_at < ? \
             LIMIT ?",
            (user_id, ts, limit),
        )
        .await
        .map_err(scylla_err)?;

    let rows = result
        .rows_typed::<UserPostRow>()
        .map_err(scylla_err)?;

    let mut posts = Vec::new();
    for row in rows {
        let r = row.map_err(scylla_err)?;
        posts.push(Post {
            post_id: r.post_id,
            user_id: r.user_id,
            username: None,
            restaurant_id: r.restaurant_id,
            restaurant_name: r.restaurant_name,
            media_id: None,
            media_type: r.media_type,
            media_url: r.media_url,
            thumbnail_url: r.thumbnail_url,
            caption: r.caption,
            rating: r.rating,
            created_at: r.created_at.map(duration_to_datetime),
        });
    }

    Ok(posts)
}

/// Return a page of posts for a restaurant, ordered newest-first, with a
/// `created_at < before` cursor.
pub async fn list_restaurant_posts(
    session: &Arc<scylla::Session>,
    restaurant_id: &str,
    before: DateTime<Utc>,
    limit: i32,
) -> Result<Vec<Post>, AppError> {
    let ts = chrono::Duration::milliseconds(before.timestamp_millis());

    let result = session
        .query(
            "SELECT restaurant_id, created_at, post_id, user_id, username, \
              media_type, media_url, thumbnail_url, caption, rating \
             FROM hackmichelin.restaurant_posts \
             WHERE restaurant_id = ? AND created_at < ? \
             LIMIT ?",
            (restaurant_id, ts, limit),
        )
        .await
        .map_err(scylla_err)?;

    let rows = result
        .rows_typed::<RestaurantPostRow>()
        .map_err(scylla_err)?;

    let mut posts = Vec::new();
    for row in rows {
        let r = row.map_err(scylla_err)?;
        posts.push(Post {
            post_id: r.post_id,
            user_id: r.user_id.unwrap_or_else(Uuid::nil),
            username: r.username,
            restaurant_id: Some(r.restaurant_id),
            restaurant_name: None,
            media_id: None,
            media_type: r.media_type,
            media_url: r.media_url,
            thumbnail_url: r.thumbnail_url,
            caption: r.caption,
            rating: r.rating,
            created_at: r.created_at.map(duration_to_datetime),
        });
    }

    Ok(posts)
}
