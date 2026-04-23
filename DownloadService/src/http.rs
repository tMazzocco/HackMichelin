use axum::{
    extract::{Query, State},
    http::header,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::path::Path;
use tower_http::services::ServeDir;

use crate::{config::Config, error::AppError};

// ── Constants ─────────────────────────────────────────────────────────────────

const PHOTO_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp"];
const VIDEO_EXTS: &[&str] = &["mp4", "ts", "m4s", "mov", "avi", "mkv", "webm"];

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
}

// ── Query params ──────────────────────────────────────────────────────────────

/// Optional comma-separated list of file names to include.
/// If absent, all media files in MEDIA_DIR are returned.
#[derive(Deserialize)]
pub struct PlaylistParams {
    pub files: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn health() -> &'static str {
    "ok"
}

/// GET /playlist.m3u8[?files=name1.jpg,name2.mp4,...]
///
/// Returns an HLS playlist (application/vnd.apple.mpegurl) whose segments
/// point to GET /files/<name>.  Photos get `PHOTO_DURATION` seconds each;
/// videos get `VIDEO_DURATION` seconds each.
async fn playlist(
    State(state): State<AppState>,
    Query(params): Query<PlaylistParams>,
) -> Result<Response, AppError> {
    let media_dir = &state.config.media_dir;

    // Collect candidate file names ─────────────────────────────────────────
    let candidates: Vec<String> = match params.files {
        Some(ref list) => list
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        None => {
            // Scan the media directory
            let mut rd = tokio::fs::read_dir(media_dir).await?;
            let mut names = Vec::new();
            while let Some(entry) = rd.next_entry().await? {
                if entry.file_type().await?.is_file() {
                    names.push(entry.file_name().to_string_lossy().into_owned());
                }
            }
            names.sort();
            names
        }
    };

    // Validate, classify, and filter ──────────────────────────────────────
    let mut segments: Vec<(String, f32)> = Vec::new();

    for name in candidates {
        // Security: reject any path-traversal attempt
        if name.contains('/') || name.contains('\\') || name.contains("..") {
            return Err(AppError::BadRequest(format!(
                "invalid filename: {name}"
            )));
        }

        let ext = Path::new(&name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let duration = if PHOTO_EXTS.contains(&ext.as_str()) {
            state.config.photo_duration
        } else if VIDEO_EXTS.contains(&ext.as_str()) {
            state.config.video_duration
        } else {
            // Skip unknown extensions silently
            continue;
        };

        // Confirm the file is actually present in the media dir
        let full = Path::new(media_dir).join(&name);
        if !tokio::fs::try_exists(&full).await.unwrap_or(false) {
            continue;
        }

        segments.push((name, duration));
    }

    if segments.is_empty() {
        return Err(AppError::NotFound);
    }

    // Build .m3u8 ─────────────────────────────────────────────────────────
    let target_duration = segments
        .iter()
        .map(|(_, d)| *d)
        .fold(0.0f32, f32::max)
        .ceil() as u32;

    let base = &state.config.files_base_url;
    let mut m3u8 = String::with_capacity(256 + segments.len() * 80);

    m3u8.push_str("#EXTM3U\n");
    m3u8.push_str("#EXT-X-VERSION:3\n");
    m3u8.push_str(&format!("#EXT-X-TARGETDURATION:{target_duration}\n"));
    m3u8.push_str("#EXT-X-MEDIA-SEQUENCE:0\n");
    m3u8.push_str("#EXT-X-PLAYLIST-TYPE:VOD\n");

    for (name, duration) in &segments {
        let title = Path::new(name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(name.as_str());
        m3u8.push_str(&format!("#EXTINF:{duration:.3},{title}\n"));
        m3u8.push_str(&format!("{base}/{name}\n"));
    }

    m3u8.push_str("#EXT-X-ENDLIST\n");

    Ok((
        [(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")],
        m3u8,
    )
        .into_response())
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router(state: AppState) -> Router {
    let media_dir = state.config.media_dir.clone();
    Router::new()
        .route("/health", get(health))
        .route("/playlist.m3u8", get(playlist))
        // Serves GET /files/<name> with full Range-request support (needed for video)
        .nest_service("/files", ServeDir::new(media_dir))
        .with_state(state)
}
