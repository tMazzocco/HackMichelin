use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use crate::error::AppError;

pub async fn follow_user(
    session: &Arc<scylla::Session>,
    follower_id: Uuid, followed_id: Uuid,
    follower_name: &str, followed_name: &str,
) -> Result<(), AppError> {
    let now = Utc::now();
    session.query(
        "INSERT INTO user_following (follower_id, followed_id, followed_name, followed_at) VALUES (?, ?, ?, ?)",
        (follower_id, followed_id, followed_name, now),
    ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;
    session.query(
        "INSERT INTO user_followers (followed_id, follower_id, follower_name, followed_at) VALUES (?, ?, ?, ?)",
        (followed_id, follower_id, follower_name, now),
    ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;
    Ok(())
}

pub async fn unfollow_user(
    session: &Arc<scylla::Session>,
    follower_id: Uuid, followed_id: Uuid,
) -> Result<(), AppError> {
    session.query(
        "DELETE FROM user_following WHERE follower_id = ? AND followed_id = ?",
        (follower_id, followed_id),
    ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;
    session.query(
        "DELETE FROM user_followers WHERE followed_id = ? AND follower_id = ?",
        (followed_id, follower_id),
    ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;
    Ok(())
}

pub async fn is_following(
    session: &Arc<scylla::Session>,
    follower_id: Uuid, followed_id: Uuid,
) -> Result<bool, AppError> {
    let result = session.query(
        "SELECT followed_id FROM user_following WHERE follower_id = ? AND followed_id = ?",
        (follower_id, followed_id),
    ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;
    Ok(result.rows.map(|r| !r.is_empty()).unwrap_or(false))
}

pub async fn list_following(
    session: &Arc<scylla::Session>,
    user_id: Uuid,
) -> Result<Vec<(Uuid, String)>, AppError> {
    let result = session.query(
        "SELECT followed_id, followed_name FROM user_following WHERE follower_id = ?",
        (user_id,),
    ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;
    let mut out = vec![];
    if let Some(rows) = result.rows {
        for row in rows.into_typed::<(Uuid, String)>() {
            if let Ok(r) = row { out.push(r); }
        }
    }
    Ok(out)
}

pub async fn list_followers(
    session: &Arc<scylla::Session>,
    user_id: Uuid,
) -> Result<Vec<(Uuid, String)>, AppError> {
    let result = session.query(
        "SELECT follower_id, follower_name FROM user_followers WHERE followed_id = ?",
        (user_id,),
    ).await.map_err(|e| AppError::Cassandra(e.to_string()))?;
    let mut out = vec![];
    if let Some(rows) = result.rows {
        for row in rows.into_typed::<(Uuid, String)>() {
            if let Ok(r) = row { out.push(r); }
        }
    }
    Ok(out)
}
