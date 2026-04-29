use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use crate::models::SupplierProfile;

pub struct SupplierRepository {
    pool: PgPool,
}

impl SupplierRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        phone: &str,
        role: &str,
        name: Option<&str>,
        business_name: Option<&str>,
        pincode: Option<&str>,
        gst_number: Option<&str>,
    ) -> Result<SupplierProfile, sqlx::Error> {
        let supplier = sqlx::query_as::<_, SupplierProfile>(
            r#"
            INSERT INTO supplier_profile (phone, role, name, business_name, pincode, gst_number)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(phone)
        .bind(role)
        .bind(name)
        .bind(business_name)
        .bind(pincode)
        .bind(gst_number)
        .fetch_one(&self.pool)
        .await?;

        Ok(supplier)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<SupplierProfile>, sqlx::Error> {
        sqlx::query_as::<_, SupplierProfile>("SELECT * FROM supplier_profile WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn find_by_phone(&self, phone: &str) -> Result<Option<SupplierProfile>, sqlx::Error> {
        sqlx::query_as::<_, SupplierProfile>("SELECT * FROM supplier_profile WHERE phone = $1")
            .bind(phone)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn exists(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM supplier_profile WHERE id = $1)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    pub async fn list(
        &self,
        limit: i64,
        offset: i64,
        role_filter: Option<&str>,
    ) -> Result<(Vec<SupplierProfile>, i64), sqlx::Error> {
        let mut query = String::from("SELECT * FROM supplier_profile");
        let mut params: Vec<&(dyn sqlx::Encode<sqlx::Postgres> + Send)> = Vec::new();
        
        if let Some(role) = role_filter {
            query.push_str(&format!(" WHERE role = ${}", params.len() + 1));
            params.push(role);
        }

        query.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            params.len() + 1,
            params.len() + 2
        ));

        let count_query = match role_filter {
            Some(_) => "SELECT COUNT(*) FROM supplier_profile WHERE role = $1",
            None => "SELECT COUNT(*) FROM supplier_profile",
        };

        let suppliers: Vec<SupplierProfile> = if role_filter.is_some() {
            sqlx::query_as_with(&query, (params[0], limit, offset))
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query_as_with(&query, (limit, offset))
                .fetch_all(&self.pool)
                .await?
        };

        let total: i64 = if role_filter.is_some() {
            sqlx::query_scalar(count_query)
                .bind(role_filter.unwrap())
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_scalar(count_query)
                .fetch_one(&self.pool)
                .await?
        };

        Ok((suppliers, total))
    }

    pub async fn update_verification(&self, id: Uuid, is_verified: bool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE supplier_profile SET is_verified = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(is_verified)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
