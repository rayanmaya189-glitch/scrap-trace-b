//! Credit scoring endpoints

use axum::{Json, extract::State, extract::Path};
use uuid::Uuid;
use crate::state::AppState;
use crate::error::{Result, AppError};
use crate::models::ScoringOutput;
use crate::scoring::generate_scoring_output;

/// GET /v1/score/:supplier_id - Get credit score for a supplier
pub async fn get_score(
    State(state): State<AppState>,
    Path(supplier_id): Path<String>,
) -> Result<Json<ScoringOutput>> {
    // Parse supplier ID
    let supplier_uuid = Uuid::parse_str(&supplier_id)
        .map_err(|_| AppError::BadRequest("Invalid supplier ID format".into()))?;
    
    // Generate scoring output from database metrics
    let score = generate_scoring_output(&state.db_pool, supplier_uuid).await?;
    
    Ok(Json(score))
}
