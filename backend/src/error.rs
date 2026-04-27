//! Error handling and response types

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Nats(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Redis(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Validation(e) => (StatusCode::BAD_REQUEST, e.clone()),
            AppError::Auth(e) => (StatusCode::UNAUTHORIZED, e.clone()),
            AppError::NotFound(e) => (StatusCode::NOT_FOUND, e.clone()),
            AppError::Conflict(e) => (StatusCode::CONFLICT, e.clone()),
            AppError::Internal(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.clone()),
            AppError::BadRequest(e) => (StatusCode::BAD_REQUEST, e.clone()),
        };

        let body = Json(json!({
            "error": true,
            "message": message,
            "type": format!("{:?}", self).split('(').next().unwrap_or("Unknown")
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
