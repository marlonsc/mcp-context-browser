//! Cryptography module for encryption at rest
//!
//! Provides AES-256-GCM encryption for sensitive data stored on disk.
//! Implements envelope encryption with data keys and master keys.

#![allow(deprecated)] // Allow deprecated aes_gcm API for compatibility

use crate::core::error::{Error, Result};
use aes_gcm::{
    Aes256Gcm, Key,
    aead::{Aead, KeyInit},
};
use rand::{Rng, rng};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Encryption algorithm configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum EncryptionAlgorithm {
    #[serde(rename = "aes256-gcm")]
    #[default]
    Aes256Gcm,
}

/// Master key configuration for envelope encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterKeyConfig {
    /// Path to the master key file
    pub key_path: String,
    /// Key rotation interval in days
    pub rotation_days: u32,
}

impl Default for MasterKeyConfig {
    fn default() -> Self {
        Self {
            key_path: "~/.context/master.key".to_string(),
            rotation_days: 90,
        }
    }
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Encryption algorithm to use
    pub algorithm: EncryptionAlgorithm,
    /// Master key configuration
    pub master_key: MasterKeyConfig,
    /// Whether encryption is enabled
    pub enabled: bool,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            master_key: MasterKeyConfig::default(),
            enabled: true,
        }
    }
}

/// Encrypted data envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    /// Encryption algorithm used
    pub algorithm: EncryptionAlgorithm,
    /// Encrypted data (ciphertext)
    pub ciphertext: Vec<u8>,
    /// Nonce/IV used for encryption
    pub nonce: Vec<u8>,
    /// Key ID for key rotation tracking
    pub key_id: String,
    /// Timestamp of encryption
    pub encrypted_at: u64,
}

/// Data encryption key (DEK) with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataEncryptionKey {
    /// The actual encryption key
    pub key: Vec<u8>,
    /// Unique identifier for this key
    pub id: String,
    /// When this key was created
    pub created_at: u64,
    /// When this key expires (for rotation)
    pub expires_at: u64,
}

/// Cryptography service for encryption/decryption operations
pub struct CryptoService {
    config: EncryptionConfig,
    master_key: Vec<u8>,
}

impl CryptoService {
    /// Create a new crypto service
    pub fn new(config: EncryptionConfig) -> Result<Self> {
        if !config.enabled {
            return Ok(Self {
                config,
                master_key: Vec::new(),
            });
        }

        let master_key_path = shellexpand::tilde(&config.master_key.key_path).to_string();
        let master_key = Self::load_or_create_master_key(&master_key_path)?;

        Ok(Self { config, master_key })
    }

    /// Encrypt data using envelope encryption
    pub fn encrypt(&self, data: &[u8]) -> Result<EncryptedEnvelope> {
        if !self.config.enabled {
            return Err(Error::generic("Encryption is disabled"));
        }

        match self.config.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.encrypt_aes256_gcm(data),
        }
    }

    /// Decrypt data from encrypted envelope
    pub fn decrypt(&self, envelope: &EncryptedEnvelope) -> Result<Vec<u8>> {
        if !self.config.enabled {
            return Err(Error::generic("Encryption is disabled"));
        }

        match envelope.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes256_gcm(envelope),
        }
    }

    /// Generate a new data encryption key
    pub fn generate_data_key(&self) -> Result<DataEncryptionKey> {
        let mut key = vec![0u8; 32]; // 256 bits for AES-256
        rng().fill(&mut key[..]);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let expires_at = now + (self.config.master_key.rotation_days as u64 * 24 * 60 * 60);

        Ok(DataEncryptionKey {
            key,
            id: format!("dek_{}", now),
            created_at: now,
            expires_at,
        })
    }

    /// Encrypt data using AES-256-GCM
    fn encrypt_aes256_gcm(&self, data: &[u8]) -> Result<EncryptedEnvelope> {
        // Use master key for encryption (simplified envelope encryption)
        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);

        // Generate a random nonce
        let mut nonce_bytes = vec![0u8; 12]; // 96 bits for GCM
        rng().fill(&mut nonce_bytes[..]);
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| Error::generic(format!("AES encryption failed: {}", e)))?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(EncryptedEnvelope {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            ciphertext,
            nonce: nonce_bytes,
            key_id: "master".to_string(), // Using master key
            encrypted_at: timestamp,
        })
    }

    /// Decrypt data using AES-256-GCM
    fn decrypt_aes256_gcm(&self, envelope: &EncryptedEnvelope) -> Result<Vec<u8>> {
        // Use master key for decryption
        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);

        let nonce = aes_gcm::Nonce::from_slice(&envelope.nonce);

        let plaintext = cipher
            .decrypt(nonce, envelope.ciphertext.as_ref())
            .map_err(|e| Error::generic(format!("AES decryption failed: {}", e)))?;

        Ok(plaintext)
    }

    /// Load or create master key from file
    fn load_or_create_master_key(key_path: &str) -> Result<Vec<u8>> {
        let path = Path::new(key_path);

        if path.exists() {
            // Load existing key
            let key_data = fs::read(path)
                .map_err(|e| Error::io(format!("Failed to read master key file: {}", e)))?;

            if key_data.len() != 32 {
                return Err(Error::generic("Invalid master key file: wrong size"));
            }

            Ok(key_data)
        } else {
            // Create new key
            let mut key = vec![0u8; 32];
            rng().fill(&mut key[..]);

            // Create directory if it doesn't exist
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| Error::io(format!("Failed to create key directory: {}", e)))?;
            }

            // Save the key
            fs::write(path, &key)
                .map_err(|e| Error::io(format!("Failed to write master key file: {}", e)))?;

            Ok(key)
        }
    }

    /// Check if encryption is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get configuration
    pub fn config(&self) -> &EncryptionConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_encryption_config_default() {
        let config = EncryptionConfig::default();
        assert!(matches!(config.algorithm, EncryptionAlgorithm::Aes256Gcm));
        assert!(config.enabled);
    }

    #[test]
    fn test_encryption_disabled() {
        let config = EncryptionConfig {
            enabled: false,
            ..Default::default()
        };
        let crypto = CryptoService::new(config).unwrap();
        assert!(!crypto.is_enabled());
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        let temp_dir = tempdir().unwrap();
        let key_path = temp_dir
            .path()
            .join("test_master.key")
            .to_string_lossy()
            .to_string();

        let config = EncryptionConfig {
            master_key: MasterKeyConfig {
                key_path,
                rotation_days: 90,
            },
            ..Default::default()
        };

        let crypto = CryptoService::new(config).unwrap();

        let original_data = b"Hello, this is a test message for encryption!";
        let encrypted = crypto.encrypt(original_data).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();

        assert_eq!(original_data.to_vec(), decrypted);
        assert_eq!(encrypted.algorithm, EncryptionAlgorithm::Aes256Gcm);
        assert!(!encrypted.ciphertext.is_empty());
        assert_eq!(encrypted.nonce.len(), 12); // GCM nonce size
    }

    #[tokio::test]
    async fn test_data_key_generation() {
        let config = EncryptionConfig::default();
        let crypto = CryptoService::new(config).unwrap();

        let data_key = crypto.generate_data_key().unwrap();
        assert_eq!(data_key.key.len(), 32); // AES-256 key size
        assert!(data_key.id.starts_with("dek_"));
        assert!(data_key.expires_at > data_key.created_at);
    }

    #[tokio::test]
    async fn test_master_key_persistence() {
        let temp_dir = tempdir().unwrap();
        let key_path = temp_dir
            .path()
            .join("persistent_master.key")
            .to_string_lossy()
            .to_string();

        // Create first instance
        let config1 = EncryptionConfig {
            master_key: MasterKeyConfig {
                key_path: key_path.clone(),
                rotation_days: 90,
            },
            ..Default::default()
        };
        let crypto1 = CryptoService::new(config1).unwrap();

        // Create second instance - should load the same key
        let config2 = EncryptionConfig {
            master_key: MasterKeyConfig {
                key_path,
                rotation_days: 90,
            },
            ..Default::default()
        };
        let crypto2 = CryptoService::new(config2).unwrap();

        // Both should work with the same master key
        let test_data = b"Test data for key persistence";
        let encrypted = crypto1.encrypt(test_data).unwrap();
        let decrypted = crypto2.decrypt(&encrypted).unwrap();

        assert_eq!(test_data.to_vec(), decrypted);
    }
}
