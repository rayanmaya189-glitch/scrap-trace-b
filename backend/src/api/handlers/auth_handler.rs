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
use crate::services::RedisManager;
use crate::auth::JwtManager;

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

#[derive(Clone)]
pub struct AuthState {
    pub redis_manager: RedisManager,
    pub jwt_manager: JwtManager,
}

pub async fn request_otp(
    State(state): State<AuthState>,
    Json(payload): Json<RequestOtpRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation failed: {}", e))
    })?;

    // Rate limiting check
    let rate_limit_key = format!("otp_request:{}", payload.phone);
    let allowed = state.redis_manager
        .increment_rate_limit(&rate_limit_key, 5, 3600) // 5 requests per hour
        .await
        .map_err(|e| AppError::InternalServerError(format!("Rate limit check failed: {}", e)))?;
    
    if !allowed {
        return Err(AppError::TooManyRequests("Too many OTP requests. Please try again later.".to_string()));
    }

    let otp = generate_otp();
    
    // Store OTP in Redis with 5 minute TTL
    state.redis_manager
        .store_otp(&payload.phone, &otp)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to store OTP: {}", e)))?;

    // Send SMS (in production, integrate with Exotel/Twilio)
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
    State(state): State<AuthState>,
    Json(payload): Json<VerifyOtpRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation failed: {}", e))
    })?;

    // Get OTP from Redis
    let stored_otp = state.redis_manager
        .get_otp(&payload.phone)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to retrieve OTP: {}", e)))?
        .ok_or_else(|| AppError::BadRequest("OTP not found or expired".to_string()))?;

    if stored_otp != payload.otp {
        return Err(AppError::Unauthorized("Invalid OTP".to_string()));
    }

    // Delete OTP after successful verification
    state.redis_manager
        .delete_otp(&payload.phone)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to delete OTP: {}", e)))?;

    // Generate JWT tokens
    let access_token = state.jwt_manager
        .generate_access_token(
            payload.phone.clone(),
            None,
            vec![],
        )
        .map_err(|e| AppError::InternalServerError(format!("Failed to generate access token: {}", e)))?;

    let refresh_token = state.jwt_manager
        .generate_refresh_token(&payload.phone)
        .map_err(|e| AppError::InternalServerError(format!("Failed to generate refresh token: {}", e)))?;

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

async fn send_sms_otp(phone: &str, otp: &str) -> Result<(), AppError> {
    // TODO: Integrate with Exotel/Twilio
    // For now, log the OTP for development
    tracing::info!("SMS OTP to {}: {} (DEV MODE - Integrate Exotel/Twilio)", phone, otp);
    Ok(())
}

pub async fn refresh_token(
    State(state): State<AuthState>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<impl IntoResponse> {
    let refresh_token = payload.get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Refresh token required".to_string()))?;

    // Check if token is blacklisted
    let is_blacklisted = state.redis_manager
        .is_token_blacklisted(refresh_token)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Token blacklist check failed: {}", e)))?;
    
    if is_blacklisted {
        return Err(AppError::Unauthorized("Token has been revoked".to_string()));
    }

    // Refresh tokens
    let (new_access_token, new_refresh_token) = state.jwt_manager
        .refresh_access_token(refresh_token)
        .map_err(|e| AppError::Unauthorized(format!("Invalid refresh token: {}", e)))?;

    Ok(Json(ApiResponse::success(
        serde_json::json!({
            "access_token": new_access_token,
            "refresh_token": new_refresh_token,
            "expires_in": 86400,
            "token_type": "Bearer"
        }),
        "Token refreshed successfully".to_string(),
    )))
}

pub async fn logout(
    State(state): State<AuthState>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<impl IntoResponse> {
    let token = payload.get("token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Token required".to_string()))?;

    // Blacklist the token
    state.redis_manager
        .blacklist_token(token, 86400) // 24 hours
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to blacklist token: {}", e)))?;

    Ok(Json(ApiResponse::success(
        serde_json::json!({
            "logged_out": true
        }),
        "Logout successful".to_string(),
    )))
}
