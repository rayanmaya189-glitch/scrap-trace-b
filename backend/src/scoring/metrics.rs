//! Scoring metrics and data structures

use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

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
