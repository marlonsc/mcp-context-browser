//! Cryptography tests
//!
//! Tests migrated from src/infrastructure/crypto.rs

use mcp_context_browser::infrastructure::crypto::{
    CryptoService, EncryptionAlgorithm, EncryptionConfig, MasterKeyConfig,
};
use tempfile::tempdir;

#[test]
fn test_encryption_config_default() {
    let config = EncryptionConfig::default();
    assert!(matches!(config.algorithm, EncryptionAlgorithm::Aes256Gcm));
    assert!(config.enabled);
}

#[tokio::test]
async fn test_encryption_disabled() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = EncryptionConfig {
        enabled: false,
        ..Default::default()
    };
    let crypto = CryptoService::new(config).await?;
    assert!(!crypto.is_enabled());
    Ok(())
}

#[tokio::test]
async fn test_encrypt_decrypt_roundtrip() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
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

    let crypto = CryptoService::new(config).await?;

    let original_data = b"Hello, this is a test message for encryption!";
    let encrypted = crypto.encrypt(original_data)?;
    let decrypted = crypto.decrypt(&encrypted)?;

    assert_eq!(original_data.to_vec(), decrypted);
    assert_eq!(encrypted.algorithm, EncryptionAlgorithm::Aes256Gcm);
    assert!(!encrypted.ciphertext.is_empty());
    assert_eq!(encrypted.nonce.len(), 12); // GCM nonce size
    Ok(())
}

#[tokio::test]
async fn test_data_key_generation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = EncryptionConfig::default();
    let crypto = CryptoService::new(config).await?;

    let data_key = crypto.generate_data_key()?;
    assert_eq!(data_key.key.len(), 32); // AES-256 key size
    assert!(data_key.id.starts_with("dek_"));
    assert!(data_key.expires_at > data_key.created_at);
    Ok(())
}

#[tokio::test]
async fn test_master_key_persistence() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
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
    let crypto1 = CryptoService::new(config1).await?;

    // Create second instance - should load the same key
    let config2 = EncryptionConfig {
        master_key: MasterKeyConfig {
            key_path,
            rotation_days: 90,
        },
        ..Default::default()
    };
    let crypto2 = CryptoService::new(config2).await?;

    // Both should work with the same master key
    let test_data = b"Test data for key persistence";
    let encrypted = crypto1.encrypt(test_data)?;
    let decrypted = crypto2.decrypt(&encrypted)?;

    assert_eq!(test_data.to_vec(), decrypted);
    Ok(())
}
