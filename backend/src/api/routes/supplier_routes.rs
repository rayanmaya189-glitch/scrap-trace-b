use axum::routing::{get, post, put};
use crate::api::handlers::supplier_handler;

pub fn router() -> axum::Router<()> {
    axum::Router::new()
        .route("/", post(supplier_handler::create_supplier))
        .route("/", get(supplier_handler::list_suppliers))
        .route("/summary", get(supplier_handler::get_supplier_summary))
        .route("/me", get(supplier_handler::get_my_profile))
        .route("/:id", get(supplier_handler::get_supplier))
        .route("/:id", put(supplier_handler::update_supplier))
        .route("/:id/verify", post(supplier_handler::verify_supplier))
}
