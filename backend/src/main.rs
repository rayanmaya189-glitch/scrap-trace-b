mod api;
mod auth;
mod config;
mod consumers;
mod core;
mod db;
mod models;
mod nats;
mod repositories;
mod services;
mod utils;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::handlers::auth_handler::AuthState as AuthHandlerState;
use crate::api::routes::{audit_routes, auth_routes, consent_routes, handshake_routes, material_routes, report_routes, scoring_routes, supplier_routes, upload_routes};
use crate::auth::JwtManager;
use crate::config::AppConfig;
use crate::consumers::EventConsumer;
use crate::db::pool::{create_pool, run_migrations};
use crate::nats::NatsManager;
use crate::repositories::{material_repository::MaterialRepository, scoring_repository::ScoringRepository, supplier_repository::SupplierRepository};
use crate::services::{RedisManager, MinioManager};

#[derive(Clone)]
pub struct AppState {
    pub material_repo: MaterialRepository,
    pub scoring_repo: ScoringRepository,
    pub supplier_repo: SupplierRepository,
    pub nats_manager: Arc<NatsManager>,
    pub redis_manager: Arc<RedisManager>,
    pub minio_manager: Arc<MinioManager>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,btrace=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env().expect("Failed to load configuration");

    // Initialize database pool
    let pool = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&config.database_url)
        .await?;
    
    run_migrations(&pool).await?;
    tracing::info!("✅ Database migrations completed");

    // Initialize NATS JetStream
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let nats_manager = Arc::new(NatsManager::new(&nats_url).await?);
    nats_manager.init_streams().await?;
    tracing::info!("✅ NATS JetStream initialized");

    // Initialize Redis
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let redis_manager = RedisManager::new(&redis_url)?;
    tracing::info!("✅ Redis connection initialized");

    // Initialize JWT Manager
    let jwt_manager = JwtManager::new()?;
    tracing::info!("✅ JWT manager initialized");

    // Initialize MinIO
    let minio_manager = Arc::new(MinioManager::new(
        &config.minio_endpoint,
        &config.minio_access_key,
        &config.minio_secret_key,
        &config.minio_bucket,
    )?);
    tracing::info!("✅ MinIO storage initialized");

    // Initialize repositories
    let supplier_repo = SupplierRepository::new(pool.clone());
    let material_repo = MaterialRepository::new(pool.clone());
    let scoring_repo = ScoringRepository::new(pool.clone());

    let app_state = AppState {
        material_repo: material_repo.clone(),
        scoring_repo: scoring_repo.clone(),
        supplier_repo: supplier_repo.clone(),
        nats_manager: nats_manager.clone(),
        redis_manager: redis_manager.clone(),
        minio_manager: minio_manager.clone(),
    };

    // Auth state for auth routes
    let auth_state = AuthHandlerState {
        redis_manager: redis_manager.clone(),
        jwt_manager: jwt_manager.clone(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/v1/auth", auth_routes::router(auth_state))
        .nest("/v1/suppliers", supplier_routes::router().with_state(supplier_repo.clone()))
        .nest("/v1/materials", material_routes::router().with_state((material_repo.clone(), supplier_repo.clone())))
        .nest("/v1/scores", scoring_routes::router().with_state((scoring_repo.clone(), supplier_repo.clone())))
        .nest("/v1/handshakes", handshake_routes::router().with_state(app_state.clone()))
        .nest("/v1/consent", consent_routes::router(pool.clone(), supplier_repo.clone()))
        .nest("/v1/reports", report_routes::router(pool.clone()))
        .nest("/v1/upload", upload_routes::router().with_state(minio_manager.clone()))
        .nest("/v1/audit", audit_routes::router(pool.clone()))
        .route("/health", get(health_check))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state.clone());

    // Start event consumer in background
    let consumer = EventConsumer::new(nats_manager.clone(), pool.clone());
    tokio::spawn(async move {
        if let Err(e) = consumer.start().await {
            tracing::error!("❌ Event consumer failed: {}", e);
        }
    });
    tracing::info!("✅ Event consumer started");

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("🚀 B-Trace API server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
