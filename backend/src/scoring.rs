//! Scoring Engine - Deterministic Industrial Credit Score calculation
//! 
//! Implements the B-Trace scoring methodology as per PRD v1.0:
//! - ICS Score (300-900)
//! - Risk Grade (A-E)
//! - Probability of Default (PD)
//! - Supply Chain Stability Index
//! - Credit Recommendation

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use chrono::{DateTime, Utc, Duration};
use sqlx::PgPool;
use crate::error::{Result, AppError};
use crate::models::{ScoringOutput, DefaultProbability, CreditRecommendation};

/// Feature weights for ICS calculation
pub struct ScoringWeights {
    pub verification_reliability: f64,  // 0.30
    pub payment_timeliness: f64,        // 0.25
    pub volume_consistency: f64,        // 0.20
    pub dispute_rate: f64,              // -0.15 (inverted)
    pub network_tenure: f64,            // 0.10
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            verification_reliability: 0.30,
            payment_timeliness: 0.25,
            volume_consistency: 0.20,
            dispute_rate: 0.15,
            network_tenure: 0.10,
        }
    }
}

/// Aggregated supplier metrics for scoring
pub struct SupplierMetrics {
    pub total_handshakes: i64,
    pub confirmed_handshakes: i64,
    pub on_time_payments: i64,
    pub delayed_payments: i64,
    pub disputed_transactions: i64,
    pub monthly_volumes: Vec<Decimal>,
    pub first_transaction_date: Option<DateTime<Utc>>,
    pub avg_monthly_flow: Decimal,
}

/// Compute Industrial Credit Score (ICS)
/// Formula: 500 + Σ(wᵢ × norm(xᵢ)), clamped 300–900
pub fn compute_ics_score(metrics: &SupplierMetrics, weights: &ScoringWeights) -> i32 {
    // Verification Reliability: confirmed / total (0-1, normalized to 0-100)
    let verification_ratio = if metrics.total_handshakes > 0 {
        metrics.confirmed_handshakes as f64 / metrics.total_handshakes as f64
    } else {
        0.0
    };
    let verification_normalized = verification_ratio * 100.0;

    // Payment Timeliness: on_time / (on_time + delayed) (0-1, normalized to 0-100)
    let total_payments = metrics.on_time_payments + metrics.delayed_payments;
    let timeliness_ratio = if total_payments > 0 {
        metrics.on_time_payments as f64 / total_payments as f64
    } else {
        0.5 // neutral default
    };
    let timeliness_normalized = timeliness_ratio * 100.0;

    // Volume Consistency: 1 - (std_dev / mean), normalized to 0-100
    let volume_consistency = compute_volume_consistency(&metrics.monthly_volumes);
    let volume_normalized = volume_consistency * 100.0;

    // Dispute Rate: inverted (lower is better), normalized to 0-100
    let dispute_ratio = if metrics.total_handshakes > 0 {
        metrics.disputed_transactions as f64 / metrics.total_handshakes as f64
    } else {
        0.0
    };
    let dispute_normalized = (1.0 - dispute_ratio) * 100.0;

    // Network Tenure: months active (capped at 36), normalized to 0-100
    let tenure_months = compute_tenure_months(metrics.first_transaction_date);
    let tenure_normalized = (tenure_months.min(36) as f64 / 36.0) * 100.0;

    // Weighted sum
    let score = 500.0
        + (weights.verification_reliability * (verification_normalized - 50.0) * 2.0)
        + (weights.payment_timeliness * (timeliness_normalized - 50.0) * 2.0)
        + (weights.volume_consistency * (volume_normalized - 50.0) * 2.0)
        + (weights.dispute_rate * (dispute_normalized - 50.0) * 2.0)
        + (weights.network_tenure * (tenure_normalized - 50.0) * 2.0);

    // Clamp to 300-900
    score.round() as i32
        .max(300)
        .min(900)
}

/// Compute volume consistency using coefficient of variation
fn compute_volume_consistency(volumes: &[Decimal]) -> f64 {
    if volumes.is_empty() || volumes.len() < 2 {
        return 0.5; // neutral default
    }

    let n = volumes.len() as f64;
    let mean: f64 = volumes.iter()
        .map(|v| v.to_f64().unwrap_or(0.0))
        .sum::<f64>() / n;

    if mean <= 0.0 {
        return 0.5;
    }

    let variance: f64 = volumes.iter()
        .map(|v| {
            let val = v.to_f64().unwrap_or(0.0);
            (val - mean).powi(2)
        })
        .sum::<f64>() / n;

    let std_dev = variance.sqrt();
    let cv = std_dev / mean; // Coefficient of variation

    // Convert to consistency score (lower CV = higher consistency)
    // CV of 0 = 100% consistency, CV of 1+ = 0% consistency
    (1.0 - cv.min(1.0)).max(0.0)
}

/// Compute tenure in months from first transaction
fn compute_tenure_months(first_date: Option<DateTime<Utc>>) -> i32 {
    match first_date {
        Some(date) => {
            let now = Utc::now();
            let duration = now.signed_duration_since(date);
            (duration.num_days() / 30) as i32
        }
        None => 0,
    }
}

/// Map ICS score to risk grade
pub fn compute_risk_grade(ics_score: i32) -> String {
    match ics_score {
        s if s >= 750 => "A".to_string(),
        s if s >= 650 => "B".to_string(),
        s if s >= 550 => "C".to_string(),
        s if s >= 450 => "D".to_string(),
        _ => "E".to_string(),
    }
}

/// Compute Probability of Default (PD)
/// Formula: PD_base × exp(β₁·Delay + β₂·Disputes + β₃·Concentration), clamped 0.1%–45.0%
pub fn compute_default_probability(metrics: &SupplierMetrics) -> DefaultProbability {
    let base_pd = 0.05; // 5% base PD

    // Delay factor: delayed / total payments
    let total_payments = metrics.on_time_payments + metrics.delayed_payments;
    let delay_ratio = if total_payments > 0 {
        metrics.delayed_payments as f64 / total_payments as f64
    } else {
        0.0
    };

    // Dispute factor: disputed / total handshakes
    let dispute_ratio = if metrics.total_handshakes > 0 {
        metrics.disputed_transactions as f64 / metrics.total_handshakes as f64
    } else {
        0.0
    };

    // Concentration factor (simplified: based on volume variance)
    let concentration = 1.0 - compute_volume_consistency(&metrics.monthly_volumes);

    // Beta coefficients (calibrated to RBI SME NPA baselines)
    let beta_delay = 1.5;
    let beta_disputes = 2.0;
    let beta_concentration = 0.8;

    let pd_multiplier = (beta_delay * delay_ratio
        + beta_disputes * dispute_ratio
        + beta_concentration * concentration).exp();

    let pd_90 = (base_pd * pd_multiplier).clamp(0.001, 0.450);
    let pd_180 = (pd_90 * 1.5).clamp(0.001, 0.450); // 180-day PD typically higher

    DefaultProbability {
        day_90: pd_90,
        day_180: pd_180,
    }
}

/// Compute Supply Chain Stability Index
/// Formula: (0.4×Redundancy)+(0.3×GeoDispersion)+(0.3×Reliability)
pub fn compute_stability_index(metrics: &SupplierMetrics) -> f64 {
    // Redundancy: based on number of transactions (proxy for network depth)
    let redundancy = (metrics.total_handshakes as f64 / 100.0).min(1.0);

    // Geo-dispersion: simplified (would need pincode diversity in real impl)
    let geo_dispersion = 0.5; // placeholder

    // Reliability: confirmed / total
    let reliability = if metrics.total_handshakes > 0 {
        metrics.confirmed_handshakes as f64 / metrics.total_handshakes as f64
    } else {
        0.0
    };

    let stability = 0.4 * redundancy + 0.3 * geo_dispersion + 0.3 * reliability;
    (stability * 100.0).round() / 100.0 // 2 decimal places
}

/// Compute credit recommendation
pub fn compute_credit_recommendation(
    ics_score: i32,
    pd: &DefaultProbability,
    avg_monthly_flow: Decimal,
) -> CreditRecommendation {
    // ICS multiplier for credit limit
    let ics_multiplier = match ics_score {
        s if s >= 750 => 3.5,
        s if s >= 650 => 2.0,
        s if s >= 550 => 1.0,
        s if s >= 450 => 0.5,
        _ => 0.25,
    };

    let monthly_flow_f64 = avg_monthly_flow.to_f64().unwrap_or(0.0);
    let recommended_limit = (monthly_flow_f64 * ics_multiplier) as u64;

    // Pricing: Base Rate + (PD × 100 × 2.5) + 0.5%, RBI-capped at 36%
    let base_rate = 10.5; // Base rate percent
    let pd_spread = pd.day_90 * 100.0 * 2.5;
    let processing_fee = 0.5;
    let final_rate = (base_rate + pd_spread + processing_fee).min(36.0);
    let pricing_spread = final_rate - base_rate;

    // Collateral required for E-grade or high PD
    let collateral_required = ics_score < 450 || pd.day_90 > 0.30;

    CreditRecommendation {
        recommended_limit_inr: recommended_limit,
        pricing_spread_percent: (pricing_spread * 100.0).round() / 100.0,
        base_rate_percent: base_rate,
        final_rate_percent: (final_rate * 100.0).round() / 100.0,
        collateral_required,
    }
}

/// Fetch supplier metrics from database
pub async fn fetch_supplier_metrics(pool: &PgPool, supplier_id: uuid::Uuid) -> Result<SupplierMetrics> {
    // Get handshake statistics
    let handshake_stats: (i64, i64, i64) = sqlx::query_as::<_, (i64, i64, i64)>(
        r#"
        SELECT 
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE sync_status = 'SYNCED') as confirmed,
            COUNT(*) FILTER (WHERE sync_status = 'DISPUTED') as disputed
        FROM digital_handshake dh
        JOIN material_passport mp ON dh.material_id = mp.id
        WHERE mp.supplier_id = $1
        "#,
    )
    .bind(supplier_id)
    .fetch_one(pool)
    .await?;

    // Get payment timeliness (from material status transitions)
    let payment_stats: (i64, i64) = sqlx::query_as::<_, (i64, i64)>(
        r#"
        SELECT 
            COUNT(*) FILTER (WHERE status = 'DELIVERED') as on_time,
            COUNT(*) FILTER (WHERE status = 'DISPUTED') as delayed
        FROM material_passport
        WHERE supplier_id = $1
        "#,
    )
    .bind(supplier_id)
    .fetch_one(pool)
    .await?;

    // Get monthly volumes (aggregated by month)
    let monthly_volumes: Vec<rust_decimal::Decimal> = sqlx::query_scalar(
        r#"
        SELECT 
            SUM(batch_weight_kg) as monthly_volume
        FROM material_passport
        WHERE supplier_id = $1
        GROUP BY DATE_TRUNC('month', created_at)
        ORDER BY DATE_TRUNC('month', created_at)
        LIMIT 12
        "#,
    )
    .bind(supplier_id)
    .fetch_all(pool)
    .await?;

    // Get average monthly flow
    let avg_monthly_flow: rust_decimal::Decimal = sqlx::query_scalar(
        r#"
        SELECT COALESCE(AVG(monthly_volume), 0) as avg_flow
        FROM (
            SELECT SUM(batch_weight_kg) as monthly_volume
            FROM material_passport
            WHERE supplier_id = $1
            GROUP BY DATE_TRUNC('month', created_at)
        ) monthly
        "#,
    )
    .bind(supplier_id)
    .fetch_one(pool)
    .await?;

    // Get first transaction date
    let first_transaction: Option<DateTime<Utc>> = sqlx::query_scalar(
        r#"
        SELECT MIN(created_at) as first_transaction
        FROM material_passport
        WHERE supplier_id = $1
        "#,
    )
    .bind(supplier_id)
    .fetch_one(pool)
    .await?;

    Ok(SupplierMetrics {
        total_handshakes: handshake_stats.0,
        confirmed_handshakes: handshake_stats.1,
        disputed_transactions: handshake_stats.2,
        on_time_payments: payment_stats.0,
        delayed_payments: payment_stats.1,
        monthly_volumes,
        first_transaction_date: first_transaction,
        avg_monthly_flow,
    })
}

/// Generate complete scoring output for a supplier
pub async fn generate_scoring_output(
    pool: &PgPool,
    supplier_id: uuid::Uuid,
) -> Result<ScoringOutput> {
    let metrics = fetch_supplier_metrics(pool, supplier_id).await?;
    let weights = ScoringWeights::default();

    let ics_score = compute_ics_score(&metrics, &weights);
    let risk_grade = compute_risk_grade(ics_score);
    let default_probability = compute_default_probability(&metrics);
    let stability_index = compute_stability_index(&metrics);
    let credit_recommendation = compute_credit_recommendation(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ics_score_calculation() {
        let metrics = SupplierMetrics {
            total_handshakes: 100,
            confirmed_handshakes: 90,
            on_time_payments: 85,
            delayed_payments: 15,
            disputed_transactions: 5,
            monthly_volumes: vec![
                Decimal::new(1000, 0),
                Decimal::new(1100, 0),
                Decimal::new(950, 0),
                Decimal::new(1050, 0),
            ],
            first_transaction_date: Some(Utc::now() - Duration::days(365)),
            avg_monthly_flow: Decimal::new(1000, 0),
        };

        let weights = ScoringWeights::default();
        let score = compute_ics_score(&metrics, &weights);

        assert!(score >= 300 && score <= 900);
    }

    #[test]
    fn test_risk_grade_mapping() {
        assert_eq!(compute_risk_grade(800), "A");
        assert_eq!(compute_risk_grade(700), "B");
        assert_eq!(compute_risk_grade(600), "C");
        assert_eq!(compute_risk_grade(500), "D");
        assert_eq!(compute_risk_grade(400), "E");
    }
}
