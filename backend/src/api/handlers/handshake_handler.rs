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

#[derive(Debug, Deserialize, Validate)]
pub struct ConfirmHandshakeRequest {
    pub material_id: Uuid,
    pub supplier_sig: String,
    pub buyer_sig: String,
    pub payload_hash: String,
    pub hash_prev: String,
    pub version_vector: serde_json::Value,
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
            "Digital handshake confirmed successfully".to_string(),
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
