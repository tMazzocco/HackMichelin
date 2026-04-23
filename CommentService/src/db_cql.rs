use std::sync::Arc;

use chrono::{DateTime, Utc};
use scylla::IntoTypedRows;
use uuid::Uuid;

use crate::{error::AppError, models::Comment};

/// INSERT INTO post_comments (post_id, comment_id, created_at, user_id, username, body)
/// VALUES (?, ?, ?, ?, ?, ?)
pub async fn insert_comment(
    session: &Arc<scylla::Session>,
    post_id: Uuid,
    comment_id: Uuid,
    user_id: Uuid,
    username: &str,
    body: &str,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    session
        .query(
            "INSERT INTO post_comments \
             (post_id, comment_id, created_at, user_id, username, body) \
             VALUES (?, ?, ?, ?, ?, ?)",
            (post_id, comment_id, now, user_id, username, body),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;
    Ok(())
}

/// SELECT comment_id, post_id, user_id, username, body, created_at
/// FROM post_comments
/// WHERE post_id = ?
/// [AND created_at > ?]
/// ORDER BY created_at ASC, comment_id ASC
/// LIMIT ?
pub async fn get_comments(
    session: &Arc<scylla::Session>,
    post_id: Uuid,
    after: Option<DateTime<Utc>>,
    limit: i32,
) -> Result<Vec<Comment>, AppError> {
    let rows = if let Some(after_ts) = after {
        session
            .query(
                "SELECT comment_id, post_id, user_id, username, body, created_at \
                 FROM post_comments \
                 WHERE post_id = ? AND created_at > ? \
                 ORDER BY created_at ASC, comment_id ASC \
                 LIMIT ?",
                (post_id, after_ts, limit),
            )
            .await
            .map_err(|e| AppError::Cassandra(e.to_string()))?
            .rows
    } else {
        session
            .query(
                "SELECT comment_id, post_id, user_id, username, body, created_at \
                 FROM post_comments \
                 WHERE post_id = ? \
                 ORDER BY created_at ASC, comment_id ASC \
                 LIMIT ?",
                (post_id, limit),
            )
            .await
            .map_err(|e| AppError::Cassandra(e.to_string()))?
            .rows
    };

    let mut out = Vec::new();
    if let Some(rows) = rows {
        type Row = (Uuid, Uuid, Uuid, String, String, DateTime<Utc>);
        for row in rows.into_typed::<Row>() {
            match row {
                Ok((cid, pid, uid, uname, body, created_at)) => {
                    out.push(Comment {
                        comment_id: cid,
                        post_id: pid,
                        user_id: uid,
                        username: uname,
                        body,
                        created_at,
                    });
                }
                Err(e) => {
                    tracing::warn!("row deserialization error: {e}");
                }
            }
        }
    }
    Ok(out)
}

/// SELECT a single comment by its full primary key: (post_id, created_at, comment_id)
pub async fn get_comment(
    session: &Arc<scylla::Session>,
    post_id: Uuid,
    comment_id: Uuid,
    created_at: DateTime<Utc>,
) -> Result<Option<Comment>, AppError> {
    let result = session
        .query(
            "SELECT comment_id, post_id, user_id, username, body, created_at \
             FROM post_comments \
             WHERE post_id = ? AND created_at = ? AND comment_id = ?",
            (post_id, created_at, comment_id),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;

    if let Some(rows) = result.rows {
        type Row = (Uuid, Uuid, Uuid, String, String, DateTime<Utc>);
        if let Some(Ok((cid, pid, uid, uname, body, cat))) = rows.into_typed::<Row>().next() {
            return Ok(Some(Comment {
                comment_id: cid,
                post_id: pid,
                user_id: uid,
                username: uname,
                body,
                created_at: cat,
            }));
        }
    }
    Ok(None)
}

/// DELETE FROM post_comments WHERE post_id = ? AND created_at = ? AND comment_id = ?
pub async fn delete_comment(
    session: &Arc<scylla::Session>,
    post_id: Uuid,
    comment_id: Uuid,
    created_at: DateTime<Utc>,
) -> Result<(), AppError> {
    session
        .query(
            "DELETE FROM post_comments \
             WHERE post_id = ? AND created_at = ? AND comment_id = ?",
            (post_id, created_at, comment_id),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;
    Ok(())
}
