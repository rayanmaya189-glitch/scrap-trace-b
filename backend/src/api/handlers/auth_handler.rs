use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::ApiResponse;
use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Deserialize, Validate)]
pub struct RequestOtpRequest {
    #[validate(length(min = 10, max = 15, message = "Phone must be 10-15 digits"))]
    pub phone: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct VerifyOtpRequest {
    #[validate(length(min = 10, max = 15))]
    pub phone: String,
    #[validate(length(min = 4, max = 8, message = "OTP must be 4-8 digits"))]
    pub otp: String,
}

#[derive(Debug, Serialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user_id: String,
    pub phone: String,
    pub role: Option<String>,
    pub tokens: AuthTokens,
}

pub async fn request_otp(
    Json(payload): Json<RequestOtpRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation failed: {}", e))
    })?;

    let otp = generate_otp();
    
    store_otp_temporarily(&payload.phone, &otp).await;

    send_sms_otp(&payload.phone, &otp).await.ok();

    Ok(Json(ApiResponse::success(
        serde_json::json!({
            "phone": payload.phone,
            "otp_sent": true,
            "expires_in_seconds": 300
        }),
        "OTP sent successfully".to_string(),
    )))
}

pub async fn verify_otp(
    Json(payload): Json<VerifyOtpRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation failed: {}", e))
    })?;

    let stored_otp = get_stored_otp(&payload.phone).await
        .ok_or_else(|| AppError::BadRequest("OTP not found or expired".to_string()))?;

    if stored_otp != payload.otp {
        return Err(AppError::Unauthorized("Invalid OTP".to_string()));
    }

    clear_stored_tp(&payload.phone).await;

    let (access_token, refresh_token) = generate_jwt_tokens(&payload.phone).await;

    let response = AuthResponse {
        user_id: uuid::Uuid::new_v4().to_string(),
        phone: payload.phone,
        role: None,
        tokens: AuthTokens {
            access_token,
            refresh_token,
            expires_in: 86400,
            token_type: "Bearer".to_string(),
        },
    };

    Ok(Json(ApiResponse::success(
        response,
        "Authentication successful".to_string(),
    )))
}

fn generate_otp() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(100000..999999))
}

async fn store_otp_temporarily(phone: &str, otp: &str) {
    tracing::info!("Storing OTP for phone: {} (in production, use Redis)", phone);
}

async fn get_stored_otp(phone: &str) -> Option<String> {
    tracing::info!("Retrieving OTP for phone: {} (in production, use Redis)", phone);
    None
}

async fn clear_stored_otp(phone: &str) {
    tracing::info!("Clearing OTP for phone: {} (in production, use Redis)", phone);
}

async fn send_sms_otp(phone: &str, otp: &str) -> Result<(), AppError> {
    tracing::info!("Sending SMS OTP to {}: {} (in production, use Exotel/Twilio)", phone, otp);
    Ok(())
}

async fn generate_jwt_tokens(phone: &str) -> (String, String) {
    let access_token = format!("mock_access_token_for_{}", phone);
    let refresh_token = format!("mock_refresh_token_for_{}", phone);
    (access_token, refresh_token)
}

pub async fn refresh_token(
    Json(payload): Json<serde_json::Value>,
) -> AppResult<impl IntoResponse> {
    let refresh_token = payload.get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Refresh token required".to_string()))?;

    let new_access_token = format!("new_access_token_for_{}", refresh_token);

    Ok(Json(ApiResponse::success(
        serde_json::json!({
            "access_token": new_access_token,
            "expires_in": 86400,
            "token_type": "Bearer"
        }),
        "Token refreshed successfully".to_string(),
    )))
}

pub async fn logout() -> AppResult<impl IntoResponse> {
    Ok(Json(ApiResponse::success(
        serde_json::json!({
            "logged_out": true
        }),
        "Logout successful".to_string(),
    )))
}
