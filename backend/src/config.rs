//! Application configuration management

use anyhow::Result;
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct Config {
    pub server_addr: SocketAddr,
    pub nats_url: String,
    pub nats_stream: String,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: Vec<u8>,
    pub minio_endpoint: String,
    pub minio_bucket: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();

        let config = Self {
            server_addr: std::env::var("SERVER_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
                .parse()?,
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            nats_stream: std::env::var("NATS_STREAM")
                .unwrap_or_else(|_| "BTRACE".to_string()),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/btrace".to_string()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "btrace-dev-secret-key-change-in-production".to_string())
                .into_bytes(),
            minio_endpoint: std::env::var("MINIO_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".to_string()),
            minio_bucket: std::env::var("MINIO_BUCKET")
                .unwrap_or_else(|_| "btrace-storage".to_string()),
        };

        Ok(config)
    }
}
