//! B-Trace Backend - Industrial Traceability & Credit Protocol
//! 
//! This is the main entry point for the B-Trace Axum API server.
//! It handles authentication, rate limiting, validation, and publishes events to NATS JetStream.

mod config;
mod crypto;
mod error;
mod models;
mod nats;
mod routes;
mod state;
mod scoring;
mod consumer;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    limit::RequestBodyLimitLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::time::Duration;

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "btrace_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    tracing::info!("Starting B-Trace Backend v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = Config::load()?;
    tracing::info!("Configuration loaded successfully");

    // Initialize application state
    let state = AppState::new(&config).await?;
    tracing::info!("Application state initialized");

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::HeaderName::from_static("x-ratelimit-limit"),
            axum::http::header::HeaderName::from_static("x-ratelimit-remaining"),
        ])
        .max_age(Duration::from_secs(86400));

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(routes::health::handler))
        // Auth endpoints
        .route("/v1/auth/request", post(routes::auth::request_otp))
        .route("/v1/auth/verify", post(routes::auth::verify_otp))
        // Material endpoints
        .route("/v1/material", post(routes::material::create_material))
        // Handshake endpoints
        .route("/v1/handshake/confirm", post(routes::handshake::confirm_handshake))
        // Score endpoints
        .route("/v1/score/:supplier_id", get(routes::score::get_score))
        // Export endpoints
        .route("/v1/export/:plugin/:id.:format", get(routes::export::generate_export))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024)) // 2MB limit
        .with_state(state);

    // Start server
    let addr = config.server_addr;
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
