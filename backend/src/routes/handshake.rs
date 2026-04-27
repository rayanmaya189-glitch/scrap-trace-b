//! Handshake confirmation endpoints

use axum::{Json, extract::State};
use uuid::Uuid;
use chrono::Utc;
use crate::state::AppState;
use crate::error::Result;
use crate::models::ConfirmHandshakeRequest;
use crate::nats::Event;
use crate::crypto::{decode_hex, generate_idempotency_key};

/// POST /v1/handshake/confirm - Confirm a digital handshake
pub async fn confirm_handshake(
    State(state): State<AppState>,
    Json(payload): Json<ConfirmHandshakeRequest>,
) -> Result<Json<serde_json::Value>> {
    // Generate handshake ID
    let handshake_id = Uuid::new_v4();
    
    // Get buyer ID from JWT context (placeholder for now)
    let buyer_id = Uuid::new_v4();
    let supplier_id = Uuid::new_v4();
    
    // Decode buyer signature from hex
    let buyer_sig_bytes = decode_hex(&payload.buyer_signature)?;
    
    // Generate idempotency key
    let device_fingerprint = "device_002"; // TODO: Extract from request headers
    let timestamp = Utc::now().timestamp() as u64;
    let idempotency_key = generate_idempotency_key(
        format!("handshake:{}:{}:{}", handshake_id, payload.material_id, timestamp).as_bytes(),
        device_fingerprint,
        timestamp,
    );
    
    // Create NATS event payload
    let event = Event::HandshakeConfirmed {
        idempotency_key: idempotency_key.clone(),
        payload: crate::nats::HandshakeConfirmedPayload {
            handshake_id: handshake_id.to_string(),
            material_id: payload.material_id.to_string(),
            supplier_id: supplier_id.to_string(),
            buyer_id: buyer_id.to_string(),
            supplier_signature: "placeholder_supplier_sig".to_string(), // TODO: Get from DB
            buyer_signature: payload.buyer_signature,
            payload_hash: payload.payload_hash,
            hash_prev: "0000000000000000000000000000000000000000000000000000000000000000".to_string(), // TODO: Get previous hash from chain
            hash_current: payload.payload_hash.clone(), // TODO: Compute actual hash chain
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
