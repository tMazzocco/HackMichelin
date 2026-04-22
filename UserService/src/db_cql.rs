use std::sync::Arc;

use scylla::Session;
use uuid::Uuid;

use crate::error::AppError;

// ── Follow / Unfollow ─────────────────────────────────────────────────────────

pub async fn follow_user(
    session: &Arc<Session>,
    follower_id: Uuid,
    followed_id: Uuid,
    follower_name: &str,
    followed_name: &str,
) -> Result<(), AppError> {
    // Insert into user_following (the follower's perspective)
    session
        .query(
            "INSERT INTO user_following (follower_id, followed_id, followed_name, followed_at) \
             VALUES (?, ?, ?, toTimestamp(now()))",
            (follower_id, followed_id, followed_name),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;

    // Insert into user_followers (the followed user's perspective)
    session
        .query(
            "INSERT INTO user_followers (followed_id, follower_id, follower_name, followed_at) \
             VALUES (?, ?, ?, toTimestamp(now()))",
            (followed_id, follower_id, follower_name),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;

    Ok(())
}

pub async fn unfollow_user(
    session: &Arc<Session>,
    follower_id: Uuid,
    followed_id: Uuid,
) -> Result<(), AppError> {
    session
        .query(
            "DELETE FROM user_following WHERE follower_id = ? AND followed_id = ?",
            (follower_id, followed_id),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;

    session
        .query(
            "DELETE FROM user_followers WHERE followed_id = ? AND follower_id = ?",
            (followed_id, follower_id),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;

    Ok(())
}

pub async fn is_following(
    session: &Arc<Session>,
    follower_id: Uuid,
    followed_id: Uuid,
) -> Result<bool, AppError> {
    let result = session
        .query(
            "SELECT followed_id FROM user_following \
             WHERE follower_id = ? AND followed_id = ?",
            (follower_id, followed_id),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;

    Ok(result.rows_num().unwrap_or(0) > 0)
}

// ── List following / followers ────────────────────────────────────────────────

pub async fn list_following(
    session: &Arc<Session>,
    user_id: Uuid,
) -> Result<Vec<(Uuid, String)>, AppError> {
    let result = session
        .query(
            "SELECT followed_id, followed_name FROM user_following WHERE follower_id = ?",
            (user_id,),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;

    let mut out = Vec::new();
    if let Some(rows) = result.rows {
        for row in rows {
            let (id, name): (Uuid, String) = row
                .into_typed::<(Uuid, String)>()
                .map_err(|e| AppError::Cassandra(e.to_string()))?;
            out.push((id, name));
        }
    }
    Ok(out)
}

pub async fn list_followers(
    session: &Arc<Session>,
    user_id: Uuid,
) -> Result<Vec<(Uuid, String)>, AppError> {
    let result = session
        .query(
            "SELECT follower_id, follower_name FROM user_followers WHERE followed_id = ?",
            (user_id,),
        )
        .await
        .map_err(|e| AppError::Cassandra(e.to_string()))?;

    let mut out = Vec::new();
    if let Some(rows) = result.rows {
        for row in rows {
            let (id, name): (Uuid, String) = row
                .into_typed::<(Uuid, String)>()
                .map_err(|e| AppError::Cassandra(e.to_string()))?;
            out.push((id, name));
        }
    }
    Ok(out)
}
