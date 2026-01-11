//! Cryptography module for encryption at rest
//!
//! Provides AES-256-GCM encryption for sensitive data stored on disk.
//! Implements envelope encryption with data keys and master keys.



use crate::domain::error::{Error, Result};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key,
};
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};

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
    pub async fn new(config: EncryptionConfig) -> Result<Self> {
        if !config.enabled {
            return Ok(Self {
                config,
                master_key: Vec::new(),
            });
        }

        let master_key_path = shellexpand::tilde(&config.master_key.key_path).to_string();
        let master_key = Self::load_or_create_master_key(&master_key_path).await?;

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
    async fn load_or_create_master_key(key_path: &str) -> Result<Vec<u8>> {
        let path = std::path::PathBuf::from(key_path);

        if path.exists() {
            // Load existing key
            let key_data = tokio::fs::read(&path)
                .await
                .map_err(|e| Error::io(format!("Failed to read master key file: {}", e)))?;

            if key_data.len() != 32 {
                return Err(Error::generic("Invalid master key file: wrong size"));
            }

            // Verify permissions on Unix
            #[cfg(unix)]
            {
                let path_clone = path.clone();
                tokio::task::spawn_blocking(move || {
                    use std::os::unix::fs::PermissionsExt;
                    let metadata = std::fs::metadata(&path_clone)
                        .map_err(|e| Error::io(format!("Failed to get key metadata: {}", e)))?;
                    let mode = metadata.permissions().mode();
                    if mode & 0o077 != 0 {
                        // Permissions are too open, try to fix them
                        let mut perms = metadata.permissions();
                        perms.set_mode(0o600);
                        std::fs::set_permissions(&path_clone, perms).map_err(|e| {
                            Error::io(format!("Failed to secure key permissions: {}", e))
                        })?;
                    }
                    Ok::<(), Error>(())
                })
                .await
                .map_err(|e| Error::generic(format!("Blocking task failed: {}", e)))??;
            }

            Ok(key_data)
        } else {
            // Create new key
            let mut key = vec![0u8; 32];
            rng().fill(&mut key[..]);

            // Create directory if it doesn't exist
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| Error::io(format!("Failed to create key directory: {}", e)))?;
            }

            // Save the key with restricted permissions (0600)
            #[cfg(unix)]
            {
                let path_clone = path.clone();
                let key_clone = key.clone();
                tokio::task::spawn_blocking(move || {
                    use std::io::Write;
                    use std::os::unix::fs::OpenOptionsExt;

                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .mode(0o600)
                        .open(&path_clone)
                        .map_err(|e| {
                            Error::io(format!("Failed to create master key file: {}", e))
                        })?;

                    file.write_all(&key_clone)
                        .map_err(|e| Error::io(format!("Failed to write master key: {}", e)))?;
                    Ok::<(), Error>(())
                })
                .await
                .map_err(|e| Error::generic(format!("Blocking task failed: {}", e)))??;
            }
            #[cfg(not(unix))]
            {
                tokio::fs::write(&path, &key)
                    .await
                    .map_err(|e| Error::io(format!("Failed to write master key file: {}", e)))?;
            }

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
