//! Cryptographic utilities
//!
//! Provides AES-GCM encryption, secure key generation, and cryptographic operations
//! for securing sensitive data at rest and in transit.

use crate::constants::*;
use crate::error_ext::ErrorContext;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHasher, password_hash::{SaltString, PasswordHash, PasswordVerifier}};
use mcb_domain::error::{Error, Result};
use rand::RngCore;
use sha2::{Sha256, Digest};
use std::fmt;

/// Encryption/decryption service
#[derive(Clone)]
pub struct CryptoService {
    /// Master key for encryption operations
    master_key: Vec<u8>,
}

impl CryptoService {
    /// Create a new crypto service with the provided master key
    pub fn new(master_key: Vec<u8>) -> Result<Self> {
        if master_key.len() != AES_GCM_KEY_SIZE {
            return Err(Error::Configuration {
                message: format!(
                    "Invalid master key size: expected {} bytes, got {}",
                    AES_GCM_KEY_SIZE,
                    master_key.len()
                ),
                source: None,
            });
        }

        Ok(Self { master_key })
    }

    /// Generate a random master key
    pub fn generate_master_key() -> Vec<u8> {
        let mut key = vec![0u8; AES_GCM_KEY_SIZE];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Encrypt data using AES-GCM
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| Error::Infrastructure {
                message: format!("Encryption failed: {}", e),
                source: Some(Box::new(e)),
            })?;

        Ok(EncryptedData {
            ciphertext,
            nonce: nonce.to_vec(),
        })
    }

    /// Decrypt data using AES-GCM
    pub fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>> {
        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&encrypted_data.nonce);

        cipher
            .decrypt(nonce, encrypted_data.ciphertext.as_ref())
            .map_err(|e| Error::Infrastructure {
                message: format!("Decryption failed: {}", e),
                source: Some(Box::new(e)),
            })
    }

    /// Generate a secure random nonce
    pub fn generate_nonce() -> Vec<u8> {
        let mut nonce = vec![0u8; AES_GCM_NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }

    /// Compute SHA-256 hash of data
    pub fn sha256(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Compute SHA-256 hash of data as hex string
    pub fn sha256_hex(data: &[u8]) -> String {
        hex::encode(Self::sha256(data))
    }
}

/// Encrypted data container
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedData {
    /// The encrypted ciphertext
    pub ciphertext: Vec<u8>,
    /// The nonce used for encryption
    pub nonce: Vec<u8>,
}

impl fmt::Display for EncryptedData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EncryptedData {{ ciphertext: {} bytes, nonce: {} bytes }}",
            self.ciphertext.len(),
            self.nonce.len()
        )
    }
}

/// Password hashing service using Argon2
#[derive(Clone)]
pub struct PasswordService {
    /// Argon2 configuration
    argon2: Argon2<'static>,
}

impl PasswordService {
    /// Create a new password service with default configuration
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }

    /// Hash a password using Argon2
    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);

        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| Error::Infrastructure {
                message: format!("Password hashing failed: {}", e),
                source: Some(Box::new(e)),
            })?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against its hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| Error::Authentication {
                message: format!("Invalid password hash format: {}", e),
                source: Some(Box::new(e)),
            })?;

        Ok(self
            .argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

impl Default for PasswordService {
    fn default() -> Self {
        Self::new()
    }
}

/// Secure token generation
pub struct TokenGenerator;

impl TokenGenerator {
    /// Generate a cryptographically secure random token
    pub fn generate_secure_token(length: usize) -> String {
        let mut bytes = vec![0u8; length];
        OsRng.fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    /// Generate a URL-safe secure token
    pub fn generate_url_safe_token(length: usize) -> String {
        let mut bytes = vec![0u8; length];
        OsRng.fill_bytes(&mut bytes);
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    }

    /// Generate a UUID v4
    pub fn generate_uuid() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

/// Key derivation utilities
pub struct KeyDerivation;

impl KeyDerivation {
    /// Derive a key from password using PBKDF2
    pub fn pbkdf2(password: &str, salt: &[u8], iterations: u32, key_len: usize) -> Vec<u8> {
        use pbkdf2::pbkdf2;
        use sha2::Sha256;

        let mut key = vec![0u8; key_len];
        pbkdf2::<Sha256>(password.as_bytes(), salt, iterations, &mut key);
        key
    }

    /// Generate a random salt
    pub fn generate_salt(length: usize) -> Vec<u8> {
        let mut salt = vec![0u8; length];
        OsRng.fill_bytes(&mut salt);
        salt
    }
}

/// Secure data erasure
pub struct SecureErasure;

impl SecureErasure {
    /// Overwrite data with zeros
    pub fn zeroize(data: &mut [u8]) {
        data.iter_mut().for_each(|b| *b = 0);
    }

    /// Overwrite data with random bytes then zeros (secure erase)
    pub fn secure_erase(data: &mut [u8]) {
        // Overwrite with random data
        OsRng.fill_bytes(data);
        // Overwrite with zeros
        Self::zeroize(data);
    }

    /// Securely erase a string by overwriting its buffer
    pub fn erase_string(s: &mut String) {
        unsafe {
            let bytes = s.as_bytes_mut();
            Self::secure_erase(bytes);
        }
        s.clear();
    }
}

/// Cryptographic hash utilities
pub struct HashUtils;

impl HashUtils {
    /// Compute HMAC-SHA256
    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
        use hmac::{Hmac, Mac};
        let mut mac = Hmac::<Sha256>::new_from_slice(key)
            .expect("HMAC key size should be valid");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    /// Constant-time comparison for cryptographic values
    pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut result = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }
        result == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_service_encrypt_decrypt() {
        let master_key = CryptoService::generate_master_key();
        let service = CryptoService::new(master_key).unwrap();

        let plaintext = b"Hello, World!";
        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_crypto_service_invalid_key_size() {
        let invalid_key = vec![0u8; 16]; // Wrong size
        assert!(CryptoService::new(invalid_key).is_err());
    }

    #[test]
    fn test_password_service_hash_verify() {
        let service = PasswordService::new();

        let password = "test_password_123";
        let hash = service.hash_password(password).unwrap();

        assert!(service.verify_password(password, &hash).unwrap());
        assert!(!service.verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_token_generator() {
        let token1 = TokenGenerator::generate_secure_token(32);
        let token2 = TokenGenerator::generate_secure_token(32);

        assert_eq!(token1.len(), 64); // 32 bytes * 2 hex chars
        assert_eq!(token2.len(), 64);
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_hash_utils_hmac() {
        let key = b"secret_key";
        let data = b"test_data";
        let hmac1 = HashUtils::hmac_sha256(key, data);
        let hmac2 = HashUtils::hmac_sha256(key, data);

        assert_eq!(hmac1, hmac2);
        assert_eq!(hmac1.len(), 32); // SHA256 output size
    }

    #[test]
    fn test_secure_erasure() {
        let mut data = vec![1, 2, 3, 4, 5];
        SecureErasure::zeroize(&mut data);
        assert_eq!(data, vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(HashUtils::constant_time_eq(b"test", b"test"));
        assert!(!HashUtils::constant_time_eq(b"test", b"different"));
        assert!(!HashUtils::constant_time_eq(b"test", b"test_longer"));
    }
}