/// Loaded from environment variables (or .env file).
#[derive(Clone, Debug)]
pub struct Config {
    /// Comma-separated list of Cassandra/Scylla nodes, e.g. "127.0.0.1:9042"
    pub cassandra_nodes: Vec<String>,
    pub http_addr: String,
    pub jwt_secret: String,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_client_id: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let cassandra_raw = std::env::var("CASSANDRA_NODES")
            .unwrap_or_else(|_| "localhost:9042".into());
        let cassandra_nodes = cassandra_raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            cassandra_nodes,
            http_addr: std::env::var("HTTP_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:3005".into()),
            jwt_secret: std::env::var("JWT_SECRET")
                .map_err(|_| anyhow::anyhow!("JWT_SECRET not set"))?,
            mqtt_host: std::env::var("MQTT_HOST")
                .unwrap_or_else(|_| "localhost".into()),
            mqtt_port: std::env::var("MQTT_PORT")
                .unwrap_or_else(|_| "1883".into())
                .parse()
                .map_err(|_| anyhow::anyhow!("MQTT_PORT must be a valid port number"))?,
            mqtt_client_id: std::env::var("MQTT_CLIENT_ID")
                .unwrap_or_else(|_| "feed-service".into()),
        })
    }
}
