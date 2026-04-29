use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::{ApiResponse, PaginatedResponse, MaterialPassport};
use crate::repositories::material_repository::MaterialRepository;
use crate::repositories::supplier_repository::SupplierRepository;
use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMaterialRequest {
    #[validate(length(min = 2, max = 50))]
    pub material_type: String,
    #[validate(range(min = 0.01, message = "Weight must be positive"))]
    pub batch_weight_kg: Decimal,
    #[validate(length(min = 2, max = 20))]
    pub material_grade: String,
    #[validate(length(equal = 6, message = "Pincode must be 6 digits"))]
    pub source_pincode: String,
    pub metadata: Option<serde_json::Value>,
    pub cbam_fields: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListMaterialsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_offset")]
    pub offset: i64,
    pub status: Option<String>,
}

fn default_limit() -> i64 {
    10
}

fn default_offset() -> i64 {
    0
}

#[derive(Debug, Serialize)]
pub struct MaterialSummary {
    pub total_materials: i64,
    pub by_status: serde_json::Value,
    pub total_weight_kg: Decimal,
}

pub async fn create_material(
    State(material_repo): State<MaterialRepository>,
    State(supplier_repo): State<SupplierRepository>,
    axum::extract::Extension(user_id): axum::extract::Extension<Uuid>,
    Json(payload): Json<CreateMaterialRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation failed: {}", e))
    })?;

    let supplier_exists = supplier_repo.exists(user_id).await?;
    if !supplier_exists {
        return Err(AppError::NotFound("Supplier not found".to_string()));
    }

    let material = material_repo
        .create(
            &payload.material_type,
            payload.batch_weight_kg,
            &payload.material_grade,
            &payload.source_pincode,
            user_id,
            payload.metadata,
            payload.cbam_fields,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            material,
            "Material passport created successfully".to_string(),
        )),
    ))
}

pub async fn get_material(
    State(repo): State<MaterialRepository>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let material = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("Material not found".to_string()))?;

    Ok(Json(ApiResponse::success(
        material,
        "Material retrieved successfully".to_string(),
    )))
}

pub async fn list_materials(
    State(repo): State<MaterialRepository>,
    query: axum::extract::Query<ListMaterialsQuery>,
) -> AppResult<impl IntoResponse> {
    let (materials, total) = repo.list(query.limit, query.offset, query.status.as_deref()).await?;

    let summary = format!(
        "Showing {} of {} materials (offset: {}, limit: {})",
        materials.len(),
        total,
        query.offset,
        query.limit
    );

    let response = PaginatedResponse::new(materials, total, query.limit, query.offset, summary);

    Ok(Json(response))
}

pub async fn list_supplier_materials(
    State(repo): State<MaterialRepository>,
    axum::extract::Path(supplier_id): axum::extract::Path<Uuid>,
    query: axum::extract::Query<ListMaterialsQuery>,
) -> AppResult<impl IntoResponse> {
    let (materials, total) = repo.list_by_supplier(supplier_id, query.limit, query.offset).await?;

    let summary = format!(
        "Showing {} of {} materials for supplier (offset: {}, limit: {})",
        materials.len(),
        total,
        query.offset,
        query.limit
    );

    let response = PaginatedResponse::new(materials, total, query.limit, query.offset, summary);

    Ok(Json(response))
}

pub async fn update_material_status(
    State(repo): State<MaterialRepository>,
    axum::extract::Path((id, status)): axum::extract::Path<(Uuid, String)>,
) -> AppResult<impl IntoResponse> {
    let material = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("Material not found".to_string()))?;

    repo.update_status(id, &status).await?;

    let updated = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::InternalError("Failed to fetch updated material".to_string()))?;

    Ok(Json(ApiResponse::success(
        updated,
        format!("Material status updated to {}", status),
    )))
}

pub async fn assign_buyer_to_material(
    State(material_repo): State<MaterialRepository>,
    State(supplier_repo): State<SupplierRepository>,
    axum::extract::Path((id, buyer_id)): axum::extract::Path<(Uuid, Uuid)>,
) -> AppResult<impl IntoResponse> {
    let material = material_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("Material not found".to_string()))?;

    let buyer_exists = supplier_repo.exists(buyer_id).await?;
    if !buyer_exists {
        return Err(AppError::NotFound("Buyer not found".to_string()));
    }

    material_repo.assign_buyer(id, buyer_id).await?;

    let updated = material_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::InternalError("Failed to fetch updated material".to_string()))?;

    Ok(Json(ApiResponse::success(
        updated,
        "Buyer assigned to material successfully".to_string(),
    )))
}

pub async fn get_material_summary(
    State(repo): State<MaterialRepository>,
) -> AppResult<impl IntoResponse> {
    let all_materials = repo.list(10000, 0, None).await?;
    let total = all_materials.1;
    
    let mut by_status = serde_json::Map::new();
    let mut total_weight = Decimal::ZERO;
    
    for material in &all_materials.0 {
        let count = by_status
            .get(&material.status)
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            + 1;
        by_status.insert(material.status.clone(), serde_json::json!(count));
        total_weight = total_weight + material.batch_weight_kg;
    }

    let summary = MaterialSummary {
        total_materials: total,
        by_status: serde_json::Value::Object(by_status),
        total_weight_kg: total_weight,
    };

    Ok(Json(ApiResponse::success(
        summary,
        "Material summary retrieved successfully".to_string(),
    )))
}
