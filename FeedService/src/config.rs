#[derive(Clone, Debug)]
pub struct Config {
    pub http_addr: String,
    pub cassandra_nodes: String,
    pub jwt_secret: String,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_client_id: String,
}
impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            http_addr: std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:3005".into()),
            cassandra_nodes: std::env::var("CASSANDRA_NODES").unwrap_or_else(|_| "localhost:9042".into()),
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "change-me".into()),
            mqtt_host: std::env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".into()),
            mqtt_port: std::env::var("MQTT_PORT").unwrap_or_else(|_| "1883".into()).parse().unwrap_or(1883),
            mqtt_client_id: std::env::var("MQTT_CLIENT_ID").unwrap_or_else(|_| "feed-service".into()),
        })
    }
}
