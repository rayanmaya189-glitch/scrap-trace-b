use sqlx::PgPool;
use uuid::Uuid;
use rust_decimal::Decimal;
use crate::models::ScoringOutput;

pub struct ScoringRepository {
    pool: PgPool,
}

impl ScoringRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        supplier_id: Uuid,
        ics_score: i32,
        risk_grade: &str,
        default_probability_90d: Option<Decimal>,
        default_probability_180d: Option<Decimal>,
        stability_index: Option<Decimal>,
        recommended_limit_inr: Option<Decimal>,
        pricing_spread_percent: Option<Decimal>,
        base_rate_percent: Option<Decimal>,
        final_rate_percent: Option<Decimal>,
        collateral_required: bool,
        methodology_version: &str,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<ScoringOutput, sqlx::Error> {
        let scoring = sqlx::query_as::<_, ScoringOutput>(
            r#"
            INSERT INTO scoring_output (
                supplier_id, ics_score, risk_grade, default_probability_90d, 
                default_probability_180d, stability_index, recommended_limit_inr,
                pricing_spread_percent, base_rate_percent, final_rate_percent,
                collateral_required, methodology_version, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (supplier_id) DO UPDATE SET
                ics_score = EXCLUDED.ics_score,
                risk_grade = EXCLUDED.risk_grade,
                default_probability_90d = EXCLUDED.default_probability_90d,
                default_probability_180d = EXCLUDED.default_probability_180d,
                stability_index = EXCLUDED.stability_index,
                recommended_limit_inr = EXCLUDED.recommended_limit_inr,
                pricing_spread_percent = EXCLUDED.pricing_spread_percent,
                base_rate_percent = EXCLUDED.base_rate_percent,
                final_rate_percent = EXCLUDED.final_rate_percent,
                collateral_required = EXCLUDED.collateral_required,
                methodology_version = EXCLUDED.methodology_version,
                calculated_at = NOW(),
                expires_at = EXCLUDED.expires_at
            RETURNING *
            "#,
        )
        .bind(supplier_id)
        .bind(ics_score)
        .bind(risk_grade)
        .bind(default_probability_90d)
        .bind(default_probability_180d)
        .bind(stability_index)
        .bind(recommended_limit_inr)
        .bind(pricing_spread_percent)
        .bind(base_rate_percent)
        .bind(final_rate_percent)
        .bind(collateral_required)
        .bind(methodology_version)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(scoring)
    }

    pub async fn find_by_supplier_id(&self, supplier_id: Uuid) -> Result<Option<ScoringOutput>, sqlx::Error> {
        sqlx::query_as::<_, ScoringOutput>(
            "SELECT * FROM scoring_output WHERE supplier_id = $1",
        )
        .bind(supplier_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn exists_for_supplier(&self, supplier_id: Uuid) -> Result<bool, sqlx::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM scoring_output WHERE supplier_id = $1)",
        )
        .bind(supplier_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }
}
