//! Material logging endpoints

use axum::{Json, extract::State};
use uuid::Uuid;
use chrono::Utc;
use crate::state::AppState;
use crate::error::{Result, AppError};
use crate::models::{CreateMaterialRequest, MaterialStatus};
use crate::nats::Event;
use crate::crypto::{hash_payload, generate_idempotency_key};

/// POST /v1/material - Create a new material passport
pub async fn create_material(
    State(state): State<AppState>,
    Json(payload): Json<CreateMaterialRequest>,
) -> Result<Json<serde_json::Value>> {
    // Generate material ID
    let material_id = Uuid::new_v4();
    
    // Get supplier ID from JWT context (placeholder for now)
    let supplier_id = Uuid::new_v4();
    
    // Generate idempotency key
    let device_fingerprint = "device_001"; // TODO: Extract from request headers
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
        "status": MaterialStatus::Pending,
        "message": "Material created and queued for processing"
    })))
}
