use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::AppError, models::UserRow};

// ── Users ─────────────────────────────────────────────────

/// Insert a new user and return the created row.
/// Maps PostgreSQL unique constraint violation (23505) to AppError::Conflict.
pub async fn create_user(
    pool:          &PgPool,
    username:      &str,
    email:         &str,
    password_hash: &str,
) -> Result<UserRow, AppError> {
    let row = sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO users (id, username, email, password_hash)
        VALUES (gen_random_uuid(), $1, $2, $3)
        RETURNING id, username, email, password_hash
        "#,
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        // Promote unique-violation to Conflict before the generic From<sqlx::Error> runs
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.code().as_deref() == Some("23505") {
                return AppError::Conflict(db_err.message().to_string());
            }
        }
        AppError::Database(e)
    })?;

    Ok(row)
}

/// Look up a user by email address.
pub async fn find_by_email(
    pool:  &PgPool,
    email: &str,
) -> Result<Option<UserRow>, AppError> {
    let row = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, username, email, password_hash
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

// ── Refresh tokens ────────────────────────────────────────

/// Store a hashed refresh token tied to a user with an expiry.
pub async fn store_refresh_token(
    pool:         &PgPool,
    token_hash:   &str,
    user_id:      Uuid,
    expires_days: u64,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, expires_at)
        VALUES ($1, $2, now() + ($3 || ' days')::interval)
        "#,
    )
    .bind(token_hash)
    .bind(user_id)
    .bind(expires_days as i64)
    .execute(pool)
    .await?;

    Ok(())
}

/// Retrieve (user_id, username) for a valid (non-expired) refresh token hash.
pub async fn get_refresh_token(
    pool:       &PgPool,
    token_hash: &str,
) -> Result<Option<(Uuid, String)>, AppError> {
    let row: Option<(Uuid, String)> = sqlx::query_as(
        r#"
        SELECT rt.user_id, u.username
        FROM refresh_tokens rt
        JOIN users u ON u.id = rt.user_id
        WHERE rt.token_hash = $1
          AND rt.expires_at > now()
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Delete a refresh token by its hash (logout).
pub async fn delete_refresh_token(
    pool:       &PgPool,
    token_hash: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        DELETE FROM refresh_tokens
        WHERE token_hash = $1
        "#,
    )
    .bind(token_hash)
    .execute(pool)
    .await?;

    Ok(())
}
