//! Scoring repository - Database access for scoring metrics

use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use super::metrics::SupplierMetrics;
use crate::error::{Result, AppError};

/// Fetch supplier metrics from database
pub async fn fetch_supplier_metrics(pool: &PgPool, supplier_id: Uuid) -> Result<SupplierMetrics> {
    let handshake_stats = fetch_handshake_stats(pool, supplier_id).await?;
    let payment_stats = fetch_payment_stats(pool, supplier_id).await?;
    let monthly_volumes = fetch_monthly_volumes(pool, supplier_id).await?;
    let avg_monthly_flow = fetch_avg_monthly_flow(pool, supplier_id).await?;
    let first_transaction = fetch_first_transaction_date(pool, supplier_id).await?;

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

async fn fetch_handshake_stats(
    pool: &PgPool,
    supplier_id: Uuid,
) -> Result<(i64, i64, i64)> {
    sqlx::query_as::<_, (i64, i64, i64)>(
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
    .await
    .map_err(|e| AppError::Database(e))
}

async fn fetch_payment_stats(
    pool: &PgPool,
    supplier_id: Uuid,
) -> Result<(i64, i64)> {
    sqlx::query_as::<_, (i64, i64)>(
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
    .await
    .map_err(|e| AppError::Database(e))
}

async fn fetch_monthly_volumes(
    pool: &PgPool,
    supplier_id: Uuid,
) -> Result<Vec<Decimal>> {
    sqlx::query_scalar::<_, Decimal>(
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
    .await
    .map_err(|e| AppError::Database(e))
}

async fn fetch_avg_monthly_flow(
    pool: &PgPool,
    supplier_id: Uuid,
) -> Result<Decimal> {
    sqlx::query_scalar::<_, Decimal>(
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
    .await
    .map_err(|e| AppError::Database(e))
}

async fn fetch_first_transaction_date(
    pool: &PgPool,
    supplier_id: Uuid,
) -> Result<Option<DateTime<Utc>>> {
    sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
        r#"
        SELECT MIN(created_at) as first_transaction
        FROM material_passport
        WHERE supplier_id = $1
        "#,
    )
    .bind(supplier_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Database(e))
}
