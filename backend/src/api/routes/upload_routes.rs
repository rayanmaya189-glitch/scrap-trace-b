use axum::routing::post;
use axum::Router;
use std::sync::Arc;
use crate::services::MinioManager;
use crate::api::handlers::upload_handler;

pub fn router() -> Router<Arc<MinioManager>> {
    Router::new()
        .route("/evidence", post(upload_handler::upload_evidence))
}
