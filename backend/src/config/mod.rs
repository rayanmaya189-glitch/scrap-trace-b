use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub host: String,
    pub database_url: String,
    pub redis_url: String,
    pub nats_url: String,
    pub jwt_secret: String,
    pub minio_endpoint: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub minio_bucket: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?;

        Ok(Self {
            port: config.get("PORT").unwrap_or(8080),
            host: config.get("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            database_url: config.get("DATABASE_URL")?,
            redis_url: config.get("REDIS_URL")?,
            nats_url: config.get("NATS_URL")?,
            jwt_secret: config.get("JWT_SECRET")?,
            minio_endpoint: config.get("MINIO_ENDPOINT")?,
            minio_access_key: config.get("MINIO_ACCESS_KEY")?,
            minio_secret_key: config.get("MINIO_SECRET_KEY")?,
            minio_bucket: config.get("MINIO_BUCKET").unwrap_or_else(|_| "btrace".to_string()),
        })
    }
}
