//! Material logging endpoints

use axum::{Json, extract::State, http::HeaderMap};
use uuid::Uuid;
use chrono::Utc;
use crate::state::AppState;
use crate::error::{Result, AppError};
use crate::models::CreateMaterialRequest;
use crate::nats::Event;
use crate::crypto::generate_idempotency_key;

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

/// POST /v1/material - Create a new material passport
pub async fn create_material(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateMaterialRequest>,
) -> Result<Json<serde_json::Value>> {
    // Extract supplier ID from JWT
    let supplier_id = extract_supplier_id(&headers)?;
    
    // Generate material ID
    let material_id = Uuid::new_v4();
    
    // Get device fingerprint from headers (set by client)
    let device_fingerprint = headers
        .get("x-device-fingerprint")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown_device");
    
    // Generate idempotency key
    let timestamp = Utc::now().timestamp() as u64;
    let idempotency_key = generate_idempotency_key(
        format!("material:{}:{}:{}", material_id, supplier_id, timestamp).as_bytes(),
        device_fingerprint,
        timestamp,
    );
    
    // Create NATS event payload
    let event = Event::MaterialCreated {
        idempotency_key: idempotency_key.clone(),
        payload: crate::nats::MaterialCreatedPayload {
            material_id: material_id.to_string(),
            supplier_id: supplier_id.to_string(),
            material_type: payload.material_type.clone(),
            batch_weight_kg: payload.batch_weight_kg.to_string(),
            material_grade: payload.material_grade.clone(),
            source_pincode: payload.source_pincode.clone(),
            timestamp,
            device_fingerprint: device_fingerprint.to_string(),
        },
    };
    
    // Publish to NATS JetStream
    state.nats.publish_event("btrace.material.created", &event).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "material_id": material_id.to_string(),
        "status": "PENDING",
        "message": "Material created and queued for processing"
    })))
}
