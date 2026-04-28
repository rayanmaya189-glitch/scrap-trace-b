//! Scoring Engine - Deterministic Industrial Credit Score calculation
//! 
//! Implements the B-Trace scoring methodology as per PRD v1.0:
//! - ICS Score (300-900)
//! - Risk Grade (A-E)
//! - Probability of Default (PD)
//! - Supply Chain Stability Index
//! - Credit Recommendation

mod metrics;
mod calculator;
mod repository;

pub use metrics::{ScoringWeights, SupplierMetrics};
pub use calculator::{
    compute_ics_score,
    compute_risk_grade,
    compute_default_probability,
    compute_stability_index,
    compute_credit_recommendation,
    compute_volume_consistency,
    compute_tenure_months,
};
pub use repository::fetch_supplier_metrics;

use crate::error::{Result, AppError};
use crate::models::{ScoringOutput, DefaultProbability, CreditRecommendation};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

/// Generate complete scoring output for a supplier
pub async fn generate_scoring_output(
    pool: &PgPool,
    supplier_id: Uuid,
) -> Result<ScoringOutput> {
    let metrics = repository::fetch_supplier_metrics(pool, supplier_id).await?;
    let weights = ScoringWeights::default();

    let ics_score = calculator::compute_ics_score(&metrics, &weights);
    let risk_grade = calculator::compute_risk_grade(ics_score);
    let default_probability = calculator::compute_default_probability(&metrics);
    let stability_index = calculator::compute_stability_index(&metrics);
    let credit_recommendation = calculator::compute_credit_recommendation(
        ics_score,
        &default_probability,
        metrics.avg_monthly_flow,
    );

    Ok(ScoringOutput {
        supplier_id,
        ics_score,
        risk_grade,
        default_probability,
        supply_chain_stability_index: stability_index,
        credit_recommendation,
        methodology_version: "v1.0-deterministic".to_string(),
        calculated_at: Utc::now(),
    })
}
