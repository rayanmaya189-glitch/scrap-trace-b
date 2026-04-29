use sqlx::PgPool;
use uuid::Uuid;
use rust_decimal::Decimal;
use crate::models::MaterialPassport;

pub struct MaterialRepository {
    pool: PgPool,
}

impl MaterialRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        material_type: &str,
        batch_weight_kg: Decimal,
        material_grade: &str,
        source_pincode: &str,
        supplier_id: Uuid,
        metadata: Option<serde_json::Value>,
        cbam_fields: Option<serde_json::Value>,
    ) -> Result<MaterialPassport, sqlx::Error> {
        let material = sqlx::query_as::<_, MaterialPassport>(
            r#"
            INSERT INTO material_passport 
                (material_type, batch_weight_kg, material_grade, source_pincode, supplier_id, metadata, cbam_fields)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(material_type)
        .bind(batch_weight_kg)
        .bind(material_grade)
        .bind(source_pincode)
        .bind(supplier_id)
        .bind(metadata.unwrap_or_else(|| serde_json::json!({})))
        .bind(cbam_fields.unwrap_or_else(|| serde_json::json!({})))
        .fetch_one(&self.pool)
        .await?;

        Ok(material)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<MaterialPassport>, sqlx::Error> {
        sqlx::query_as::<_, MaterialPassport>("SELECT * FROM material_passport WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn exists(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM material_passport WHERE id = $1)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    pub async fn update_status(&self, id: Uuid, status: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE material_passport SET status = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(status)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn assign_buyer(&self, id: Uuid, buyer_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE material_passport SET buyer_id = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(buyer_id)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_by_supplier(
        &self,
        supplier_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<MaterialPassport>, i64), sqlx::Error> {
        let materials: Vec<MaterialPassport> = sqlx::query_as::<_, MaterialPassport>(
            "SELECT * FROM material_passport WHERE supplier_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(supplier_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM material_passport WHERE supplier_id = $1",
        )
        .bind(supplier_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((materials, total))
    }

    pub async fn list(
        &self,
        limit: i64,
        offset: i64,
        status_filter: Option<&str>,
    ) -> Result<(Vec<MaterialPassport>, i64), sqlx::Error> {
        let (materials, total) = match status_filter {
            Some(status) => {
                let materials: Vec<MaterialPassport> = sqlx::query_as::<_, MaterialPassport>(
                    "SELECT * FROM material_passport WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                )
                .bind(status)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?;

                let total: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM material_passport WHERE status = $1",
                )
                .bind(status)
                .fetch_one(&self.pool)
                .await?;

                (materials, total)
            }
            None => {
                let materials: Vec<MaterialPassport> = sqlx::query_as::<_, MaterialPassport>(
                    "SELECT * FROM material_passport ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?;

                let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM material_passport")
                    .fetch_one(&self.pool)
                    .await?;

                (materials, total)
            }
        };

        Ok((materials, total))
    }
}
