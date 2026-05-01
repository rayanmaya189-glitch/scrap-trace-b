use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use hex;
use uuid::Uuid;

/// Device key pair for cryptographic signatures
#[derive(Clone)]
pub struct DeviceKeyPair {
    pub device_id: String,
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl DeviceKeyPair {
    /// Generate a new device key pair
    pub fn generate() -> Result<Self, Box<dyn std::error::Error>> {
        use rand::rngs::OsRng;
        
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let device_id = Uuid::new_v4().to_string();
        
        Ok(DeviceKeyPair {
            device_id,
            signing_key,
            verifying_key,
        })
    }
    
    /// Create from existing keys (for loading from storage)
    pub fn from_keys(
        device_id: String,
        signing_key_bytes: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let signing_key = SigningKey::from_bytes(signing_key_bytes.try_into()?);
        let verifying_key = signing_key.verifying_key();
        
        Ok(DeviceKeyPair {
            device_id,
            signing_key,
            verifying_key,
        })
    }
    
    /// Sign a payload hash
    pub fn sign(&self, payload_hash: &[u8]) -> Signature {
        self.signing_key.sign(payload_hash)
    }
    
    /// Sign and return hex-encoded signature
    pub fn sign_hex(&self, payload_hash: &[u8]) -> String {
        let sig = self.sign(payload_hash);
        hex::encode(sig.to_bytes())
    }
    
    /// Export signing key as bytes (for secure storage)
    pub fn export_signing_key(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
    
    /// Export verifying key as bytes
    pub fn export_verifying_key(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }
}

/// Verify a signature against a payload hash
pub fn verify_signature(
    signature_hex: &str,
    payload_hash: &[u8],
    public_key_hex: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let signature_bytes = hex::decode(signature_hex)?;
    let signature = Signature::from_bytes(&signature_bytes.try_into()?);
    
    let public_key_bytes = hex::decode(public_key_hex)?;
    let public_key = VerifyingKey::from_bytes(&public_key_bytes.try_into()?)?;
    
    match public_key.verify(payload_hash, &signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Compute SHA256 hash of data
pub fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Compute hash for handshake payload
pub fn compute_payload_hash(
    material_id: &str,
    from_party: &str,
    to_party: &str,
    timestamp: &str,
    data: &str,
) -> String {
    let combined = format!("{}|{}|{}|{}|{}", material_id, from_party, to_party, timestamp, data);
    compute_hash(combined.as_bytes())
}

/// Validate hash chain integrity
/// Returns true if hash_prev matches the previous handshake's hash_current
pub fn validate_hash_chain(
    current_hash_prev: &str,
    previous_hash_current: &str,
) -> bool {
    current_hash_prev == previous_hash_current
}

/// Generate fingerprint for a device/browser
pub fn generate_device_fingerprint(
    user_agent: &str,
    screen_resolution: &str,
    timezone: &str,
    language: &str,
) -> String {
    let fingerprint_data = format!(
        "{}|{}|{}|{}",
        user_agent, screen_resolution, timezone, language
    );
    compute_hash(fingerprint_data.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_key_generation() {
        let keypair = DeviceKeyPair::generate().unwrap();
        
        assert!(!keypair.device_id.is_empty());
        assert_eq!(keypair.export_signing_key().len(), 32);
        assert_eq!(keypair.export_verifying_key().len(), 32);
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = DeviceKeyPair::generate().unwrap();
        
        let payload = b"test payload";
        let payload_hash = compute_hash(payload);
        let signature = keypair.sign_hex(payload_hash.as_bytes());
        
        let public_key_hex = hex::encode(keypair.export_verifying_key());
        let is_valid = verify_signature(&signature, payload_hash.as_bytes(), &public_key_hex).unwrap();
        
        assert!(is_valid);
    }

    #[test]
    fn test_hash_chain_validation() {
        let prev_hash_current = "abc123def456";
        let current_hash_prev = "abc123def456";
        let invalid_hash_prev = "xyz789";
        
        assert!(validate_hash_chain(current_hash_prev, prev_hash_current));
        assert!(!validate_hash_chain(invalid_hash_prev, prev_hash_current));
    }

    #[test]
    fn test_device_fingerprint() {
        let fp1 = generate_device_fingerprint(
            "Mozilla/5.0",
            "1920x1080",
            "UTC+5:30",
            "en-US"
        );
        
        let fp2 = generate_device_fingerprint(
            "Mozilla/5.0",
            "1920x1080",
            "UTC+5:30",
            "en-US"
        );
        
        let fp3 = generate_device_fingerprint(
            "Mozilla/5.1",
            "1920x1080",
            "UTC+5:30",
            "en-US"
        );
        
        assert_eq!(fp1, fp2); // Same inputs = same fingerprint
        assert_ne!(fp1, fp3); // Different inputs = different fingerprint
    }
}
