use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{ApiResponse, ScoringOutput};
use crate::repositories::scoring_repository::ScoringRepository;
use crate::repositories::supplier_repository::SupplierRepository;
use crate::services::scoring_service::{ScoringParams, ScoringService};
use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct CalculateScoreRequest {
    pub verification_reliability: f64,
    pub payment_timeliness: f64,
    pub volume_consistency: f64,
    pub dispute_rate: f64,
    pub network_tenure_months: i32,
    pub avg_delay_days: f64,
    pub concentration_ratio: f64,
    pub redundancy_score: f64,
    pub geo_dispersion_score: f64,
    pub reliability_score: f64,
    pub verified_monthly_flow: f64,
    pub base_rate: f64,
    pub base_pd: f64,
    pub compliance_adjustment: i32,
}

#[derive(Debug, Serialize)]
pub struct ScoreResponse {
    pub supplier_id: Uuid,
    pub ics_score: i32,
    pub risk_grade: String,
    pub default_probability: DefaultProbability,
    pub supply_chain_stability_index: Decimal,
    pub credit_recommendation: CreditRecommendation,
    pub methodology_version: String,
}

#[derive(Debug, Serialize)]
pub struct DefaultProbability {
    pub ninety_day: Decimal,
    pub one_eighty_day: Decimal,
}

#[derive(Debug, Serialize)]
pub struct CreditRecommendation {
    pub recommended_limit_inr: Decimal,
    pub pricing_spread_percent: Decimal,
    pub base_rate_percent: Decimal,
    pub final_rate_percent: Decimal,
    pub collateral_required: bool,
}

pub async fn calculate_score(
    State(scoring_repo): State<ScoringRepository>,
    State(supplier_repo): State<SupplierRepository>,
    axum::extract::Path(supplier_id): axum::extract::Path<Uuid>,
    Json(payload): Json<CalculateScoreRequest>,
) -> AppResult<impl IntoResponse> {
    let supplier_exists = supplier_repo.exists(supplier_id).await?;
    if !supplier_exists {
        return Err(AppError::NotFound("Supplier not found".to_string()));
    }

    let scoring_params = ScoringService::generate_scoring_output(
        supplier_id,
        payload.verification_reliability,
        payload.payment_timeliness,
        payload.volume_consistency,
        payload.dispute_rate,
        payload.network_tenure_months,
        payload.avg_delay_days,
        payload.concentration_ratio,
        payload.redundancy_score,
        payload.geo_dispersion_score,
        payload.reliability_score,
        payload.verified_monthly_flow,
        payload.base_rate,
        payload.base_pd,
        payload.compliance_adjustment,
    );

    let expires_at = Some(chrono::Utc::now() + chrono::Duration::days(90));

    let scoring_output = scoring_repo
        .upsert(
            scoring_params.supplier_id,
            scoring_params.ics_score,
            &scoring_params.risk_grade,
            Some(Decimal::from_f64_retain(scoring_params.default_probability_90d).unwrap_or(Decimal::ZERO)),
            Some(Decimal::from_f64_retain(scoring_params.default_probability_180d).unwrap_or(Decimal::ZERO)),
            Some(Decimal::from_f64_retain(scoring_params.stability_index).unwrap_or(Decimal::ZERO)),
            Some(Decimal::from_f64_retain(scoring_params.recommended_limit).unwrap_or(Decimal::ZERO)),
            Some(Decimal::from_f64_retain(scoring_params.pricing_spread).unwrap_or(Decimal::ZERO)),
            Some(Decimal::from_f64_retain(scoring_params.base_rate).unwrap_or(Decimal::ZERO)),
            Some(Decimal::from_f64_retain(scoring_params.final_rate).unwrap_or(Decimal::ZERO)),
            scoring_params.collateral_required,
            "v1.0-deterministic",
            expires_at,
        )
        .await?;

    let response = ScoreResponse {
        supplier_id: scoring_output.supplier_id,
        ics_score: scoring_output.ics_score,
        risk_grade: scoring_output.risk_grade,
        default_probability: DefaultProbability {
            ninety_day: scoring_output.default_probability_90d.unwrap_or(Decimal::ZERO),
            one_eighty_day: scoring_output.default_probability_180d.unwrap_or(Decimal::ZERO),
        },
        supply_chain_stability_index: scoring_output.stability_index.unwrap_or(Decimal::ZERO),
        credit_recommendation: CreditRecommendation {
            recommended_limit_inr: scoring_output.recommended_limit_inr.unwrap_or(Decimal::ZERO),
            pricing_spread_percent: scoring_output.pricing_spread_percent.unwrap_or(Decimal::ZERO),
            base_rate_percent: scoring_output.base_rate_percent.unwrap_or(Decimal::ZERO),
            final_rate_percent: scoring_output.final_rate_percent.unwrap_or(Decimal::ZERO),
            collateral_required: scoring_output.collateral_required,
        },
        methodology_version: scoring_output.methodology_version,
    };

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            response,
            "Credit score calculated and saved successfully".to_string(),
        )),
    ))
}

pub async fn get_score(
    State(repo): State<ScoringRepository>,
    axum::extract::Path(supplier_id): axum::extract::Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let scoring = repo
        .find_by_supplier_id(supplier_id)
        .await?
        .ok_or_else(|| AppError::NotFound("No score found for this supplier".to_string()))?;

    let response = ScoreResponse {
        supplier_id: scoring.supplier_id,
        ics_score: scoring.ics_score,
        risk_grade: scoring.risk_grade,
        default_probability: DefaultProbability {
            ninety_day: scoring.default_probability_90d.unwrap_or(Decimal::ZERO),
            one_eighty_day: scoring.default_probability_180d.unwrap_or(Decimal::ZERO),
        },
        supply_chain_stability_index: scoring.stability_index.unwrap_or(Decimal::ZERO),
        credit_recommendation: CreditRecommendation {
            recommended_limit_inr: scoring.recommended_limit_inr.unwrap_or(Decimal::ZERO),
            pricing_spread_percent: scoring.pricing_spread_percent.unwrap_or(Decimal::ZERO),
            base_rate_percent: scoring.base_rate_percent.unwrap_or(Decimal::ZERO),
            final_rate_percent: scoring.final_rate_percent.unwrap_or(Decimal::ZERO),
            collateral_required: scoring.collateral_required,
        },
        methodology_version: scoring.methodology_version,
    };

    Ok(Json(ApiResponse::success(
        response,
        "Credit score retrieved successfully".to_string(),
    )))
}

pub async fn recalculate_score(
    State(scoring_repo): State<ScoringRepository>,
    State(supplier_repo): State<SupplierRepository>,
    axum::extract::Path(supplier_id): axum::extract::Path<Uuid>,
    Json(payload): Json<CalculateScoreRequest>,
) -> AppResult<impl IntoResponse> {
    let supplier_exists = supplier_repo.exists(supplier_id).await?;
    if !supplier_exists {
        return Err(AppError::NotFound("Supplier not found".to_string()));
    }

    let existing_score = scoring_repo.find_by_supplier_id(supplier_id).await?;
    if existing_score.is_none() {
        return Err(AppError::NotFound(
            "No existing score to recalculate. Use calculate endpoint first.".to_string(),
        ));
    }

    let scoring_params = ScoringService::generate_scoring_output(
        supplier_id,
        payload.verification_reliability,
        payload.payment_timeliness,
        payload.volume_consistency,
        payload.dispute_rate,
        payload.network_tenure_months,
        payload.avg_delay_days,
        payload.concentration_ratio,
        payload.redundancy_score,
        payload.geo_dispersion_score,
        payload.reliability_score,
        payload.verified_monthly_flow,
        payload.base_rate,
        payload.base_pd,
        payload.compliance_adjustment,
    );

    let expires_at = Some(chrono::Utc::now() + chrono::Duration::days(90));

    let scoring_output = scoring_repo
        .upsert(
            scoring_params.supplier_id,
            scoring_params.ics_score,
            &scoring_params.risk_grade,
            Some(Decimal::from_f64_retain(scoring_params.default_probability_90d).unwrap_or(Decimal::ZERO)),
            Some(Decimal::from_f64_retain(scoring_params.default_probability_180d).unwrap_or(Decimal::ZERO)),
            Some(Decimal::from_f64_retain(scoring_params.stability_index).unwrap_or(Decimal::ZERO)),
            Some(Decimal::from_f64_retain(scoring_params.recommended_limit).unwrap_or(Decimal::ZERO)),
            Some(Decimal::from_f64_retain(scoring_params.pricing_spread).unwrap_or(Decimal::ZERO)),
            Some(Decimal::from_f64_retain(scoring_params.base_rate).unwrap_or(Decimal::ZERO)),
            Some(Decimal::from_f64_retain(scoring_params.final_rate).unwrap_or(Decimal::ZERO)),
            scoring_params.collateral_required,
            "v1.0-deterministic",
            expires_at,
        )
        .await?;

    let response = ScoreResponse {
        supplier_id: scoring_output.supplier_id,
        ics_score: scoring_output.ics_score,
        risk_grade: scoring_output.risk_grade,
        default_probability: DefaultProbability {
            ninety_day: scoring_output.default_probability_90d.unwrap_or(Decimal::ZERO),
            one_eighty_day: scoring_output.default_probability_180d.unwrap_or(Decimal::ZERO),
        },
        supply_chain_stability_index: scoring_output.stability_index.unwrap_or(Decimal::ZERO),
        credit_recommendation: CreditRecommendation {
            recommended_limit_inr: scoring_output.recommended_limit_inr.unwrap_or(Decimal::ZERO),
            pricing_spread_percent: scoring_output.pricing_spread_percent.unwrap_or(Decimal::ZERO),
            base_rate_percent: scoring_output.base_rate_percent.unwrap_or(Decimal::ZERO),
            final_rate_percent: scoring_output.final_rate_percent.unwrap_or(Decimal::ZERO),
            collateral_required: scoring_output.collateral_required,
        },
        methodology_version: scoring_output.methodology_version,
    };

    Ok(Json(ApiResponse::success(
        response,
        "Credit score recalculated and updated successfully".to_string(),
    )))
}
