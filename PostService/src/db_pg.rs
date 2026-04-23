use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppError;

pub async fn get_media(
    pool: &PgPool,
    media_id: Uuid,
    user_id: Uuid,
) -> Result<Option<(String, String, Option<String>)>, AppError> {
    let row = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT media_type, url, thumbnail_url FROM media WHERE id = $1 AND user_id = $2"
    ).bind(media_id).bind(user_id).fetch_optional(pool).await?;
    Ok(row)
}

pub async fn get_restaurant_name(pool: &PgPool, restaurant_id: &str) -> Result<Option<String>, AppError> {
    let row = sqlx::query_scalar::<_, String>(
        "SELECT name FROM restaurants WHERE id = $1"
    ).bind(restaurant_id).fetch_optional(pool).await?;
    Ok(row)
}
