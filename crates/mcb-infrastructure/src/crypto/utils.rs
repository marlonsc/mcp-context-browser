//! Cryptographic utilities

use aes_gcm::aead::{OsRng as AeadOsRng, rand_core::RngCore as AeadRngCore};
use mcb_domain::error::{Error, Result};
use sha2::Sha256;

/// Convert bytes to hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Key derivation utilities
pub struct KeyDerivation;

impl KeyDerivation {
    /// Derive a key from password using PBKDF2
    pub fn pbkdf2(password: &str, salt: &[u8], iterations: u32, key_len: usize) -> Vec<u8> {
        use pbkdf2::pbkdf2_hmac;

        let mut key = vec![0u8; key_len];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut key);
        key
    }

    /// Generate a random salt
    pub fn generate_salt(length: usize) -> Vec<u8> {
        let mut salt = vec![0u8; length];
        AeadOsRng.fill_bytes(&mut salt);
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
        AeadOsRng.fill_bytes(data);
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
    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;
        let mut mac =
            <HmacSha256 as Mac>::new_from_slice(key).map_err(|e| Error::Infrastructure {
                message: format!("HMAC initialization failed: {}", e),
                source: None,
            })?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
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
