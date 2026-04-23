#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub http_addr: String,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_client_id: String,
}
impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL not set"))?,
            http_addr: std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:3010".into()),
            mqtt_host: std::env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".into()),
            mqtt_port: std::env::var("MQTT_PORT").unwrap_or_else(|_| "1883".into()).parse().unwrap_or(1883),
            mqtt_client_id: std::env::var("MQTT_CLIENT_ID").unwrap_or_else(|_| "stats-service".into()),
        })
    }
}
