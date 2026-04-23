use sqlx::PgPool;
use uuid::Uuid;
use crate::{error::AppError, models::{StarEntry, UserProfile}};

pub async fn get_profile(pool: &PgPool, user_id: Uuid) -> Result<Option<UserProfile>, AppError> {
    let row = sqlx::query_as::<_, UserProfile>(
        "SELECT id, username, bio, avatar_url, stars_collected, followers_count, following_count, created_at FROM users WHERE id = $1"
    ).bind(user_id).fetch_optional(pool).await?;
    Ok(row)
}

pub async fn update_profile(pool: &PgPool, user_id: Uuid, bio: Option<String>, avatar_url: Option<String>) -> Result<UserProfile, AppError> {
    let row = sqlx::query_as::<_, UserProfile>(
        "UPDATE users SET bio = COALESCE($2, bio), avatar_url = COALESCE($3, avatar_url), updated_at = NOW() WHERE id = $1 RETURNING id, username, bio, avatar_url, stars_collected, followers_count, following_count, created_at"
    ).bind(user_id).bind(bio).bind(avatar_url).fetch_one(pool).await?;
    Ok(row)
}

pub async fn collect_star(pool: &PgPool, user_id: Uuid, restaurant_id: &str) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO user_star_collections (user_id, restaurant_id, collected_at, michelin_award, green_star, distinction_score)
         SELECT $1, r.id, NOW(), r.michelin_award, r.green_star, r.distinction_score
         FROM restaurants r WHERE r.id = $2"
    ).bind(user_id).bind(restaurant_id).execute(pool).await
    .map_err(|e| match e {
        sqlx::Error::Database(ref dbe) if dbe.code().as_deref() == Some("23505") =>
            AppError::Conflict("already collected".into()),
        other => AppError::Database(other),
    })?;
    Ok(())
}

pub async fn uncollect_star(pool: &PgPool, user_id: Uuid, restaurant_id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM user_star_collections WHERE user_id = $1 AND restaurant_id = $2")
        .bind(user_id).bind(restaurant_id).execute(pool).await?;
    Ok(())
}

pub async fn list_stars(pool: &PgPool, user_id: Uuid, limit: i64, offset: i64) -> Result<Vec<StarEntry>, AppError> {
    let rows = sqlx::query_as::<_, StarEntry>(
        "SELECT sc.restaurant_id, r.name AS restaurant_name, sc.michelin_award, sc.green_star, sc.distinction_score, sc.collected_at
         FROM user_star_collections sc JOIN restaurants r ON r.id = sc.restaurant_id
         WHERE sc.user_id = $1 ORDER BY sc.collected_at DESC LIMIT $2 OFFSET $3"
    ).bind(user_id).bind(limit).bind(offset).fetch_all(pool).await?;
    Ok(rows)
}

pub async fn adjust_counter(pool: &PgPool, user_id: Uuid, column: &str, delta: i32) -> Result<(), AppError> {
    let sql = match (column, delta > 0) {
        ("stars_collected", true)  => "UPDATE users SET stars_collected = stars_collected + 1 WHERE id = $1",
        ("stars_collected", false) => "UPDATE users SET stars_collected = GREATEST(0, stars_collected - 1) WHERE id = $1",
        ("followers_count", true)  => "UPDATE users SET followers_count = followers_count + 1 WHERE id = $1",
        ("followers_count", false) => "UPDATE users SET followers_count = GREATEST(0, followers_count - 1) WHERE id = $1",
        ("following_count", true)  => "UPDATE users SET following_count = following_count + 1 WHERE id = $1",
        ("following_count", false) => "UPDATE users SET following_count = GREATEST(0, following_count - 1) WHERE id = $1",
        _ => return Ok(()),
    };
    sqlx::query(sql).bind(user_id).execute(pool).await?;
    Ok(())
}
