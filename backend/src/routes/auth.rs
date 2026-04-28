//! Authentication endpoints

use axum::{Json, extract::State};
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::state::AppState;
use crate::error::{Result, AppError};
use crate::models::{AuthResponse, AuthRequestOtpRequest, AuthVerifyOtpRequest};

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // supplier_id
    phone: String,
    device_fingerprint: String,
    exp: usize,
    iat: usize,
}

/// OTP storage (in production, use Redis)
struct OtpRecord {
    otp: String,
    expires_at: i64,
}

/// POST /v1/auth/request - Request OTP for login
pub async fn request_otp(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequestOtpRequest>,
) -> Result<Json<serde_json::Value>> {
    // Validate phone number (Indian format)
    if !is_valid_indian_phone(&payload.phone) {
        return Err(AppError::Validation("Invalid phone number format".into()));
    }

    // Generate 6-digit OTP
    let otp = generate_otp();
    
    // Calculate expiry (5 minutes)
    let expires_at = Utc::now().timestamp() + 300;

    // Store OTP in Redis with TTL
    let redis_key = format!("otp:{}", payload.phone);
    let mut redis_conn = state.redis.clone();
    
    redis::cmd("SET")
        .arg(&redis_key)
        .arg(&format!("{}:{}", otp, expires_at))
        .arg("EX")
        .arg(300) // 5 minute TTL
        .query_async::<()>(&mut redis_conn)
        .await
        .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

    // TODO: Send OTP via SMS (Exotel/Twilio integration)
    // For now, log OTP for development
    tracing::info!("OTP for {} is {} (expires in 5 min)", payload.phone, otp);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "OTP sent successfully",
        "phone": mask_phone(&payload.phone)
    })))
}

/// POST /v1/auth/verify - Verify OTP and get JWT tokens
pub async fn verify_otp(
    State(state): State<AppState>,
    Json(payload): Json<AuthVerifyOtpRequest>,
) -> Result<Json<AuthResponse>> {
    // Validate phone number
    if !is_valid_indian_phone(&payload.phone) {
        return Err(AppError::Validation("Invalid phone number format".into()));
    }

    // Validate OTP format
    if payload.otp.len() != 6 || !payload.otp.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::Validation("Invalid OTP format".into()));
    }

    // Retrieve OTP from Redis
    let redis_key = format!("otp:{}", payload.phone);
    let mut redis_conn = state.redis.clone();
    
    let stored_value: Option<String> = redis::cmd("GET")
        .arg(&redis_key)
        .query_async(&mut redis_conn)
        .await
        .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

    let stored_value = stored_value.ok_or_else(|| AppError::Auth("OTP expired or not found".into()))?;

    // Parse stored OTP and expiry
    let parts: Vec<&str> = stored_value.split(':').collect();
    if parts.len() != 2 {
        return Err(AppError::Internal("Invalid OTP storage format".into()));
    }

    let stored_otp = parts[0];
    let expires_at: i64 = parts[1].parse()
        .map_err(|_| AppError::Internal("Invalid OTP expiry format".into()))?;

    // Check expiry
    if Utc::now().timestamp() > expires_at {
        return Err(AppError::Auth("OTP expired".into()));
    }

    // Verify OTP
    if stored_otp != payload.otp {
        return Err(AppError::Auth("Invalid OTP".into()));
    }

    // Get or create supplier profile
    let supplier_id = get_or_create_supplier(&state.db_pool, &payload.phone, &payload.device_fingerprint).await?;

    // Delete used OTP
    redis::cmd("DEL")
        .arg(&redis_key)
        .query_async::<()>(&mut redis_conn)
        .await
        .ok();

    // Generate JWT tokens
    let access_token = generate_jwt(&supplier_id.to_string(), &payload.phone, &payload.device_fingerprint, 24)?;
    let refresh_token = generate_jwt(&supplier_id.to_string(), &payload.phone, &payload.device_fingerprint, 720)?; // 30 days

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        expires_in: 86400, // 24 hours
    }))
}

/// Generate a 6-digit OTP
fn generate_otp() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(0..1_000_000))
}

/// Validate Indian phone number format
fn is_valid_indian_phone(phone: &str) -> bool {
    // Accept formats: +91XXXXXXXXXX, 91XXXXXXXXXX, 0XXXXXXXXXX, XXXXXXXXXX
    let cleaned = phone.replace(['+', ' ', '-'], "");
    
    if cleaned.len() == 10 && cleaned.chars().all(|c| c.is_ascii_digit()) {
        return cleaned.starts_with(['6', '7', '8', '9']);
    }
    
    if cleaned.len() == 12 && cleaned.starts_with("91") && cleaned[2..].chars().all(|c| c.is_ascii_digit()) {
        return cleaned[2..].starts_with(['6', '7', '8', '9']);
    }
    
    false
}

/// Mask phone number for display
fn mask_phone(phone: &str) -> String {
    let cleaned = phone.replace(['+', ' ', '-'], "");
    if cleaned.len() >= 4 {
        format!("{}****{}", &cleaned[..2], &cleaned[cleaned.len()-2..])
    } else {
        "***".to_string()
    }
}

/// Generate JWT token
fn generate_jwt(sub: &str, phone: &str, device_fingerprint: &str, expiry_hours: u64) -> Result<String> {
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "btrace-dev-secret-key-change-in-production".to_string());
    
    let now = Utc::now();
    let exp = now + Duration::hours(expiry_hours as i64);

    let claims = Claims {
        sub: sub.to_string(),
        phone: phone.to_string(),
        device_fingerprint: device_fingerprint.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(&Header::default(), &claims, secret.as_bytes())
        .map_err(|e| AppError::Internal(format!("JWT encoding error: {}", e)))
}

/// Get or create supplier profile
async fn get_or_create_supplier(
    pool: &sqlx::PgPool,
    phone: &str,
    device_fingerprint: &str,
) -> Result<Uuid> {
    // Normalize phone number
    let normalized_phone = phone.replace(['+', ' ', '-'], "");
    
    // Try to find existing supplier
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM supplier_profile WHERE phone = $1"
    )
    .bind(&normalized_phone)
    .fetch_optional(pool)
    .await?;

    if let Some((id,)) = existing {
        return Ok(id);
    }

    // Create new supplier
    let new_id = Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO supplier_profile (id, name, phone, pincode, business_type, is_verified, device_public_key)
        VALUES ($1, $2, $3, $4, $5, FALSE, NULL)
        "#,
    )
    .bind(new_id)
    .bind(format!("Dealer_{}", &normalized_phone[..4]))
    .bind(&normalized_phone)
    .bind("000000") // Default pincode, to be updated
    .bind("DEALER")
    .execute(pool)
    .await?;

    Ok(new_id)
}
