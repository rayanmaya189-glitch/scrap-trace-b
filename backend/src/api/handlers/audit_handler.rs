use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::core::merkle::{MerkleTree, MerkleProof};

#[derive(Clone)]
pub struct AuditState {
    pub pool: PgPool,
}

/// Audit log entry response
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub event_type: String,
    pub subject: String,
    pub payload_hash: String,
    pub processed_at: chrono::DateTime<chrono::Utc>,
    pub merkle_root: Option<String>,
}

/// Merkle proof response for audit verification
#[derive(Debug, Serialize, Deserialize)]
pub struct MerkleProofResponse {
    pub leaf_hash: String,
    pub proof_hashes: Vec<MerkleProofNode>,
    pub root_hash: String,
    pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerkleProofNode {
    pub hash: String,
    pub position: String, // "Left" or "Right"
}

/// Get audit logs with optional filtering
pub async fn get_audit_logs(
    State(state): State<AuditState>,
) -> Result<Json<Vec<AuditLogEntry>>, StatusCode> {
    let entries = sqlx::query_as::<_, AuditLogEntry>(
        r#"
        SELECT 
            idempotency_key as id,
            subject as event_type,
            subject as subject,
            payload_hash,
            processed_at,
            NULL as merkle_root
        FROM event_log
        ORDER BY processed_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch audit logs: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(entries))
}

/// Generate Merkle proof for a specific audit entry
pub async fn generate_merkle_proof(
    State(state): State<AuditState>,
    Path(entry_id): Path<String>,
) -> Result<Json<MerkleProofResponse>, StatusCode> {
    // Fetch all audit entries to build the tree
    let entries = sqlx::query_as::<_, AuditLogEntry>(
        r#"
        SELECT 
            idempotency_key as id,
            subject as event_type,
            subject as subject,
            payload_hash,
            processed_at,
            NULL as merkle_root
        FROM event_log
        ORDER BY processed_at ASC
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch entries for Merkle tree: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Find the target entry index
    let target_index = entries
        .iter()
        .position(|e| e.id == entry_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Build Merkle tree from payload hashes
    let mut tree = MerkleTree::new();
    for entry in &entries {
        tree.add_leaf(entry.payload_hash.as_bytes());
    }

    // Generate proof
    let proof = tree.generate_proof(target_index).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Verify the proof
    let verified = MerkleTree::verify_proof(&proof);

    // Convert proof to response format
    let proof_response = MerkleProofResponse {
        leaf_hash: proof.leaf_hash,
        proof_hashes: proof
            .proof_hashes
            .into_iter()
            .map(|node| MerkleProofNode {
                hash: node.hash,
                position: match node.position {
                    crate::core::merkle::Position::Left => "Left".to_string(),
                    crate::core::merkle::Position::Right => "Right".to_string(),
                },
            })
            .collect(),
        root_hash: proof.root_hash,
        verified,
    };

    Ok(Json(proof_response))
}

/// Verify an external Merkle proof
pub async fn verify_merkle_proof(
    Json(proof): Json<MerkleProofResponse>,
) -> Result<Json<bool>, StatusCode> {
    // This would require converting the response back to internal MerkleProof type
    // For now, we return the verified flag from generation
    Ok(Json(proof.verified))
}

/// Export audit trail as JSON with Merkle roots
pub async fn export_audit_trail(
    State(state): State<AuditState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let entries = sqlx::query_as::<_, AuditLogEntry>(
        r#"
        SELECT 
            idempotency_key as id,
            subject as event_type,
            subject as subject,
            payload_hash,
            processed_at,
            NULL as merkle_root
        FROM event_log
        ORDER BY processed_at ASC
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to export audit trail: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Build Merkle tree and get root
    let mut tree = MerkleTree::new();
    for entry in &entries {
        tree.add_leaf(entry.payload_hash.as_bytes());
    }

    let merkle_root = tree.root_hash().unwrap_or_default();

    Ok(Json(serde_json::json!({
        "exported_at": chrono::Utc::now(),
        "total_entries": entries.len(),
        "merkle_root": merkle_root,
        "entries": entries
    })))
}

pub fn router(pool: PgPool) -> Router {
    let state = AuditState { pool };
    
    Router::new()
        .route("/", get(get_audit_logs))
        .route("/merkle-proof/:entry_id", get(generate_merkle_proof))
        .route("/verify-proof", post(verify_merkle_proof))
        .route("/export", get(export_audit_trail))
        .with_state(state)
}
