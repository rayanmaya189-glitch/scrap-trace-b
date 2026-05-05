use axum::{
    routing::{get, post},
    Router,
};

use crate::api::handlers::consent_handler::{ConsentState, create_consent, get_my_consents, get_supplier_consents, revoke_consent};
use crate::repositories::supplier_repository::SupplierRepository;
use sqlx::PgPool;

pub fn router(pool: PgPool, supplier_repo: SupplierRepository) -> Router {
    let state = ConsentState {
        pool,
        supplier_repo,
    };
    
    Router::new()
        .route("/", post(create_consent))
        .route("/my", get(get_my_consents))
        .route("/:supplier_id", get(get_supplier_consents))
        .route("/:id/revoke", post(revoke_consent))
        .with_state(state)
}
