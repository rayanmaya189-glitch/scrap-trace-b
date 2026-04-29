//! Core data models and DTOs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Material Passport - represents a batch of industrial material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialPassport {
    pub id: Uuid,
    pub material_type: String,
    pub batch_weight_kg: rust_decimal::Decimal,
    pub material_grade: String,
    pub source_pincode: String,
    pub supplier_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buyer_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub cbam_fields: serde_json::Value,
    pub status: MaterialStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MaterialStatus {
    Pending,
    Confirmed,
    InTransit,
    Delivered,
    Disputed,
}

impl Default for MaterialStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Digital Handshake - cryptographically verified transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalHandshake {
    pub id: Uuid,
    pub material_id: Uuid,
    pub supplier_sig: Vec<u8>,
    pub buyer_sig: Vec<u8>,
    pub payload_hash: String,
    pub hash_prev: String,
    pub hash_current: String,
    pub version_vector: serde_json::Value,
    pub sync_status: SyncStatus,
    pub timestamp_utc: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyncStatus {
    Local,
    Syncing,
    Synced,
    Conflict,
    Disputed,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::Local
    }
}

/// Supplier Profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierProfile {
    pub id: Uuid,
    pub name: String,
    pub phone: String,
    pub pincode: String,
    pub business_type: String,
    pub gst_number: Option<String>,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
}

/// Credit Scoring Output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringOutput {
    pub supplier_id: Uuid,
    pub ics_score: i32,
    pub risk_grade: String,
    pub default_probability: DefaultProbability,
    pub supply_chain_stability_index: f64,
    pub credit_recommendation: CreditRecommendation,
    pub methodology_version: String,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultProbability {
    pub day_90: f64,
    pub day_180: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditRecommendation {
    pub recommended_limit_inr: u64,
    pub pricing_spread_percent: f64,
    pub base_rate_percent: f64,
    pub final_rate_percent: f64,
    pub collateral_required: bool,
}

/// Request/Response DTOs

#[derive(Debug, Deserialize)]
pub struct CreateMaterialRequest {
    pub material_type: String,
    pub batch_weight_kg: rust_decimal::Decimal,
    pub material_grade: String,
    pub source_pincode: String,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub cbam_fields: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmHandshakeRequest {
    pub material_id: Uuid,
    pub buyer_signature: String, // hex-encoded Ed25519 signature
    pub payload_hash: String,
    pub version_vector: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct AuthRequestOtpRequest {
    pub phone: String,
}

#[derive(Debug, Serialize)]
pub struct AuthVerifyOtpRequest {
    pub phone: String,
    pub otp: String,
    pub device_fingerprint: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
}
