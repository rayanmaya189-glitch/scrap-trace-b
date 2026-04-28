//! Handshake confirmation endpoints

use axum::{Json, extract::State, http::HeaderMap};
use uuid::Uuid;
use chrono::Utc;
use crate::state::AppState;
use crate::error::{Result, AppError};
use crate::models::ConfirmHandshakeRequest;
use crate::nats::Event;
use crate::crypto::{decode_hex, generate_idempotency_key, compute_hash_chain};

/// Extract supplier ID from JWT token in headers
fn extract_supplier_id(headers: &HeaderMap) -> Result<Uuid> {
    let auth_header = headers
        .get("authorization")
        .ok_or_else(|| AppError::Auth("Missing authorization header".into()))?
        .to_str()
        .map_err(|_| AppError::Auth("Invalid authorization header format".into()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Auth("Invalid authorization scheme".into()));
    }

    let token = &auth_header[7..];
    
    // Decode JWT to extract supplier ID
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "btrace-dev-secret-key-change-in-production".to_string());
    
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    let token_data = jsonwebtoken::decode::<serde_json::Value>(token, secret.as_bytes(), &validation)
        .map_err(|_| AppError::Auth("Invalid or expired token".into()))?;
    
    let sub = token_data.claims
        .get("sub")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Auth("Token missing subject claim".into()))?;
    
    Uuid::parse_str(sub)
        .map_err(|_| AppError::Auth("Invalid supplier ID in token".into()))
}

/// POST /v1/handshake/confirm - Confirm a digital handshake
pub async fn confirm_handshake(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ConfirmHandshakeRequest>,
) -> Result<Json<serde_json::Value>> {
    // Extract buyer ID from JWT (the person confirming)
    let buyer_id = extract_supplier_id(&headers)?;
    
    // Generate handshake ID
    let handshake_id = Uuid::new_v4();
    
    // Get device fingerprint from headers
    let device_fingerprint = headers
        .get("x-device-fingerprint")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown_device");
    
    // Get timestamp
    let timestamp = Utc::now().timestamp() as u64;
    
    // Fetch material to get supplier ID
    let supplier_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT supplier_id FROM material_passport WHERE id = $1"
    )
    .bind(payload.material_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Material not found".into()))?;
    
    // Fetch previous hash from chain (last handshake for this material or supplier)
    let prev_hash: Option<String> = sqlx::query_scalar(
        r#"
        SELECT hash_current FROM digital_handshake dh
        JOIN material_passport mp ON dh.material_id = mp.id
        WHERE mp.supplier_id = $1 OR mp.buyer_id = $1
        ORDER BY dh.timestamp_utc DESC
        LIMIT 1
        "#
    )
    .bind(buyer_id)
    .fetch_optional(&state.db_pool)
    .await?;
    
    let hash_prev = prev_hash.unwrap_or_else(|| "0000000000000000000000000000000000000000000000000000000000000000".to_string());
    
    // Compute hash chain: Hₙ = SHA-256(payloadₙ + Hₙ₋₁ + device_salt + timestamp)
    let payload_bytes = format!("{}:{}:{}:{}", payload.material_id, buyer_id, supplier_id, timestamp);
    let hash_current = compute_hash_chain(
        payload_bytes.as_bytes(),
        &hash_prev,
        device_fingerprint.as_bytes(),
        timestamp,
    );
    
    // Generate idempotency key
    let idempotency_key = generate_idempotency_key(
        format!("handshake:{}:{}:{}", handshake_id, payload.material_id, timestamp).as_bytes(),
        device_fingerprint,
        timestamp,
    );
    
    // Note: In production, supplier_signature would be fetched from DB (created during material creation)
    // For now, we use a placeholder that should be replaced with actual signature
    let supplier_signature = "placeholder_supplier_sig".to_string();
    
    // Create NATS event payload
    let event = Event::HandshakeConfirmed {
        idempotency_key: idempotency_key.clone(),
        payload: crate::nats::HandshakeConfirmedPayload {
            handshake_id: handshake_id.to_string(),
            material_id: payload.material_id.to_string(),
            supplier_id: supplier_id.to_string(),
            buyer_id: buyer_id.to_string(),
            supplier_signature,
            buyer_signature: payload.buyer_signature,
            payload_hash: payload.payload_hash,
            hash_prev,
            hash_current,
            timestamp,
        },
    };
    
    // Publish to NATS JetStream
    state.nats.publish_event("btrace.handshake.confirmed", &event).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "handshake_id": handshake_id.to_string(),
        "material_id": payload.material_id.to_string(),
        "status": "SYNCED",
        "message": "Handshake confirmed and queued for processing"
    })))
}
