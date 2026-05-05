use axum::{
    routing::post,
    Router,
};

use crate::api::handlers::report_handler::{ReportState, generate_report};
use sqlx::PgPool;

pub fn router(pool: PgPool) -> Router {
    let state = ReportState { pool };
    
    Router::new()
        .route("/generate", post(generate_report))
        .with_state(state)
}
