use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{StarEntry, UserProfile};

// ── Profile ───────────────────────────────────────────────────────────────────

pub async fn get_profile(pool: &PgPool, user_id: Uuid) -> Result<Option<UserProfile>, AppError> {
    let row = sqlx::query_as::<_, UserProfile>(
        r#"
        SELECT id, username, bio, avatar_url,
               stars_collected, followers_count, following_count, created_at
        FROM   users
        WHERE  id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn update_profile(
    pool: &PgPool,
    user_id: Uuid,
    bio: Option<String>,
    avatar_url: Option<String>,
) -> Result<UserProfile, AppError> {
    let row = sqlx::query_as::<_, UserProfile>(
        r#"
        UPDATE users
        SET    bio        = COALESCE($2, bio),
               avatar_url = COALESCE($3, avatar_url),
               updated_at = NOW()
        WHERE  id = $1
        RETURNING id, username, bio, avatar_url,
                  stars_collected, followers_count, following_count, created_at
        "#,
    )
    .bind(user_id)
    .bind(bio)
    .bind(avatar_url)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(row)
}

// ── Star collections ──────────────────────────────────────────────────────────

pub async fn collect_star(
    pool: &PgPool,
    user_id: Uuid,
    restaurant_id: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        r#"
        INSERT INTO user_star_collections
            (user_id, restaurant_id, collected_at, michelin_award, green_star, distinction_score)
        SELECT $1, r.id, NOW(), r.michelin_award, r.green_star, r.distinction_score
        FROM   restaurants r
        WHERE  r.id = $2
        "#,
    )
    .bind(user_id)
    .bind(restaurant_id)
    .execute(pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => Err(AppError::NotFound),
        Ok(_) => Ok(()),
        Err(sqlx::Error::Database(ref db_err))
            if db_err.code().as_deref() == Some("23505") =>
        {
            Err(AppError::Conflict("already collected".into()))
        }
        Err(e) => Err(AppError::Database(e)),
    }
}

pub async fn uncollect_star(
    pool: &PgPool,
    user_id: Uuid,
    restaurant_id: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "DELETE FROM user_star_collections WHERE user_id = $1 AND restaurant_id = $2",
    )
    .bind(user_id)
    .bind(restaurant_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_stars(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<StarEntry>, AppError> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>, bool, Option<i32>, chrono::DateTime<chrono::Utc>)>(
        r#"
        SELECT sc.restaurant_id,
               r.name            AS restaurant_name,
               sc.michelin_award,
               sc.green_star,
               sc.distinction_score,
               sc.collected_at
        FROM   user_star_collections sc
        JOIN   restaurants           r ON r.id = sc.restaurant_id
        WHERE  sc.user_id = $1
        ORDER  BY sc.collected_at DESC
        LIMIT  $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let entries = rows
        .into_iter()
        .map(|(restaurant_id, restaurant_name, michelin_award, green_star, distinction_score, collected_at)| {
            StarEntry {
                restaurant_id,
                restaurant_name,
                michelin_award,
                green_star,
                distinction_score,
                collected_at,
            }
        })
        .collect();

    Ok(entries)
}

// ── Counter helpers ───────────────────────────────────────────────────────────

pub async fn increment_following(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE users SET following_count = following_count + 1 WHERE id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn decrement_following(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE users SET following_count = GREATEST(0, following_count - 1) WHERE id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn increment_followers(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE users SET followers_count = followers_count + 1 WHERE id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn decrement_followers(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE users SET followers_count = GREATEST(0, followers_count - 1) WHERE id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn increment_stars(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE users SET stars_collected = stars_collected + 1 WHERE id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn decrement_stars(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE users SET stars_collected = GREATEST(0, stars_collected - 1) WHERE id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}
