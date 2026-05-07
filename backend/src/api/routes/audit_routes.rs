use axum::routing::{get, post};
use axum::Router;
use sqlx::PgPool;

use crate::api::handlers::audit_handler;

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/", get(audit_handler::get_audit_logs))
        .route("/merkle-proof/:entry_id", get(audit_handler::generate_merkle_proof))
        .route("/verify-proof", post(audit_handler::verify_merkle_proof))
        .route("/export", get(audit_handler::export_audit_trail))
}
