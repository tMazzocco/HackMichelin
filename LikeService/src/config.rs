use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub http_addr: String,
    pub cassandra_nodes: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            http_addr: env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:3009".to_string()),
            cassandra_nodes: env::var("CASSANDRA_NODES")
                .unwrap_or_else(|_| "localhost:9042".to_string()),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
        }
    }
}
