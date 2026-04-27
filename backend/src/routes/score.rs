//! Credit scoring endpoints

use axum::{Json, extract::State, extract::Path};
use uuid::Uuid;
use chrono::Utc;
use crate::state::AppState;
use crate::error::{Result, AppError};
use crate::models::{ScoringOutput, DefaultProbability, CreditRecommendation};

/// GET /v1/score/:supplier_id - Get credit score for a supplier
pub async fn get_score(
    State(state): State<AppState>,
    Path(supplier_id): Path<String>,
) -> Result<Json<ScoringOutput>> {
    // Parse supplier ID
    let supplier_uuid = Uuid::parse_str(&supplier_id)
        .map_err(|_| AppError::BadRequest("Invalid supplier ID format".into()))?;
    
    // TODO: Query database for scoring output or compute on-demand
    // For now, return placeholder scoring data
    
    let score = ScoringOutput {
        supplier_id: supplier_uuid,
        ics_score: 712,
        risk_grade: "B".to_string(),
        default_probability: DefaultProbability {
            day_90: 0.084,
            day_180: 0.132,
        },
        supply_chain_stability_index: 78.5,
        credit_recommendation: CreditRecommendation {
            recommended_limit_inr: 2850000,
            pricing_spread_percent: 14.25,
            base_rate_percent: 10.50,
            final_rate_percent: 24.75,
            collateral_required: false,
        },
        methodology_version: "v1.0-deterministic".to_string(),
        calculated_at: Utc::now(),
    };
    
    Ok(Json(score))
}
