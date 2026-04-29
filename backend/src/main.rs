mod api;
mod config;
mod db;
mod models;
mod repositories;
mod services;
mod utils;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::routes::{auth_routes, handshake_routes, material_routes, scoring_routes, supplier_routes};
use crate::config::AppConfig;
use crate::db::pool::{create_pool, run_migrations};
use crate::repositories::{material_repository::MaterialRepository, scoring_repository::ScoringRepository, supplier_repository::SupplierRepository};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env().expect("Failed to load configuration");

    let pool = create_pool(&config.database_url).await?;
    run_migrations(&pool).await?;

    let supplier_repo = SupplierRepository::new(pool.clone());
    let material_repo = MaterialRepository::new(pool.clone());
    let scoring_repo = ScoringRepository::new(pool.clone());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/v1/auth", auth_routes::router())
        .nest("/v1/suppliers", supplier_routes::router().with_state(supplier_repo.clone()))
        .nest("/v1/materials", material_routes::router().with_state((material_repo.clone(), supplier_repo.clone())))
        .nest("/v1/scores", scoring_routes::router().with_state((scoring_repo.clone(), supplier_repo.clone())))
        .nest("/v1/handshakes", handshake_routes::router())
        .route("/health", get(health_check))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(()); 

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("🚀 B-Trace API server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
