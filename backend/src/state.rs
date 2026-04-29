//! Application state management

use sqlx::postgres::PgPool;
use redis::aio::ConnectionManager;
use crate::config::Config;
use crate::nats::NatsConnection;
use crate::error::Result;
use std::sync::Arc;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db_pool: PgPool,
    pub redis: ConnectionManager,
    pub nats: Arc<NatsConnection>,
}

impl AppState {
    /// Initialize application state with all connections
    pub async fn new(config: &Config) -> Result<Self> {
        // Create database connection pool
        let db_pool = PgPool::connect(&config.database_url).await?;
        
        // Create Redis connection manager
        let redis_client = redis::Client::open(config.redis_url.as_str())?;
        let redis = ConnectionManager::new(redis_client).await?;
        
        // Connect to NATS
        let nats = NatsConnection::connect(config).await?;
        
        // Ensure JetStream stream exists
        nats.ensure_stream().await?;
        
        Ok(Self {
            config: Arc::new(config.clone()),
            db_pool,
            redis,
            nats: Arc::new(nats),
        })
    }
}
