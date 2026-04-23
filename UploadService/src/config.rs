/// Loaded from environment variables (or .env file).
#[derive(Clone, Debug)]
pub struct Config {
    /// PostgreSQL connection URL (required)
    pub database_url: String,
    /// Bind address, e.g. "0.0.0.0:3007"
    pub http_addr: String,
    /// Secret used to verify JWTs (must match LoginService)
    pub jwt_secret: String,
    /// Absolute path to the folder where media files are stored
    pub media_dir: String,
    /// Maximum accepted upload size in bytes (default 200 MB)
    pub max_upload_bytes: u64,
    /// URL prefix prepended to filenames to build the public URL
    pub download_base_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL env var is required"))?,
            http_addr: std::env::var("HTTP_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:3007".into()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change-me".into()),
            media_dir: std::env::var("MEDIA_DIR")
                .unwrap_or_else(|_| "/media".into()),
            max_upload_bytes: std::env::var("MAX_UPLOAD_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(209_715_200u64), // 200 MB
            download_base_url: std::env::var("DOWNLOAD_BASE_URL")
                .unwrap_or_else(|_| "/api/download/files".into()),
        })
    }
}
