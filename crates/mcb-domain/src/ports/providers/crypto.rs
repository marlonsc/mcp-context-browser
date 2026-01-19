//! Cryptographic Provider Port
//!
//! Defines the interface for cryptographic operations used by providers
//! that need encryption capabilities (e.g., EncryptedVectorStoreProvider).
//!
//! ## Usage
//!
//! This port follows the Dependency Inversion Principle:
//! - The trait is defined here (mcb-domain)
//! - Implementations live in mcb-infrastructure (CryptoService)
//! - Providers depend on the abstraction, not the concrete implementation

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Cryptographic provider port
///
/// Defines the contract for encryption/decryption operations.
/// Implementations provide the actual cryptographic primitives (e.g., AES-256-GCM).
///
/// # Example
///
/// ```ignore
/// use mcb_domain::ports::providers::CryptoProvider;
///
/// async fn encrypt_metadata(
///     crypto: &dyn CryptoProvider,
///     data: &[u8],
/// ) -> Result<EncryptedData> {
///     crypto.encrypt(data)
/// }
/// ```
#[async_trait]
pub trait CryptoProvider: Send + Sync {
    /// Encrypt plaintext data
    ///
    /// # Arguments
    ///
    /// * `plaintext` - The data to encrypt
    ///
    /// # Returns
    ///
    /// Encrypted data container with ciphertext and nonce
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData>;

    /// Decrypt encrypted data
    ///
    /// # Arguments
    ///
    /// * `encrypted_data` - The encrypted data container
    ///
    /// # Returns
    ///
    /// The decrypted plaintext
    fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>>;

    /// Get the name/identifier of this provider implementation
    fn provider_name(&self) -> &str;
}

/// Encrypted data container
///
/// Holds the ciphertext and nonce produced by encryption.
/// Can be serialized for storage in vector store metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedData {
    /// The encrypted ciphertext
    pub ciphertext: Vec<u8>,
    /// The nonce used for encryption
    pub nonce: Vec<u8>,
}

impl EncryptedData {
    /// Create a new encrypted data container
    pub fn new(ciphertext: Vec<u8>, nonce: Vec<u8>) -> Self {
        Self { ciphertext, nonce }
    }
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
