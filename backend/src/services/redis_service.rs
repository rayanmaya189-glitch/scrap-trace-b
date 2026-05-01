use redis::{Client, ConnectionManager, aio::ConnectionLike};
use std::time::Duration;
use tracing::{info, warn};

pub struct RedisManager {
    client: Client,
}

impl RedisManager {
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::open(redis_url)?;
        Ok(RedisManager { client })
    }

    pub async fn get_connection(&self) -> Result<ConnectionManager, Box<dyn std::error::Error>> {
        let manager = ConnectionManager::new(self.client.clone()).await?;
        Ok(manager)
    }

    /// Store OTP with 5 minute expiration
    pub async fn store_otp(
        &self,
        phone: &str,
        otp: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("otp:{}", phone);
        
        // Set with 5 minute TTL
        redis::cmd("SET")
            .arg(&key)
            .arg(otp)
            .arg("EX")
            .arg(300) // 5 minutes
            .query_async(&mut conn)
            .await?;
        
        info!("OTP stored for phone: {}", phone);
        Ok(())
    }

    /// Retrieve OTP
    pub async fn get_otp(&self, phone: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("otp:{}", phone);
        
        let otp: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        
        Ok(otp)
    }

    /// Delete OTP after successful verification
    pub async fn delete_otp(&self, phone: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("otp:{}", phone);
        
        redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        
        info!("OTP deleted for phone: {}", phone);
        Ok(())
    }

    /// Store refresh token blacklist (for logout)
    pub async fn blacklist_token(
        &self,
        token: &str,
        ttl_seconds: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("blacklist:{}", token);
        
        redis::cmd("SET")
            .arg(&key)
            .arg("1")
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }

    /// Check if token is blacklisted
    pub async fn is_token_blacklisted(&self, token: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("blacklist:{}", token);
        
        let exists: bool = redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        
        Ok(exists)
    }

    /// Rate limiting: Track request count per IP/phone
    pub async fn increment_rate_limit(
        &self,
        key: &str,
        limit: usize,
        window_seconds: usize,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let redis_key = format!("ratelimit:{}", key);
        
        // Increment counter
        let count: i32 = redis::cmd("INCR")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await?;
        
        // Set TTL on first request
        if count == 1 {
            redis::cmd("EXPIRE")
                .arg(&redis_key)
                .arg(window_seconds)
                .query_async(&mut conn)
                .await?;
        }
        
        Ok(count as usize <= limit)
    }

    /// Store user session data
    pub async fn store_session(
        &self,
        user_id: &str,
        data: &str,
        ttl_seconds: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("session:{}", user_id);
        
        redis::cmd("SET")
            .arg(&key)
            .arg(data)
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }

    /// Get user session data
    pub async fn get_session(&self, user_id: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("session:{}", user_id);
        
        let data: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        
        Ok(data)
    }

    /// Cache supplier score with TTL
    pub async fn cache_score(
        &self,
        supplier_id: &str,
        score_data: &str,
        ttl_seconds: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("score:{}", supplier_id);
        
        redis::cmd("SET")
            .arg(&key)
            .arg(score_data)
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }

    /// Get cached supplier score
    pub async fn get_cached_score(&self, supplier_id: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("score:{}", supplier_id);
        
        let data: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        
        Ok(data)
    }

    /// Invalidate score cache
    pub async fn invalidate_score_cache(&self, supplier_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("score:{}", supplier_id);
        
        redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running Redis
    async fn test_otp_storage() {
        let manager = RedisManager::new("redis://localhost").unwrap();
        
        manager.store_otp("9876543210", "123456").await.unwrap();
        let otp = manager.get_otp("9876543210").await.unwrap();
        
        assert_eq!(otp, Some("123456".to_string()));
        
        manager.delete_otp("9876543210").await.unwrap();
        let otp_after_delete = manager.get_otp("9876543210").await.unwrap();
        
        assert_eq!(otp_after_delete, None);
    }
}
