use chrono::{DateTime, Utc};
use scylla::frame::value::Counter;
use scylla::Session;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{LikeCount, LikeEntry};

/// Insert into post_likes using IF NOT EXISTS, then increment the counter.
pub async fn like_post(
    session: &Arc<Session>,
    post_id: Uuid,
    user_id: Uuid,
    username: &str,
) -> Result<(), AppError> {
    let liked_at: DateTime<Utc> = Utc::now();

    // Step 1: lightweight transaction — IF NOT EXISTS
    let result = session
        .query(
            "INSERT INTO hackmichelin.post_likes (post_id, user_id, username, liked_at) \
             VALUES (?, ?, ?, ?) IF NOT EXISTS",
            (post_id, user_id, username, liked_at),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;

    // The LWT result row has a single [applied] boolean column.
    let applied = result
        .rows
        .and_then(|rows| rows.into_typed::<(bool,)>().next())
        .and_then(|r| r.ok())
        .map(|(b,)| b)
        .unwrap_or(false);

    if !applied {
        return Err(AppError::Conflict);
    }

    // Step 2: increment counter (separate statement — cannot mix with regular writes in a batch)
    session
        .query(
            "UPDATE hackmichelin.post_likes_count SET likes = likes + 1 WHERE post_id = ?",
            (post_id,),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;

    Ok(())
}

/// Check existence, delete from post_likes, then decrement the counter.
pub async fn unlike_post(
    session: &Arc<Session>,
    post_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    // Step 1: verify the like exists
    let result = session
        .query(
            "SELECT user_id FROM hackmichelin.post_likes WHERE post_id = ? AND user_id = ?",
            (post_id, user_id),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;

    let exists = result
        .rows
        .map(|rows| !rows.is_empty())
        .unwrap_or(false);

    if !exists {
        return Err(AppError::NotFound);
    }

    // Step 2: delete the like row
    session
        .query(
            "DELETE FROM hackmichelin.post_likes WHERE post_id = ? AND user_id = ?",
            (post_id, user_id),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;

    // Step 3: decrement counter
    session
        .query(
            "UPDATE hackmichelin.post_likes_count SET likes = likes - 1 WHERE post_id = ?",
            (post_id,),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;

    Ok(())
}

/// Return the like count for a post (0 if no row exists).
pub async fn get_like_count(
    session: &Arc<Session>,
    post_id: Uuid,
) -> Result<LikeCount, AppError> {
    let result = session
        .query(
            "SELECT likes FROM hackmichelin.post_likes_count WHERE post_id = ?",
            (post_id,),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;

    let likes = result
        .rows
        .and_then(|rows| rows.into_typed::<(Counter,)>().next())
        .and_then(|r| r.ok())
        .map(|(c,)| c.0)
        .unwrap_or(0);

    Ok(LikeCount { post_id, likes })
}

/// List users who liked a post, up to `limit` entries.
pub async fn list_likes(
    session: &Arc<Session>,
    post_id: Uuid,
    limit: i32,
) -> Result<Vec<LikeEntry>, AppError> {
    let result = session
        .query(
            "SELECT user_id, username, liked_at FROM hackmichelin.post_likes WHERE post_id = ? LIMIT ?",
            (post_id, limit),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;

    let mut entries = Vec::new();

    if let Some(rows) = result.rows {
        type Row = (Uuid, String, DateTime<Utc>);
        for row in rows.into_typed::<Row>() {
            match row {
                Ok((user_id, username, liked_at)) => {
                    entries.push(LikeEntry {
                        user_id,
                        username,
                        liked_at,
                    });
                }
                Err(e) => {
                    tracing::warn!("row deserialization error: {e}");
                }
            }
        }
    }

    Ok(entries)
}
