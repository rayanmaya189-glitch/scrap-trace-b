use axum::routing::{get, post};
use crate::api::handlers::scoring_handler;

pub fn router() -> axum::Router<()> {
    axum::Router::new()
        .route("/:supplier_id", post(scoring_handler::calculate_score))
        .route("/:supplier_id", get(scoring_handler::get_score))
        .route("/:supplier_id/recalculate", post(scoring_handler::recalculate_score))
}
