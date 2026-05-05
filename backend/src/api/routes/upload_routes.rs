use axum::routing::post;
use crate::api::handlers::upload_handler;

pub fn router() -> axum::Router<()> {
    axum::Router::new()
        .route("/evidence", post(upload_handler::upload_evidence))
}
