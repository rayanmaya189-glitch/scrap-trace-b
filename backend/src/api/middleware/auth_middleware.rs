use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

pub async fn auth_middleware<B>(
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    if let Some(auth) = auth_header {
        if auth.starts_with("Bearer ") {
            let token = &auth[7..];
            
            if let Ok(user_id) = validate_token(token) {
                request.extensions_mut().insert(user_id);
                return Ok(next.run(request).await);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

fn validate_token(token: &str) -> Result<Uuid, String> {
    if token.starts_with("mock_access_token_for_") {
        Ok(Uuid::new_v4())
    } else {
        Err("Invalid token".to_string())
    }
}

pub async fn optional_auth_middleware<B>(
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    if let Some(auth) = auth_header {
        if auth.starts_with("Bearer ") {
            let token = &auth[7..];
            if let Ok(user_id) = validate_token(token) {
                request.extensions_mut().insert(user_id);
            }
        }
    }

    next.run(request).await
}
