//! Encryption/decryption service using AES-GCM

use crate::constants::*;
use aes_gcm::{
    aead::{rand_core::RngCore as AeadRngCore, Aead, AeadCore, KeyInit, OsRng as AeadOsRng},
    Aes256Gcm, Key, Nonce,
};
use mcb_application::ports::providers::{CryptoProvider, EncryptedData};
use mcb_domain::error::{Error, Result};
use sha2::{Digest, Sha256};

use super::utils::bytes_to_hex;

/// Encryption/decryption service
///
/// Implements the CryptoProvider port from mcb-domain.
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
        AeadOsRng.fill_bytes(&mut key);
        key
    }

    /// Encrypt data using AES-GCM
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut AeadOsRng);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| Error::Infrastructure {
                message: format!("Encryption failed: {}", e),
                source: None,
            })?;

        Ok(EncryptedData::new(ciphertext, nonce.to_vec()))
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
                source: None,
            })
    }

    /// Generate a secure random nonce
    pub fn generate_nonce() -> Vec<u8> {
        let mut nonce = vec![0u8; AES_GCM_NONCE_SIZE];
        AeadOsRng.fill_bytes(&mut nonce);
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
        bytes_to_hex(&Self::sha256(data))
    }
}

// Implement the CryptoProvider port from mcb-domain
impl CryptoProvider for CryptoService {
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        // Delegate to inherent method
        CryptoService::encrypt(self, plaintext)
    }

    fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>> {
        // Delegate to inherent method
        CryptoService::decrypt(self, encrypted_data)
    }

    fn provider_name(&self) -> &str {
        "aes-256-gcm"
    }
}
