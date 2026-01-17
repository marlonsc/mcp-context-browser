//! Password hashing service using Argon2

use argon2::{
    password_hash::{rand_core::OsRng as ArgonOsRng, PasswordHash, PasswordVerifier, SaltString},
    Argon2, PasswordHasher,
};
use mcb_domain::error::{Error, Result};

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
        let salt = SaltString::generate(&mut ArgonOsRng);

        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| Error::Infrastructure {
                message: format!("Password hashing failed: {}", e),
                source: None,
            })?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against its hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash).map_err(|e| Error::Authentication {
            message: format!("Invalid password hash format: {}", e),
            source: None,
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
