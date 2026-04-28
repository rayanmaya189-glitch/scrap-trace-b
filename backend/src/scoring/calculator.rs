//! Scoring calculator - Core scoring algorithms

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use chrono::{DateTime, Utc, Duration};
use super::metrics::{ScoringWeights, SupplierMetrics};
use crate::models::{DefaultProbability, CreditRecommendation};

/// Compute Industrial Credit Score (ICS)
/// Formula: 500 + Σ(wᵢ × norm(xᵢ)), clamped 300–900
pub fn compute_ics_score(metrics: &SupplierMetrics, weights: &ScoringWeights) -> i32 {
    let verification_normalized = compute_verification_ratio(metrics) * 100.0;
    let timeliness_normalized = compute_timeliness_ratio(metrics) * 100.0;
    let volume_normalized = compute_volume_consistency(&metrics.monthly_volumes) * 100.0;
    let dispute_normalized = compute_dispute_ratio(metrics);
    let tenure_normalized = compute_tenure_normalized(metrics.first_transaction_date);

    let score = 500.0
        + (weights.verification_reliability * (verification_normalized - 50.0) * 2.0)
        + (weights.payment_timeliness * (timeliness_normalized - 50.0) * 2.0)
        + (weights.volume_consistency * (volume_normalized - 50.0) * 2.0)
        + (weights.dispute_rate * (dispute_normalized - 50.0) * 2.0)
        + (weights.network_tenure * (tenure_normalized - 50.0) * 2.0);

    score.round() as i32
        .max(300)
        .min(900)
}

fn compute_verification_ratio(metrics: &SupplierMetrics) -> f64 {
    if metrics.total_handshakes > 0 {
        metrics.confirmed_handshakes as f64 / metrics.total_handshakes as f64
    } else {
        0.0
    }
}

fn compute_timeliness_ratio(metrics: &SupplierMetrics) -> f64 {
    let total_payments = metrics.on_time_payments + metrics.delayed_payments;
    if total_payments > 0 {
        metrics.on_time_payments as f64 / total_payments as f64
    } else {
        0.5
    }
}

/// Compute volume consistency using coefficient of variation
pub fn compute_volume_consistency(volumes: &[Decimal]) -> f64 {
    if volumes.is_empty() || volumes.len() < 2 {
        return 0.5;
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
    let cv = std_dev / mean;
    (1.0 - cv.min(1.0)).max(0.0)
}

fn compute_dispute_ratio(metrics: &SupplierMetrics) -> f64 {
    if metrics.total_handshakes > 0 {
        (1.0 - (metrics.disputed_transactions as f64 / metrics.total_handshakes as f64)) * 100.0
    } else {
        100.0
    }
}

/// Compute tenure in months from first transaction
pub fn compute_tenure_months(first_date: Option<DateTime<Utc>>) -> i32 {
    match first_date {
        Some(date) => {
            let now = Utc::now();
            let duration = now.signed_duration_since(date);
            (duration.num_days() / 30) as i32
        }
        None => 0,
    }
}

fn compute_tenure_normalized(first_date: Option<DateTime<Utc>>) -> f64 {
    let tenure_months = compute_tenure_months(first_date);
    (tenure_months.min(36) as f64 / 36.0) * 100.0
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
pub fn compute_default_probability(metrics: &SupplierMetrics) -> DefaultProbability {
    let base_pd = 0.05;
    let delay_ratio = compute_delay_ratio(metrics);
    let dispute_ratio = compute_dispute_ratio_for_pd(metrics);
    let concentration = 1.0 - compute_volume_consistency(&metrics.monthly_volumes);

    let beta_delay = 1.5;
    let beta_disputes = 2.0;
    let beta_concentration = 0.8;

    let pd_multiplier = (beta_delay * delay_ratio
        + beta_disputes * dispute_ratio
        + beta_concentration * concentration).exp();

    let pd_90 = (base_pd * pd_multiplier).clamp(0.001, 0.450);
    let pd_180 = (pd_90 * 1.5).clamp(0.001, 0.450);

    DefaultProbability {
        day_90: pd_90,
        day_180: pd_180,
    }
}

fn compute_delay_ratio(metrics: &SupplierMetrics) -> f64 {
    let total_payments = metrics.on_time_payments + metrics.delayed_payments;
    if total_payments > 0 {
        metrics.delayed_payments as f64 / total_payments as f64
    } else {
        0.0
    }
}

fn compute_dispute_ratio_for_pd(metrics: &SupplierMetrics) -> f64 {
    if metrics.total_handshakes > 0 {
        metrics.disputed_transactions as f64 / metrics.total_handshakes as f64
    } else {
        0.0
    }
}

/// Compute Supply Chain Stability Index
pub fn compute_stability_index(metrics: &SupplierMetrics) -> f64 {
    let redundancy = (metrics.total_handshakes as f64 / 100.0).min(1.0);
    let geo_dispersion = 0.5;
    let reliability = if metrics.total_handshakes > 0 {
        metrics.confirmed_handshakes as f64 / metrics.total_handshakes as f64
    } else {
        0.0
    };

    let stability = 0.4 * redundancy + 0.3 * geo_dispersion + 0.3 * reliability;
    (stability * 100.0).round() / 100.0
}

/// Compute credit recommendation
pub fn compute_credit_recommendation(
    ics_score: i32,
    pd: &DefaultProbability,
    avg_monthly_flow: Decimal,
) -> CreditRecommendation {
    let ics_multiplier = match ics_score {
        s if s >= 750 => 3.5,
        s if s >= 650 => 2.0,
        s if s >= 550 => 1.0,
        s if s >= 450 => 0.5,
        _ => 0.25,
    };

    let monthly_flow_f64 = avg_monthly_flow.to_f64().unwrap_or(0.0);
    let recommended_limit = (monthly_flow_f64 * ics_multiplier) as u64;

    let base_rate = 10.5;
    let pd_spread = pd.day_90 * 100.0 * 2.5;
    let processing_fee = 0.5;
    let final_rate = (base_rate + pd_spread + processing_fee).min(36.0);
    let pricing_spread = final_rate - base_rate;

    let collateral_required = ics_score < 450 || pd.day_90 > 0.30;

    CreditRecommendation {
        recommended_limit_inr: recommended_limit,
        pricing_spread_percent: (pricing_spread * 100.0).round() / 100.0,
        base_rate_percent: base_rate,
        final_rate_percent: (final_rate * 100.0).round() / 100.0,
        collateral_required,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

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
