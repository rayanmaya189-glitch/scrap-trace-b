use axum::routing::{get, post};
use crate::api::handlers::auth_handler;
use crate::api::handlers::auth_handler::AuthState;

pub fn router(state: AuthState) -> axum::Router<()> {
    axum::Router::new()
        .route("/request-otp", post(auth_handler::request_otp).with_state(state.clone()))
        .route("/verify-otp", post(auth_handler::verify_otp).with_state(state.clone()))
        .route("/refresh", post(auth_handler::refresh_token).with_state(state.clone()))
        .route("/logout", post(auth_handler::logout).with_state(state))
}
