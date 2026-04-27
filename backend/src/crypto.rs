//! Cryptographic utilities for digital signatures and hash chains

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use hex::{FromHex, ToHex};
use crate::error::{AppError, Result};

/// Generate a SHA-256 hash of the payload
pub fn hash_payload(payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    hasher.finalize().encode_hex()
}

/// Compute hash chain: Hₙ = SHA-256(payloadₙ + Hₙ₋₁ + device_salt + timestamp)
pub fn compute_hash_chain(
    payload: &[u8],
    prev_hash: &str,
    device_salt: &[u8],
    timestamp: u64,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    hasher.update(prev_hash.as_bytes());
    hasher.update(device_salt);
    hasher.update(&timestamp.to_be_bytes());
    hasher.finalize().encode_hex()
}

/// Generate Ed25519 keypair (for device-side storage)
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Sign a payload with Ed25519
pub fn sign_payload(signing_key: &SigningKey, payload: &[u8]) -> Result<Vec<u8>> {
    let signature = signing_key.sign(payload);
    Ok(signature.to_bytes().to_vec())
}

/// Verify an Ed25519 signature
pub fn verify_signature(
    verifying_key: &[u8],
    payload: &[u8],
    signature: &[u8],
) -> Result<bool> {
    let vk_bytes: [u8; 32] = verifying_key
        .try_into()
        .map_err(|_| AppError::Validation("Invalid verifying key length".into()))?;
    
    let sig_bytes: [u8; 64] = signature
        .try_into()
        .map_err(|_| AppError::Validation("Invalid signature length".into()))?;
    
    let verifying_key = VerifyingKey::from_bytes(&vk_bytes)
        .map_err(|e| AppError::Validation(format!("Invalid public key: {}", e)))?;
    
    let signature = Signature::from_bytes(&sig_bytes);
    
    match verifying_key.verify(payload, &signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Generate idempotency key: SHA-256(payload + device_fingerprint + timestamp)
pub fn generate_idempotency_key(
    payload: &[u8],
    device_fingerprint: &str,
    timestamp: u64,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    hasher.update(device_fingerprint.as_bytes());
    hasher.update(&timestamp.to_be_bytes());
    hasher.finalize().encode_hex()
}

/// Decode hex string to bytes
pub fn decode_hex(hex_str: &str) -> Result<Vec<u8>> {
    Vec::<u8>::from_hex(hex_str)
        .map_err(|e| AppError::Validation(format!("Invalid hex: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_payload() {
        let payload = b"test data";
        let hash = hash_payload(payload);
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[test]
    fn test_hash_chain() {
        let payload = b"transaction data";
        let prev_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        let device_salt = b"device_salt_123";
        let timestamp = 1234567890;
        
        let hash = compute_hash_chain(payload, prev_hash, device_salt, timestamp);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_sign_and_verify() {
        let (signing_key, verifying_key) = generate_keypair();
        let payload = b"test message to sign";
        
        let signature = sign_payload(&signing_key, payload).unwrap();
        let is_valid = verify_signature(
            &verifying_key.to_bytes(),
            payload,
            &signature
        ).unwrap();
        
        assert!(is_valid);
    }

    #[test]
    fn test_idempotency_key() {
        let payload = b"event payload";
        let fingerprint = "device_abc123";
        let timestamp = 1234567890;
        
        let key1 = generate_idempotency_key(payload, fingerprint, timestamp);
        let key2 = generate_idempotency_key(payload, fingerprint, timestamp);
        
        assert_eq!(key1, key2); // Same inputs produce same key
        assert_eq!(key1.len(), 64);
    }
}
