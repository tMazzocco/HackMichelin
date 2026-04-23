use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use sqlx::PgPool;
use std::path::Path;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::{auth, config::Config, db, error::AppError, models::UploadResponse};

// ── Allowed extensions ────────────────────────────────────────────────────────

const PHOTO_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp"];
const VIDEO_EXTS: &[&str] = &["mp4", "mov", "avi", "mkv", "webm", "ts"];

// ── App state ─────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Derive a basic MIME type string from the file extension.
fn mime_from_ext(ext: &str) -> Option<&'static str> {
    match ext {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "bmp" => Some("image/bmp"),
        "mp4" => Some("video/mp4"),
        "mov" => Some("video/quicktime"),
        "avi" => Some("video/x-msvideo"),
        "mkv" => Some("video/x-matroska"),
        "webm" => Some("video/webm"),
        "ts" => Some("video/mp2t"),
        _ => None,
    }
}

/// Run ffmpeg to extract a single frame from a video file.
/// Returns the thumbnail path on success, or `None` if ffmpeg is unavailable
/// or returns a non-zero exit code.
async fn extract_video_thumbnail(video_path: &str, thumb_path: &str) -> Option<()> {
    let status = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            video_path,
            "-ss",
            "00:00:01.000",
            "-vframes",
            "1",
            thumb_path,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await;

    match status {
        Ok(s) if s.success() => Some(()),
        Ok(s) => {
            tracing::warn!("ffmpeg exited with status {s} for {video_path}");
            None
        }
        Err(e) => {
            tracing::warn!("ffmpeg not available or failed to spawn: {e}");
            None
        }
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn health() -> &'static str {
    "ok"
}

/// POST /
///
/// Accepts a multipart/form-data body with:
///   - `file`      (required) — the media file to upload
///   - `thumbnail` (optional) — a custom thumbnail image (video only)
async fn upload(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::Claims>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    // Parse user_id from JWT subject
    let user_id: Uuid = claims.sub.parse().map_err(|_| {
        AppError::BadRequest("JWT sub is not a valid UUID".into())
    })?;

    // ── Collect multipart fields ──────────────────────────────────────────────
    let mut file_bytes: Option<bytes::Bytes> = None;
    let mut file_name_hint: Option<String> = None;
    let mut thumbnail_bytes: Option<bytes::Bytes> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("multipart read error: {e}"))
    })? {
        let field_name = field.name().unwrap_or("").to_string();
        let original_filename = field
            .file_name()
            .map(|s| s.to_string());

        let data = field.bytes().await.map_err(|e| {
            AppError::BadRequest(format!("failed to read field bytes: {e}"))
        })?;

        match field_name.as_str() {
            "file" => {
                if data.len() as u64 > state.config.max_upload_bytes {
                    return Err(AppError::BadRequest(format!(
                        "file exceeds maximum allowed size of {} bytes",
                        state.config.max_upload_bytes
                    )));
                }
                file_name_hint = original_filename;
                file_bytes = Some(data);
            }
            "thumbnail" => {
                thumbnail_bytes = Some(data);
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    // ── Validate required field ───────────────────────────────────────────────
    let file_bytes = file_bytes.ok_or_else(|| {
        AppError::BadRequest("missing required multipart field 'file'".into())
    })?;

    // ── Determine extension ───────────────────────────────────────────────────
    let ext = file_name_hint
        .as_deref()
        .and_then(|name| Path::new(name).extension())
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or_else(|| AppError::BadRequest("could not determine file extension".into()))?;

    // ── Classify media type ───────────────────────────────────────────────────
    let media_type = if PHOTO_EXTS.contains(&ext.as_str()) {
        "photo"
    } else if VIDEO_EXTS.contains(&ext.as_str()) {
        "video"
    } else {
        return Err(AppError::BadRequest(format!(
            "unsupported file extension: .{ext}"
        )));
    };

    // ── Generate filenames & paths ────────────────────────────────────────────
    let file_uuid = Uuid::new_v4();
    let filename = format!("{file_uuid}.{ext}");
    let storage_path = format!("{}/{}", state.config.media_dir, filename);
    let url = format!("{}/{}", state.config.download_base_url, filename);
    let mime_type = mime_from_ext(&ext);
    let size_bytes = file_bytes.len() as i64;

    // ── Write main file to disk ───────────────────────────────────────────────
    tokio::fs::write(&storage_path, &file_bytes).await.map_err(|e| {
        AppError::Internal(format!("failed to write file to {storage_path}: {e}"))
    })?;

    // ── Handle thumbnail (video only) ─────────────────────────────────────────
    let thumbnail_url: Option<String> = if media_type == "video" {
        let thumb_filename = format!("{file_uuid}_thumb.jpg");
        let thumb_path = format!("{}/{}", state.config.media_dir, thumb_filename);

        let thumb_saved = if let Some(thumb_data) = thumbnail_bytes {
            // Custom thumbnail provided — save it directly
            match tokio::fs::write(&thumb_path, &thumb_data).await {
                Ok(_) => true,
                Err(e) => {
                    tracing::warn!("Failed to write custom thumbnail: {e}");
                    false
                }
            }
        } else {
            // Auto-generate via ffmpeg
            extract_video_thumbnail(&storage_path, &thumb_path).await.is_some()
        };

        if thumb_saved {
            Some(format!(
                "{}/{}",
                state.config.download_base_url, thumb_filename
            ))
        } else {
            None
        }
    } else {
        // Photos never have a thumbnail_url
        None
    };

    // ── Insert into database ──────────────────────────────────────────────────
    let media_id = db::insert_media(
        &state.pool,
        user_id,
        media_type,
        &filename,
        &storage_path,
        &url,
        thumbnail_url.as_deref(),
        mime_type,
        Some(size_bytes),
    )
    .await?;

    // ── Respond ───────────────────────────────────────────────────────────────
    let response = UploadResponse {
        media_id,
        url,
        thumbnail_url,
        media_type: media_type.to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Protected upload route
    let protected = Router::new()
        .route("/", post(upload))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));

    Router::new()
        .route("/health", get(health))
        .merge(protected)
        .layer(cors)
        .with_state(state)
}
