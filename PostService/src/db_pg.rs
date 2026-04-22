use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/// Fetch media metadata from PostgreSQL, validating ownership.
///
/// Returns `(media_type, url, thumbnail_url)` when the record exists and
/// belongs to `user_id`, or `None` when not found.
pub async fn get_media(
    pool: &PgPool,
    media_id: Uuid,
    user_id: Uuid,
) -> Result<Option<(String, String, Option<String>)>, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT media_type, url, thumbnail_url
        FROM media
        WHERE id = $1 AND user_id = $2
        "#,
        media_id,
        user_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| (r.media_type, r.url, r.thumbnail_url)))
}

/// Fetch a restaurant's display name from PostgreSQL.
///
/// Returns `None` when the restaurant does not exist.
pub async fn get_restaurant_name(
    pool: &PgPool,
    restaurant_id: &str,
) -> Result<Option<String>, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT name
        FROM restaurants
        WHERE id = $1
        "#,
        restaurant_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.name))
}
