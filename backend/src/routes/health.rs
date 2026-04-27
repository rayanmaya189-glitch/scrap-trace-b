//! Health check endpoint

use axum::{Json, http::StatusCode};
use chrono::Utc;
use crate::models::HealthResponse;

/// GET /health - Health check endpoint
pub async fn handler() -> (StatusCode, Json<HealthResponse>) {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: Utc::now(),
        }),
    )
}
