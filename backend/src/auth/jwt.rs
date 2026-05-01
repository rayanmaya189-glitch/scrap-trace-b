use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,          // User ID (phone)
    pub exp: usize,           // Expiration time
    pub iat: usize,           // Issued at
    pub jti: String,          // JWT ID for idempotency
    pub role: Option<String>, // User role
    pub permissions: Vec<String>,
}

impl Claims {
    pub fn new(
        user_id: String,
        role: Option<String>,
        permissions: Vec<String>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as usize;

        Claims {
            sub: user_id,
            exp: now + 86400, // 24 hours
            iat: now,
            jti: Uuid::new_v4().to_string(),
            role,
            permissions,
        }
    }

    pub fn validate_exp(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as usize;
        self.exp > now
    }
}

pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    secret: String,
}

impl JwtManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            // Generate a secure random secret if not provided
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
            hex::encode(bytes)
        });

        Ok(JwtManager {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            secret,
        })
    }

    pub fn generate_access_token(
        &self,
        user_id: String,
        role: Option<String>,
        permissions: Vec<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let claims = Claims::new(user_id, role, permissions);
        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        Ok(token)
    }

    pub fn generate_refresh_token(&self, user_id: String) -> Result<String, Box<dyn std::error::Error>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as usize;

        let refresh_claims = Claims {
            sub: user_id,
            exp: now + (86400 * 30), // 30 days
            iat: now,
            jti: Uuid::new_v4().to_string(),
            role: None,
            permissions: vec![],
        };

        let token = encode(&Header::default(), &refresh_claims, &self.encoding_key)?;
        Ok(token)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
        let validation = Validation::default();
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        
        // Additional validation
        if !token_data.claims.validate_exp() {
            return Err("Token expired".into());
        }

        Ok(token_data.claims)
    }

    pub fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let claims = self.validate_token(refresh_token)?;
        
        // Generate new access token
        let new_access = self.generate_access_token(
            claims.sub.clone(),
            claims.role.clone(),
            claims.permissions.clone(),
        )?;

        // Generate new refresh token (rotation)
        let new_refresh = self.generate_refresh_token(claims.sub)?;

        Ok((new_access, new_refresh))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generation_and_validation() {
        let manager = JwtManager::new().unwrap();
        
        let token = manager
            .generate_access_token(
                "9876543210".to_string(),
                Some("dealer".to_string()),
                vec!["materials:create".to_string()],
            )
            .unwrap();

        let claims = manager.validate_token(&token).unwrap();
        
        assert_eq!(claims.sub, "9876543210");
        assert_eq!(claims.role, Some("dealer".to_string()));
        assert!(claims.validate_exp());
    }

    #[test]
    fn test_token_expiration() {
        let manager = JwtManager::new().unwrap();
        
        let token = manager
            .generate_access_token(
                "9876543210".to_string(),
                None,
                vec![],
            )
            .unwrap();

        let claims = manager.validate_token(&token).unwrap();
        assert!(claims.validate_exp());
    }
}
