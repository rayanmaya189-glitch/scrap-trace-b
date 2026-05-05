use axum::{
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::middleware::auth_middleware::Claims;
use crate::models::ApiResponse;
use sqlx::PgPool;

#[derive(Clone)]
pub struct ReportState {
    pub pool: PgPool,
}

#[derive(Debug, Deserialize)]
pub struct GenerateReportRequest {
    pub report_type: String, // "cbam", "epr", "gst_audit", "consent_export"
    pub supplier_id: Option<Uuid>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub format: Option<String>, // "json", "csv"
}

#[derive(Debug, Serialize)]
pub struct ReportData {
    pub report_type: String,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
    pub row_count: usize,
}

/// POST /v1/reports/generate - Generate a compliance report
pub async fn generate_report(
    State(state): ReportState,
    Extension(claims): Extension<Claims>,
    Json(req): Json<GenerateReportRequest>,
) -> Result<Json<ApiResponse<ReportData>>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    
    match req.report_type.as_str() {
        "cbam" => generate_cbam_report(&state, &claims, req).await,
        "epr" => generate_epr_report(&state, &claims, req).await,
        "gst_audit" => generate_gst_audit_report(&state, &claims, req).await,
        "consent_export" => generate_consent_export(&state, &claims, req).await,
        _ => Err((
            StatusCode::BAD_REQUEST,
            format!("Unknown report type: {}", req.report_type),
        )),
    }
}

async fn generate_cbam_report(
    state: &ReportState,
    claims: &Claims,
    req: GenerateReportRequest,
) -> Result<Json<ApiResponse<ReportData>>, (StatusCode, String)> {
    // CBAM report: carbon emissions, embedded carbon, energy consumption
    let query = r#"
        SELECT 
            mp.id as material_id,
            mp.material_type,
            mp.batch_weight_kg,
            mp.cbam_fields->>'embedded_carbon_kg' as embedded_carbon,
            mp.cbam_fields->>'energy_consumption_kwh' as energy_consumption,
            mp.cbam_fields->>'direct_emissions' as direct_emissions,
            mp.cbam_fields->>'indirect_emissions' as indirect_emissions,
            mp.source_pincode,
            mp.created_at
        FROM material_passports mp
        WHERE mp.cbam_fields IS NOT NULL
        AND ($1::uuid IS NULL OR mp.supplier_id = $1)
        AND ($2::timestamptz IS NULL OR mp.created_at >= $2)
        AND ($3::timestamptz IS NULL OR mp.created_at <= $3)
        ORDER BY mp.created_at DESC
    "#;

    let rows = sqlx::query(query)
        .bind(req.supplier_id)
        .bind(req.start_date)
        .bind(req.end_date)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to generate CBAM report: {}", e),
            )
        })?;

    let total_carbon: f64 = rows
        .iter()
        .filter_map(|r| r.get::<Option<f64>, _>("embedded_carbon"))
        .sum();

    let data = serde_json::json!({
        "report_name": "CBAM Compliance Report",
        "total_batches": rows.len(),
        "total_embedded_carbon_kg": total_carbon,
        "batches": rows
    });

    Ok(Json(ApiResponse::success(
        ReportData {
            report_type: "cbam".to_string(),
            generated_at: now,
            data,
            row_count: rows.len(),
        },
        "CBAM report generated successfully".to_string(),
    )))
}

async fn generate_epr_report(
    state: &ReportState,
    claims: &Claims,
    req: GenerateReportRequest,
) -> Result<Json<ApiResponse<ReportData>>, (StatusCode, String)> {
    // EPR report: Extended Producer Responsibility tracking
    let query = r#"
        SELECT 
            mp.id as material_id,
            mp.material_type,
            mp.batch_weight_kg,
            mp.cbam_fields->>'recycled_content_percent' as recycled_content,
            mp.cbam_fields->>'epr_category' as epr_category,
            mp.cbam_fields->>'producer_responsibility_org' as pro,
            mp.source_pincode,
            mp.created_at
        FROM material_passports mp
        WHERE mp.cbam_fields IS NOT NULL
        AND ($1::uuid IS NULL OR mp.supplier_id = $1)
        AND ($2::timestamptz IS NULL OR mp.created_at >= $2)
        AND ($3::timestamptz IS NULL OR mp.created_at <= $3)
        ORDER BY mp.created_at DESC
    "#;

    let rows = sqlx::query(query)
        .bind(req.supplier_id)
        .bind(req.start_date)
        .bind(req.end_date)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to generate EPR report: {}", e),
            )
        })?;

    let total_weight: f64 = rows
        .iter()
        .filter_map(|r| r.get::<Option<rust_decimal::Decimal>, _>("batch_weight_kg"))
        .map(|d| d.to_f64().unwrap_or(0.0))
        .sum();

    let data = serde_json::json!({
        "report_name": "EPR Compliance Report",
        "total_batches": rows.len(),
        "total_weight_kg": total_weight,
        "batches": rows
    });

    Ok(Json(ApiResponse::success(
        ReportData {
            report_type: "epr".to_string(),
            generated_at: now,
            data,
            row_count: rows.len(),
        },
        "EPR report generated successfully".to_string(),
    )))
}

async fn generate_gst_audit_report(
    state: &ReportState,
    claims: &Claims,
    req: GenerateReportRequest,
) -> Result<Json<ApiResponse<ReportData>>, (StatusCode, String)> {
    // GST audit trail: all transactions with timestamps and parties
    let query = r#"
        SELECT 
            dh.id as handshake_id,
            dh.material_id,
            dh.payload_hash,
            dh.hash_prev,
            dh.hash_current,
            dh.timestamp_utc,
            mp.material_type,
            mp.batch_weight_kg,
            sp.name as supplier_name,
            sp.gst_number as supplier_gst,
            bp.name as buyer_name,
            bp.gst_number as buyer_gst
        FROM digital_handshakes dh
        JOIN material_passports mp ON dh.material_id = mp.id
        JOIN supplier_profiles sp ON mp.supplier_id = sp.id
        LEFT JOIN supplier_profiles bp ON mp.buyer_id = bp.id
        WHERE ($1::uuid IS NULL OR mp.supplier_id = $1)
        AND ($2::timestamptz IS NULL OR dh.timestamp_utc >= $2)
        AND ($3::timestamptz IS NULL OR dh.timestamp_utc <= $3)
        ORDER BY dh.timestamp_utc DESC
    "#;

    let rows = sqlx::query(query)
        .bind(req.supplier_id)
        .bind(req.start_date)
        .bind(req.end_date)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to generate GST audit report: {}", e),
            )
        })?;

    let data = serde_json::json!({
        "report_name": "GST Audit Trail Report",
        "total_transactions": rows.len(),
        "transactions": rows
    });

    Ok(Json(ApiResponse::success(
        ReportData {
            report_type: "gst_audit".to_string(),
            generated_at: now,
            data,
            row_count: rows.len(),
        },
        "GST audit report generated successfully".to_string(),
    )))
}

async fn generate_consent_export(
    state: &ReportState,
    claims: &Claims,
    req: GenerateReportRequest,
) -> Result<Json<ApiResponse<ReportData>>, (StatusCode, String)> {
    // DPDP consent export for data portability
    let query = r#"
        SELECT 
            cl.id,
            cl.supplier_id,
            cl.purpose,
            cl.granted,
            cl.revoked_at,
            cl.created_at,
            sp.name as supplier_name,
            sp.phone
        FROM consent_logs cl
        JOIN supplier_profiles sp ON cl.supplier_id = sp.id
        WHERE ($1::uuid IS NULL OR cl.supplier_id = $1)
        AND ($2::timestamptz IS NULL OR cl.created_at >= $2)
        AND ($3::timestamptz IS NULL OR cl.created_at <= $3)
        ORDER BY cl.created_at DESC
    "#;

    let rows = sqlx::query(query)
        .bind(req.supplier_id)
        .bind(req.start_date)
        .bind(req.end_date)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to generate consent export: {}", e),
            )
        })?;

    let data = serde_json::json!({
        "report_name": "DPDP Consent Export",
        "total_consents": rows.len(),
        "consents": rows
    });

    Ok(Json(ApiResponse::success(
        ReportData {
            report_type: "consent_export".to_string(),
            generated_at: now,
            data,
            row_count: rows.len(),
        },
        "Consent export generated successfully".to_string(),
    )))
}
