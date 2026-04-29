use crate::models::ScoringOutput;

pub struct ScoringService;

impl ScoringService {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_ics_score(
        verification_reliability: f64,
        payment_timeliness: f64,
        volume_consistency: f64,
        dispute_rate: f64,
        network_tenure_months: i32,
    ) -> i32 {
        let normalized_tenure = (network_tenure_months.min(36) as f64) / 36.0;

        let weighted_sum = (0.30 * verification_reliability.clamp(0.0, 1.0))
            + (0.25 * payment_timeliness.clamp(0.0, 1.0))
            + (0.20 * volume_consistency.clamp(0.0, 1.0))
            - (0.15 * dispute_rate.clamp(0.0, 1.0))
            + (0.10 * normalized_tenure);

        let score = 500.0 + (weighted_sum * 400.0);
        score.clamp(300.0, 900.0) as i32
    }

    pub fn determine_risk_grade(ics_score: i32, compliance_adjustment: i32) -> String {
        let adjusted_score = ics_score + compliance_adjustment;
        
        match adjusted_score {
            s if s >= 750 => "A".to_string(),
            s if s >= 650 => "B".to_string(),
            s if s >= 550 => "C".to_string(),
            s if s >= 450 => "D".to_string(),
            _ => "E".to_string(),
        }
    }

    pub fn calculate_default_probability(
        base_pd: f64,
        avg_delay_days: f64,
        dispute_ratio: f64,
        concentration_ratio: f64,
    ) -> f64 {
        let beta1 = 0.02;
        let beta2 = 0.15;
        let beta3 = 0.08;

        let exponent = (beta1 * avg_delay_days) + (beta2 * dispute_ratio) + (beta3 * concentration_ratio);
        let pd = base_pd * exponent.exp();

        pd.clamp(0.001, 0.45)
    }

    pub fn calculate_stability_index(
        redundancy_score: f64,
        geo_dispersion_score: f64,
        reliability_score: f64,
    ) -> f64 {
        let stability = (0.4 * redundancy_score.clamp(0.0, 1.0))
            + (0.3 * geo_dispersion_score.clamp(0.0, 1.0))
            + (0.3 * reliability_score.clamp(0.0, 1.0));

        (stability * 100.0).clamp(0.0, 100.0)
    }

    pub fn calculate_credit_limit(verified_monthly_flow: f64, ics_score: i32) -> f64 {
        let multiplier = match ics_score {
            s if s >= 750 => 3.5,
            s if s >= 650 => 2.0,
            s if s >= 550 => 1.0,
            s if s >= 450 => 0.5,
            _ => 0.0,
        };

        verified_monthly_flow * multiplier
    }

    pub fn calculate_pricing(
        base_rate: f64,
        default_probability: f64,
    ) -> f64 {
        let spread = default_probability * 100.0 * 2.5;
        let final_rate = base_rate + spread + 0.5;

        final_rate.min(36.0)
    }

    pub fn generate_scoring_output(
        supplier_id: uuid::Uuid,
        verification_reliability: f64,
        payment_timeliness: f64,
        volume_consistency: f64,
        dispute_rate: f64,
        network_tenure_months: i32,
        avg_delay_days: f64,
        concentration_ratio: f64,
        redundancy_score: f64,
        geo_dispersion_score: f64,
        reliability_score: f64,
        verified_monthly_flow: f64,
        base_rate: f64,
        base_pd: f64,
        compliance_adjustment: i32,
    ) -> ScoringParams {
        let ics_score = Self::calculate_ics_score(
            verification_reliability,
            payment_timeliness,
            volume_consistency,
            dispute_rate,
            network_tenure_months,
        );

        let risk_grade = Self::determine_risk_grade(ics_score, compliance_adjustment);

        let default_probability_90d = Self::calculate_default_probability(
            base_pd,
            avg_delay_days,
            dispute_rate,
            concentration_ratio,
        );

        let default_probability_180d = (default_probability_90d * 1.5).min(0.45);

        let stability_index = Self::calculate_stability_index(
            redundancy_score,
            geo_dispersion_score,
            reliability_score,
        );

        let recommended_limit = Self::calculate_credit_limit(verified_monthly_flow, ics_score);

        let final_rate = Self::calculate_pricing(base_rate, default_probability_90d);
        let pricing_spread = final_rate - base_rate;

        let collateral_required = ics_score < 550 || default_probability_90d > 0.30;

        ScoringParams {
            supplier_id,
            ics_score,
            risk_grade,
            default_probability_90d,
            default_probability_180d,
            stability_index,
            recommended_limit,
            pricing_spread,
            base_rate,
            final_rate,
            collateral_required,
        }
    }
}

pub struct ScoringParams {
    pub supplier_id: uuid::Uuid,
    pub ics_score: i32,
    pub risk_grade: String,
    pub default_probability_90d: f64,
    pub default_probability_180d: f64,
    pub stability_index: f64,
    pub recommended_limit: f64,
    pub pricing_spread: f64,
    pub base_rate: f64,
    pub final_rate: f64,
    pub collateral_required: bool,
}

impl Default for ScoringService {
    fn default() -> Self {
        Self::new()
    }
}
