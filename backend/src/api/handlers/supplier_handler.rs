use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::{ApiResponse, PaginatedResponse, SupplierProfile};
use crate::repositories::supplier_repository::SupplierRepository;
use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSupplierRequest {
    #[validate(length(min = 10, max = 15, message = "Phone must be 10-15 digits"))]
    pub phone: String,
    #[validate(length(min = 3, max = 100))]
    pub name: Option<String>,
    #[validate(length(min = 3, max = 200))]
    pub business_name: Option<String>,
    #[validate(length(equal = 6, message = "Pincode must be 6 digits"))]
    pub pincode: Option<String>,
    #[validate(pattern(r"^[0-9]{2}[A-Z]{5}[0-9]{4}[A-Z]{1}$", message = "Invalid GST format"))]
    pub gst_number: Option<String>,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct ListSuppliersQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_offset")]
    pub offset: i64,
    pub role: Option<String>,
}

fn default_limit() -> i64 {
    10
}

fn default_offset() -> i64 {
    0
}

#[derive(Debug, Serialize)]
pub struct SupplierSummary {
    pub total_suppliers: i64,
    pub verified_count: i64,
    pub by_role: serde_json::Value,
}

pub async fn create_supplier(
    State(repo): State<SupplierRepository>,
    Json(payload): Json<CreateSupplierRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation failed: {}", e))
    })?;

    if let Some(existing) = repo.find_by_phone(&payload.phone).await? {
        return Ok((
            StatusCode::CONFLICT,
            Json(ApiResponse::<SupplierProfile>::error(
                "Supplier with this phone already exists".to_string(),
            )),
        ));
    }

    let supplier = repo
        .create(
            &payload.phone,
            &payload.role,
            payload.name.as_deref(),
            payload.business_name.as_deref(),
            payload.pincode.as_deref(),
            payload.gst_number.as_deref(),
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            supplier,
            "Supplier created successfully".to_string(),
        )),
    ))
}

pub async fn get_supplier(
    State(repo): State<SupplierRepository>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let supplier = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("Supplier not found".to_string()))?;

    Ok(Json(ApiResponse::success(
        supplier,
        "Supplier retrieved successfully".to_string(),
    )))
}

pub async fn list_suppliers(
    State(repo): State<SupplierRepository>,
    query: axum::extract::Query<ListSuppliersQuery>,
) -> AppResult<impl IntoResponse> {
    let (suppliers, total) = repo.list(query.limit, query.offset, query.role.as_deref()).await?;

    let summary = format!(
        "Showing {} of {} suppliers (offset: {}, limit: {})",
        suppliers.len(),
        total,
        query.offset,
        query.limit
    );

    let response = PaginatedResponse::new(suppliers, total, query.limit, query.offset, summary);

    Ok(Json(response))
}

pub async fn verify_supplier(
    State(repo): State<SupplierRepository>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let supplier = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("Supplier not found".to_string()))?;

    if supplier.is_verified {
        return Ok((
            StatusCode::OK,
            Json(ApiResponse::error("Supplier already verified".to_string())),
        ));
    }

    repo.update_verification(id, true).await?;

    let updated = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::InternalError("Failed to fetch updated supplier".to_string()))?;

    Ok(Json(ApiResponse::success(
        updated,
        "Supplier verified successfully".to_string(),
    )))
}

pub async fn get_supplier_summary(
    State(repo): State<SupplierRepository>,
) -> AppResult<impl IntoResponse> {
    let all_suppliers = repo.list(10000, 0, None).await?;
    let total = all_suppliers.1;
    
    let verified_count = all_suppliers.0.iter().filter(|s| s.is_verified).count() as i64;
    
    let mut by_role = serde_json::Map::new();
    for supplier in &all_suppliers.0 {
        let count = by_role
            .get(&supplier.role)
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            + 1;
        by_role.insert(supplier.role.clone(), serde_json::json!(count));
    }

    let summary = SupplierSummary {
        total_suppliers: total,
        verified_count,
        by_role: serde_json::Value::Object(by_role),
    };

    Ok(Json(ApiResponse::success(
        summary,
        "Supplier summary retrieved successfully".to_string(),
    )))
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSupplierRequest {
    #[validate(length(min = 3, max = 100))]
    pub name: Option<String>,
    #[validate(length(min = 3, max = 200))]
    pub business_name: Option<String>,
    #[validate(length(equal = 6, message = "Pincode must be 6 digits"))]
    pub pincode: Option<String>,
    #[validate(pattern(r"^[0-9]{2}[A-Z]{5}[0-9]{4}[A-Z]{1}$", message = "Invalid GST format"))]
    pub gst_number: Option<String>,
}

pub async fn update_supplier(
    State(repo): State<SupplierRepository>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateSupplierRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation failed: {}", e))
    })?;

    let supplier = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("Supplier not found".to_string()))?;

    repo.update_profile(
        id,
        payload.name.as_deref(),
        payload.business_name.as_deref(),
        payload.pincode.as_deref(),
        payload.gst_number.as_deref(),
    ).await?;

    let updated = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::InternalError("Failed to fetch updated supplier".to_string()))?;

    Ok(Json(ApiResponse::success(
        updated,
        "Supplier profile updated successfully".to_string(),
    )))
}

pub async fn get_my_profile(
    State(repo): State<SupplierRepository>,
    extensions: axum::extract::Extension<crate::auth::jwt::Claims>,
) -> AppResult<impl IntoResponse> {
    let supplier_id = extensions.0.sub.parse::<Uuid>()
        .map_err(|_| AppError::Unauthorized("Invalid supplier ID in token".to_string()))?;

    let supplier = repo
        .find_by_id(supplier_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Supplier not found".to_string()))?;

    Ok(Json(ApiResponse::success(
        supplier,
        "Profile retrieved successfully".to_string(),
    )))
}
