//! Secure token generation

use aes_gcm::aead::{rand_core::RngCore as AeadRngCore, OsRng as AeadOsRng};

use super::utils::bytes_to_hex;

/// Secure token generation
pub struct TokenGenerator;

impl TokenGenerator {
    /// Generate a cryptographically secure random token
    pub fn generate_secure_token(length: usize) -> String {
        let mut bytes = vec![0u8; length];
        AeadOsRng.fill_bytes(&mut bytes);
        bytes_to_hex(&bytes)
    }

    /// Generate a URL-safe secure token
    pub fn generate_url_safe_token(length: usize) -> String {
        let mut bytes = vec![0u8; length];
        AeadOsRng.fill_bytes(&mut bytes);
        use base64::{engine::general_purpose, Engine as _};
        general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    }

    /// Generate a UUID v4
    pub fn generate_uuid() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
