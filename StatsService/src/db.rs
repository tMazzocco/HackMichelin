use sqlx::PgPool;
use crate::error::AppError;

pub async fn upsert_on_create(pool: &PgPool, restaurant_id: &str, is_good: bool) -> Result<(), AppError> {
    let good = if is_good { 1i32 } else { 0i32 };
    let bad  = if is_good { 0i32 } else { 1i32 };
    sqlx::query(
        "INSERT INTO restaurant_stats (restaurant_id, total_posts, good_posts, bad_posts, last_updated)
         VALUES ($1, 1, $2, $3, NOW())
         ON CONFLICT (restaurant_id) DO UPDATE SET
           total_posts = restaurant_stats.total_posts + 1,
           good_posts  = restaurant_stats.good_posts  + $2,
           bad_posts   = restaurant_stats.bad_posts   + $3,
           last_updated = NOW()"
    ).bind(restaurant_id).bind(good).bind(bad).execute(pool).await?;
    Ok(())
}

pub async fn decrement_on_delete(pool: &PgPool, restaurant_id: &str, was_good: bool) -> Result<(), AppError> {
    let good = if was_good { 1i32 } else { 0i32 };
    let bad  = if was_good { 0i32 } else { 1i32 };
    sqlx::query(
        "UPDATE restaurant_stats SET
           total_posts = GREATEST(0, total_posts - 1),
           good_posts  = GREATEST(0, good_posts  - $2),
           bad_posts   = GREATEST(0, bad_posts   - $3),
           last_updated = NOW()
         WHERE restaurant_id = $1"
    ).bind(restaurant_id).bind(good).bind(bad).execute(pool).await?;
    Ok(())
}

pub async fn get_stats(pool: &PgPool, restaurant_id: &str) -> Result<Option<crate::models::RestaurantStats>, AppError> {
    let row = sqlx::query_as::<_, crate::models::RestaurantStats>(
        "SELECT restaurant_id, total_posts, good_posts, bad_posts,
                COALESCE(good_posts::float / NULLIF(total_posts, 0), 0.0) AS good_pct
         FROM restaurant_stats WHERE restaurant_id = $1"
    ).bind(restaurant_id).fetch_optional(pool).await?;
    Ok(row)
}
