use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/// Insert a new row into the `media` table and return the generated UUID.
#[allow(clippy::too_many_arguments)]
pub async fn insert_media(
    pool: &PgPool,
    user_id: Uuid,
    media_type: &str,
    filename: &str,
    storage_path: &str,
    url: &str,
    thumbnail_url: Option<&str>,
    mime_type: Option<&str>,
    size_bytes: Option<i64>,
) -> Result<Uuid, AppError> {
    let row = sqlx::query!(
        r#"
        INSERT INTO media
            (user_id, media_type, filename, storage_path, url, thumbnail_url, mime_type, size_bytes)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
        user_id,
        media_type,
        filename,
        storage_path,
        url,
        thumbnail_url,
        mime_type,
        size_bytes,
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}
