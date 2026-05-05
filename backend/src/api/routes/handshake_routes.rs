use axum::routing::{get, post};
use crate::api::handlers::handshake_handler;

pub fn router() -> axum::Router<()> {
    axum::Router::new()
        .route("/confirm", post(handshake_handler::confirm_handshake))
        .route("/dispute", post(handshake_handler::raise_dispute))
        .route("/", get(handshake_handler::list_handshakes))
        .route("/:id", get(handshake_handler::get_handshake))
        .route("/material/:material_id", get(handshake_handler::list_material_handshakes))
}
