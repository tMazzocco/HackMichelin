/// Loaded from environment variables (or .env file).
#[derive(Clone, Debug)]
pub struct Config {
    pub http_addr:  String,
    pub es_host:    String,
    pub mqtt_host:  String,
    pub mqtt_port:  u16,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            http_addr: std::env::var("HTTP_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:3006".into()),
            es_host: std::env::var("ES_HOST")
                .unwrap_or_else(|_| "http://localhost:9200".into()),
            mqtt_host: std::env::var("MQTT_HOST")
                .unwrap_or_else(|_| "localhost".into()),
            mqtt_port: std::env::var("MQTT_PORT")
                .unwrap_or_else(|_| "1883".into())
                .parse()
                .map_err(|_| anyhow::anyhow!("MQTT_PORT must be a valid port number"))?,
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change-me".into()),
        })
    }
}
