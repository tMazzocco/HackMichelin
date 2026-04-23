/// Loaded from environment variables (or .env file).
#[derive(Clone, Debug)]
pub struct Config {
    /// Bind address, e.g. "0.0.0.0:3001"
    pub http_addr: String,
    /// Absolute path to the folder containing media files
    pub media_dir: String,
    /// URL prefix used in .m3u8 segment entries (default "files")
    pub files_base_url: String,
    /// Duration (seconds) assigned to each photo segment in the playlist
    pub photo_duration: f32,
    /// Default duration (seconds) assigned to each video segment when unknown
    pub video_duration: f32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            http_addr: std::env::var("HTTP_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:3001".into()),
            media_dir: std::env::var("MEDIA_DIR")
                .unwrap_or_else(|_| "/media".into()),
            files_base_url: std::env::var("FILES_BASE_URL")
                .unwrap_or_else(|_| "files".into()),
            photo_duration: std::env::var("PHOTO_DURATION")
                .unwrap_or_else(|_| "5".into())
                .parse()
                .unwrap_or(5.0),
            video_duration: std::env::var("VIDEO_DURATION")
                .unwrap_or_else(|_| "10".into())
                .parse()
                .unwrap_or(10.0),
        })
    }
}
