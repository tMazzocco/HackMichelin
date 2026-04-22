/// Loaded from environment variables (or .env file).
#[derive(Clone, Debug)]
pub struct Config {
    pub database_url:    String,
    pub http_addr:       String,
    pub cassandra_nodes: String,
    pub jwt_secret:      String,
    pub mqtt_host:       String,
    pub mqtt_port:       u16,
    pub mqtt_client_id:  String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL not set"))?,
            http_addr: std::env::var("HTTP_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:3003".into()),
            cassandra_nodes: std::env::var("CASSANDRA_NODES")
                .unwrap_or_else(|_| "localhost:9042".into()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change-me".into()),
            mqtt_host: std::env::var("MQTT_HOST")
                .unwrap_or_else(|_| "localhost".into()),
            mqtt_port: std::env::var("MQTT_PORT")
                .unwrap_or_else(|_| "1883".into())
                .parse()
                .map_err(|_| anyhow::anyhow!("MQTT_PORT must be a valid port number"))?,
            mqtt_client_id: std::env::var("MQTT_CLIENT_ID")
                .unwrap_or_else(|_| "user-service".into()),
        })
    }
}
