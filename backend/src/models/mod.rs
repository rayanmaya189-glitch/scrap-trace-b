use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SupplierProfile {
    pub id: Uuid,
    pub phone: String,
    pub name: Option<String>,
    pub business_name: Option<String>,
    pub role: String,
    pub pincode: Option<String>,
    pub gst_number: Option<String>,
    pub is_verified: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MaterialPassport {
    pub id: Uuid,
    pub material_type: String,
    pub batch_weight_kg: rust_decimal::Decimal,
    pub material_grade: String,
    pub source_pincode: String,
    pub supplier_id: Uuid,
    pub buyer_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub cbam_fields: serde_json::Value,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DigitalHandshake {
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

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ScoringOutput {
    pub id: Uuid,
    pub supplier_id: Uuid,
    pub ics_score: i32,
    pub risk_grade: String,
    pub default_probability_90d: Option<rust_decimal::Decimal>,
    pub default_probability_180d: Option<rust_decimal::Decimal>,
    pub stability_index: Option<rust_decimal::Decimal>,
    pub recommended_limit_inr: Option<rust_decimal::Decimal>,
    pub pricing_spread_percent: Option<rust_decimal::Decimal>,
    pub base_rate_percent: Option<rust_decimal::Decimal>,
    pub final_rate_percent: Option<rust_decimal::Decimal>,
    pub collateral_required: bool,
    pub methodology_version: String,
    pub calculated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ConsentLog {
    pub id: Uuid,
    pub supplier_id: Uuid,
    pub purpose: String,
    pub granted: bool,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            message: message.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
    pub summary: String,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, limit: i64, offset: i64, summary: String) -> Self {
        let has_more = offset + limit < total;
        Self {
            success: true,
            data,
            pagination: PaginationMeta {
                total,
                limit,
                offset,
                has_more,
                summary,
            },
            message: format!("Retrieved {} items (total: {})", data.len(), total),
        }
    }
}
