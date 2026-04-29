use axum::routing::{get, post};
use crate::api::handlers::auth_handler;

pub fn router() -> axum::Router<()> {
    axum::Router::new()
        .route("/request", post(auth_handler::request_otp))
        .route("/verify", post(auth_handler::verify_otp))
        .route("/refresh", post(auth_handler::refresh_token))
        .route("/logout", post(auth_handler::logout))
}
