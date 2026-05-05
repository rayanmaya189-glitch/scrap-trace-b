use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::middleware::auth_middleware::Claims;
use crate::models::{ApiResponse, ConsentLog};
use crate::repositories::supplier_repository::SupplierRepository;
use sqlx::PgPool;

#[derive(Clone)]
pub struct ConsentState {
    pub pool: PgPool,
    pub supplier_repo: SupplierRepository,
}

#[derive(Debug, Deserialize)]
pub struct CreateConsentRequest {
    pub supplier_id: Uuid,
    pub purpose: String,
    pub granted: bool,
}

#[derive(Debug, Serialize)]
pub struct ConsentResponse {
    pub id: Uuid,
    pub supplier_id: Uuid,
    pub purpose: String,
    pub granted: bool,
    pub revoked_at: Option<chrono::DateTime<Utc>>,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<ConsentLog> for ConsentResponse {
    fn from(log: ConsentLog) -> Self {
        Self {
            id: log.id,
            supplier_id: log.supplier_id,
            purpose: log.purpose,
            granted: log.granted,
            revoked_at: log.revoked_at,
            created_at: log.created_at,
        }
    }
}

/// POST /v1/consent - Create a new consent record
pub async fn create_consent(
    State(state): State<ConsentState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateConsentRequest>,
) -> Result<Json<ApiResponse<ConsentResponse>>, (StatusCode, String)> {
    // Verify user has permission to create consent for this supplier
    if claims.sub != req.supplier_id.to_string() && claims.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Not authorized to create consent for this supplier".to_string(),
        ));
    }

    let consent = sqlx::query_as::<_, ConsentLog>(
        r#"
        INSERT INTO consent_logs (supplier_id, purpose, granted, created_at)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(req.supplier_id)
    .bind(&req.purpose)
    .bind(req.granted)
    .bind(Utc::now())
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create consent: {}", e),
        )
    })?;

    Ok(Json(ApiResponse::success(
        ConsentResponse::from(consent),
        "Consent recorded successfully".to_string(),
    )))
}

/// GET /v1/consent/:supplier_id - Get all consent records for a supplier
pub async fn get_supplier_consents(
    State(state): State<ConsentState>,
    Extension(claims): Extension<Claims>,
    Path(supplier_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<ConsentResponse>>>, (StatusCode, String)> {
    // Verify user has permission to view consents for this supplier
    if claims.sub != supplier_id.to_string() && claims.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Not authorized to view consents for this supplier".to_string(),
        ));
    }

    let consents = sqlx::query_as::<_, ConsentLog>(
        r#"
        SELECT * FROM consent_logs
        WHERE supplier_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(supplier_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch consents: {}", e),
        )
    })?;

    let responses: Vec<ConsentResponse> = consents.into_iter().map(ConsentResponse::from).collect();

    Ok(Json(ApiResponse::success(
        responses,
        format!("Retrieved {} consent records", responses.len()),
    )))
}

/// POST /v1/consent/:id/revoke - Revoke a consent
pub async fn revoke_consent(
    State(state): State<ConsentState>,
    Extension(claims): Extension<Claims>,
    Path(consent_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ConsentResponse>>, (StatusCode, String)> {
    // First verify the consent belongs to the user
    let consent = sqlx::query_as::<_, ConsentLog>("SELECT * FROM consent_logs WHERE id = $1")
        .bind(consent_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch consent: {}", e),
            )
        })?;

    let consent = match consent {
        Some(c) => c,
        None => {
            return Err((StatusCode::NOT_FOUND, "Consent not found".to_string()));
        }
    };

    // Verify user has permission to revoke this consent
    if claims.sub != consent.supplier_id.to_string() && claims.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Not authorized to revoke this consent".to_string(),
        ));
    }

    let updated_consent = sqlx::query_as::<_, ConsentLog>(
        r#"
        UPDATE consent_logs
        SET revoked_at = $1
        WHERE id = $2
        RETURNING *
        "#,
    )
    .bind(Utc::now())
    .bind(consent_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to revoke consent: {}", e),
        )
    })?;

    Ok(Json(ApiResponse::success(
        ConsentResponse::from(updated_consent),
        "Consent revoked successfully".to_string(),
    )))
}

/// GET /v1/consent - Get current user's consents
pub async fn get_my_consents(
    State(state): State<ConsentState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<ConsentResponse>>>, (StatusCode, String)> {
    let supplier_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid supplier ID in token: {}", e),
        )
    })?;

    let consents = sqlx::query_as::<_, ConsentLog>(
        r#"
        SELECT * FROM consent_logs
        WHERE supplier_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(supplier_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch consents: {}", e),
        )
    })?;

    let responses: Vec<ConsentResponse> = consents.into_iter().map(ConsentResponse::from).collect();

    Ok(Json(ApiResponse::success(
        responses,
        format!("Retrieved {} consent records", responses.len()),
    )))
}
