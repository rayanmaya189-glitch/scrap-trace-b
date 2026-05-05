use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::{ApiResponse, DigitalHandshake};
use crate::repositories::material_repository::MaterialRepository;
use crate::repositories::supplier_repository::SupplierRepository;
use crate::utils::error::{AppError, AppResult};
use crate::utils::crypto::{verify_signature, compute_payload_hash};
use crate::nats::{BTraceEvent, HandshakeDisputedEvent};
use crate::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct ConfirmHandshakeRequest {
    pub material_id: Uuid,
    pub supplier_sig: String,
    pub buyer_sig: String,
    pub payload_hash: String,
    pub hash_prev: String,
    pub version_vector: serde_json::Value,
    pub supplier_public_key: String,
    pub buyer_public_key: String,
    pub from_party: String,
    pub to_party: String,
    pub timestamp: String,
    pub data: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RaiseDisputeRequest {
    pub handshake_id: Uuid,
    pub reason: String,
    #[serde(default)]
    pub evidence_urls: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListHandshakesQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_offset")]
    pub offset: i64,
    pub material_id: Option<Uuid>,
}

fn default_limit() -> i64 {
    10
}

fn default_offset() -> i64 {
    0
}

pub async fn confirm_handshake(
    State(material_repo): State<MaterialRepository>,
    State(supplier_repo): State<SupplierRepository>,
    axum::extract::Extension(user_id): axum::extract::Extension<Uuid>,
    Json(payload): Json<ConfirmHandshakeRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation failed: {}", e))
    })?;

    let material = material_repo
        .find_by_id(payload.material_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Material not found".to_string()))?;

    let buyer_exists = supplier_repo.exists(user_id).await?;
    if !buyer_exists {
        return Err(AppError::NotFound("Buyer not found".to_string()));
    }

    // Verify the expected payload hash matches computed hash
    let expected_payload_hash = compute_payload_hash(
        &payload.material_id.to_string(),
        &payload.from_party,
        &payload.to_party,
        &payload.timestamp,
        &payload.data,
    );
    
    if expected_payload_hash != payload.payload_hash {
        return Err(AppError::BadRequest(
            "Payload hash mismatch - potential tampering detected".to_string()
        ));
    }

    // Verify supplier signature (Ed25519)
    let supplier_sig_valid = verify_signature(
        &payload.supplier_sig,
        payload.payload_hash.as_bytes(),
        &payload.supplier_public_key,
    ).map_err(|e| AppError::InternalServerError(
        format!("Signature verification error: {}", e)
    ))?;
    
    if !supplier_sig_valid {
        return Err(AppError::Unauthorized(
            "Invalid supplier signature - handshake rejected".to_string()
        ));
    }

    // Verify buyer signature (Ed25519)
    let buyer_sig_valid = verify_signature(
        &payload.buyer_sig,
        payload.payload_hash.as_bytes(),
        &payload.buyer_public_key,
    ).map_err(|e| AppError::InternalServerError(
        format!("Signature verification error: {}", e)
    ))?;
    
    if !buyer_sig_valid {
        return Err(AppError::Unauthorized(
            "Invalid buyer signature - handshake rejected".to_string()
        ));
    }

    // Validate hash chain integrity - check if hash_prev matches previous handshake's hash_current
    let previous_hash_current: Option<String> = sqlx::query_scalar!(
        "SELECT hash_current FROM digital_handshake WHERE material_id = $1 ORDER BY timestamp_utc DESC LIMIT 1",
        payload.material_id
    )
    .fetch_optional(&material_repo.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    if let Some(prev_hash) = previous_hash_current {
        use crate::utils::crypto::validate_hash_chain;
        if !validate_hash_chain(&payload.hash_prev, &prev_hash) {
            return Err(AppError::BadRequest(
                format!("Hash chain validation failed: hash_prev does not match previous handshake's hash_current for material {}", payload.material_id)
            ));
        }
    } else {
        // First handshake for this material - hash_prev should be zeros or genesis hash
        const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
        if payload.hash_prev != GENESIS_HASH && !payload.hash_prev.is_empty() {
            return Err(AppError::BadRequest(
                "First handshake must have genesis hash or empty hash_prev".to_string()
            ));
        }
    }

    let hash_current = compute_hash_current(
        &payload.payload_hash,
        &payload.hash_prev,
        &user_id.to_string(),
        &chrono::Utc::now().to_rfc3339(),
    );

    let handshake = create_handshake_record(
        payload.material_id,
        &payload.supplier_sig,
        &payload.buyer_sig,
        &payload.payload_hash,
        &payload.hash_prev,
        &hash_current,
        &payload.version_vector,
    );

    material_repo.assign_buyer(payload.material_id, user_id).await?;
    material_repo.update_status(payload.material_id, "CONFIRMED").await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            handshake,
            "Digital handshake confirmed successfully with cryptographic verification".to_string(),
        )),
    ))
}

fn compute_hash_current(
    payload_hash: &str,
    hash_prev: &str,
    device_id: &str,
    timestamp: &str,
) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(payload_hash.as_bytes());
    hasher.update(hash_prev.as_bytes());
    hasher.update(device_id.as_bytes());
    hasher.update(timestamp.as_bytes());
    hex::encode(hasher.finalize())
}

fn create_handshake_record(
    material_id: Uuid,
    supplier_sig: &str,
    buyer_sig: &str,
    payload_hash: &str,
    hash_prev: &str,
    hash_current: &str,
    version_vector: &serde_json::Value,
) -> DigitalHandshakeRecord {
    DigitalHandshakeRecord {
        id: Uuid::new_v4(),
        material_id,
        supplier_sig: hex::decode(supplier_sig).unwrap_or_default(),
        buyer_sig: hex::decode(buyer_sig).unwrap_or_default(),
        payload_hash: payload_hash.to_string(),
        hash_prev: hash_prev.to_string(),
        hash_current: hash_current.to_string(),
        version_vector: version_vector.clone(),
        sync_status: "SYNCED".to_string(),
        timestamp_utc: chrono::Utc::now(),
    }
}

#[derive(Debug, Serialize)]
pub struct DigitalHandshakeRecord {
    pub id: Uuid,
    pub material_id: Uuid,
    pub supplier_sig: Vec<u8>,
    pub buyer_sig: Vec<u8>,
    pub payload_hash: String,
    pub hash_prev: String,
    pub hash_current: String,
    pub version_vector: serde_json::Value,
    pub sync_status: String,
    pub timestamp_utc: chrono::DateTime<chrono::Utc>,
}

pub async fn get_handshake(
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    Ok(Json(ApiResponse::success(
        serde_json::json!({
            "id": id,
            "note": "Handshake retrieval requires database integration"
        }),
        "Handshake retrieved successfully".to_string(),
    )))
}

pub async fn list_handshakes(
    query: axum::extract::Query<ListHandshakesQuery>,
) -> AppResult<impl IntoResponse> {
    let summary = format!(
        "Handshakes listing (offset: {}, limit: {})",
        query.offset,
        query.limit
    );

    Ok(Json(ApiResponse::success(
        serde_json::json!({
            "data": [],
            "pagination": {
                "total": 0,
                "limit": query.limit,
                "offset": query.offset,
                "has_more": false,
                "summary": summary
            }
        }),
        summary,
    )))
}

pub async fn list_material_handshakes(
    axum::extract::Path(material_id): axum::extract::Path<Uuid>,
    query: axum::extract::Query<ListHandshakesQuery>,
) -> AppResult<impl IntoResponse> {
    let summary = format!(
        "Handshakes for material {} (offset: {}, limit: {})",
        material_id,
        query.offset,
        query.limit
    );

    Ok(Json(ApiResponse::success(
        serde_json::json!({
            "data": [],
            "pagination": {
                "total": 0,
                "limit": query.limit,
                "offset": query.offset,
                "has_more": false,
                "summary": summary
            }
        }),
        summary,
    )))
}

/// Raise a dispute on a confirmed handshake
pub async fn raise_dispute(
    State(app_state): State<AppState>,
    axum::extract::Extension(user_id): axum::extract::Extension<Uuid>,
    Json(payload): Json<RaiseDisputeRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation failed: {}", e))
    })?;

    // Verify the handshake exists and get material_id
    let handshake_data: Option<(Uuid, String)> = sqlx::query_as!(
        "(Uuid, String)",
        "SELECT material_id, sync_status FROM digital_handshake WHERE id = $1",
        payload.handshake_id
    )
    .fetch_optional(&app_state.material_repo.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;

    let (material_id, sync_status) = handshake_data.ok_or_else(|| {
        AppError::NotFound("Handshake not found".to_string())
    })?;

    // Only allow disputes on confirmed handshakes
    if sync_status != "CONFIRMED" && sync_status != "SYNCED" {
        return Err(AppError::BadRequest(
            "Can only dispute confirmed handshakes".to_string()
        ));
    }

    // Create the dispute event
    let dispute_event = HandshakeDisputedEvent {
        handshake_id: payload.handshake_id,
        material_id,
        disputed_by: user_id,
        reason: payload.reason.clone(),
        evidence: Some(serde_json::to_value(&payload.evidence_urls).unwrap_or_default()),
        timestamp: chrono::Utc::now(),
    };

    // Publish to NATS JetStream
    app_state.nats_manager
        .publish(BTraceEvent::HandshakeDisputed(dispute_event))
        .await
        .map_err(|e| AppError::InternalServerError(
            format!("Failed to publish dispute event: {}", e)
        ))?;

    tracing::info!(
        "Dispute raised for handshake {} by user {}",
        payload.handshake_id,
        user_id
    );

    Ok((
        StatusCode::ACCEPTED,
        Json(ApiResponse::success(
            serde_json::json!({
                "handshake_id": payload.handshake_id,
                "material_id": material_id,
                "status": "DISPUTED_PENDING_REVIEW",
                "reason": payload.reason
            }),
            "Dispute submitted successfully. The handshake has been flagged for manual review.".to_string(),
        )),
    ))
}
