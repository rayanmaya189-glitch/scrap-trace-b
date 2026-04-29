use axum::routing::{get, post, patch};
use crate::api::handlers::material_handler;

pub fn router() -> axum::Router<()> {
    axum::Router::new()
        .route("/", post(material_handler::create_material))
        .route("/", get(material_handler::list_materials))
        .route("/summary", get(material_handler::get_material_summary))
        .route("/:id", get(material_handler::get_material))
        .route("/:id/status/:status", patch(material_handler::update_material_status))
        .route("/:id/buyer/:buyer_id", patch(material_handler::assign_buyer_to_material))
        .route("/supplier/:supplier_id", get(material_handler::list_supplier_materials))
}
